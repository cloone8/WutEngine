//! Asset importing

use alloc::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use wutengine::task::TaskHandle;
use wutengine_egui::egui;

use crate::assets::path::AssetPath;
use crate::filepicker;
use crate::project;
use crate::project::assetmanager::ProjectAssetFormat;

pub(crate) static IMPORT_QUEUE: Mutex<VecDeque<ImportJob>> = Mutex::new(VecDeque::new());

pub(crate) struct ImportJob {
    pub(crate) path: PathBuf,
    pub(crate) file_type: String,
    pub(crate) destination_dir: AssetPath,
    pub(crate) name: String,
    pub(crate) pick_new_dir_job: Option<TaskHandle<Option<AssetPath>>>,
}

impl ImportJob {
    fn check_async_tasks(&mut self) {
        if let Some(result) = TaskHandle::get_if_started_and_ready(&mut self.pick_new_dir_job)
            && let Some(new_dir) = result
        {
            self.destination_dir = new_dir;
        }
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui) -> bool {
        self.check_async_tasks();

        ui.heading("Import asset");

        ui.label(format!("Importing: {}", self.path.to_string_lossy()));

        ui.separator();

        ui.label(format!("File type: {}", self.file_type));

        ui.horizontal(|ui| {
            ui.label(format!(
                "Destination directory: /{}",
                self.destination_dir.relative().to_string_lossy()
            ));

            if ui.button("Choose...").clicked() && self.pick_new_dir_job.is_none() {
                let picked_folder_task = filepicker::pick_folder(
                    rfd::AsyncFileDialog::new().set_directory(self.destination_dir.absolute()),
                );

                self.pick_new_dir_job = Some(wutengine::task::spawn_async(async move {
                    let picked_folder = picked_folder_task.get_async().await?;

                    let picked_folder_path = picked_folder.canonicalize().unwrap();

                    if !picked_folder_path.starts_with(project::asset_manager().asset_root()) {
                        log::error!("Picked import destination is not within project");
                        return None;
                    }

                    Some(AssetPath::new(picked_folder_path))
                }));
            }
        });

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.separator();

        if ui.button("Import").clicked() {
            import_asset(
                Some(self.file_type.as_str()),
                Some(self.name.as_str()),
                &self.path,
                self.destination_dir.absolute(),
            );
            return true;
        }

        false
    }
}

impl ImportJob {
    /// Create a new import job from a given path to an asset to import
    pub(crate) fn new_from_path(path: PathBuf, destination_dir: Option<&AssetPath>) -> Self {
        let destination_dir = destination_dir.cloned().unwrap_or_else(AssetPath::root);

        Self {
            file_type: get_file_type_from_path(&path)
                .map(ToString::to_string)
                .unwrap_or_default(),
            destination_dir,
            name: get_file_name_from_path(&path)
                .map(ToString::to_string)
                .unwrap_or_default(),
            path,
            pick_new_dir_job: None,
        }
    }
}

/// Prompts the user for an asset to import, and then starts an import job if a file was picked
pub(crate) fn import_asset_prompt(destination_dir: Option<AssetPath>) {
    _ = wutengine::task::spawn_async(async move {
        let import_result = filepicker::pick_files(
            rfd::AsyncFileDialog::new().set_title("Select the asset to import"),
        )
        .get_async()
        .await;

        let Some(imported_paths) = import_result else {
            return;
        };

        let mut import_queue = IMPORT_QUEUE.lock().unwrap();

        for imported_path in imported_paths {
            import_queue.push_back(ImportJob::new_from_path(
                imported_path,
                destination_dir.as_ref(),
            ));
        }
    });
}

/// Returns the file type from a path, if it has an extension
fn get_file_type_from_path(path: &Path) -> Option<&str> {
    let ext = path.extension()?;

    ext.to_str()
}

/// Returns the file type from a path, if it has an extension
fn get_file_name_from_path(path: &Path) -> Option<&str> {
    let stem = path.file_stem()?;

    stem.to_str()
}

/// Import an asset into the project. The resulting assets are placed within `destination_dir`
pub(crate) fn import_asset(
    file_type: Option<&str>,
    name: Option<&str>,
    source_file: &Path,
    destination_dir: &Path,
) {
    let file_type = if let Some(ftype) = file_type {
        ftype
    } else if let Some(ftype_ext) = get_file_type_from_path(source_file) {
        ftype_ext
    } else {
        log::error!("Failed to import asset, because the asset type could not be determined");
        return;
    };

    let Some(importers) = wutengine_asset_importers::default_importers().get(file_type) else {
        log::error!(
            "Failed to import asset of type \"{file_type}\" because no compatible importer could be found"
        );
        return;
    };

    let mut errs = Vec::new();
    let mut imported_assets = Vec::new();

    for importer in importers {
        match importer.import_from_path(file_type, source_file) {
            Ok(imported) => {
                imported_assets = imported;
                break;
            }
            Err(e) => {
                errs.push((importer.name(), e));
            }
        }
    }

    if imported_assets.is_empty() && !errs.is_empty() {
        let mut msgs = Vec::new();

        for (imp_name, err) in errs {
            msgs.push(format!("{imp_name}: {err}"));
        }

        let importer_errs_fmt = msgs.join("\n\t");

        log::error!("All compatible importers failed with errors:\n{importer_errs_fmt}");
        return;
    }

    if imported_assets.is_empty() {
        log::warn!(
            "Importing file \"{}\" did not result in any imported assets",
            source_file.to_string_lossy()
        );
    }

    for (index_in_batch, imported_asset) in imported_assets.into_iter().enumerate() {
        let asset_unique_idx = index_in_batch;

        let Some(target_type) =
            wutengine_asset_importers::default_asset_types().get(&imported_asset.asset_type_id)
        else {
            log::error!(
                "Asset type with ID {} is unknown, cannot import and serialize",
                imported_asset.asset_type_id
            );
            continue;
        };

        let (serialized, format) = if target_type.prefers_binary() {
            log::debug!("Serializing as binary");
            (
                target_type.serialize_binary(imported_asset.asset.as_ref()),
                ProjectAssetFormat::Postcard,
            )
        } else {
            log::debug!("Serializing as text");
            (
                target_type.serialize_text(imported_asset.asset.as_ref()),
                ProjectAssetFormat::Json,
            )
        };

        match serialized {
            Ok(ser) => {
                let dest_name = imported_asset.name.unwrap_or_else(|| {
                    name.map_or_else(
                        || format!("{}_{}", target_type.asset_type_name(), asset_unique_idx),
                        |name| format!("{name}_{asset_unique_idx}"),
                    )
                });

                let extension = if target_type.prefers_binary() {
                    ".we-binasset"
                } else {
                    ".we-txtasset"
                };

                let asset_file_name = format!("{dest_name}{extension}");
                let destination_path = destination_dir.join(asset_file_name);

                if let Err(e) = project::asset_manager().insert_serialized_asset(
                    &ser,
                    format,
                    imported_asset.asset_type_id,
                    &destination_path,
                ) {
                    log::error!("Failed to insert asset into project: {e}");
                } else {
                    log::info!(
                        "Imported asset {dest_name} to path {}",
                        destination_path.to_string_lossy()
                    );
                }
            }
            Err(e) => {
                log::error!("Failed to serialize asset due to error: {e}");
            }
        }
    }
}
