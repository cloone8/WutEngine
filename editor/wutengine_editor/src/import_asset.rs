use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use wutengine::runtime;
use wutengine::task::TaskHandle;
use wutengine_egui::egui;

use crate::project;

pub(crate) static IMPORT_QUEUE: Mutex<VecDeque<ImportJob>> = Mutex::new(VecDeque::new());

pub(crate) struct ImportJob {
    pub(crate) path: PathBuf,
    pub(crate) file_type: String,
    pub(crate) destination_dir: String,
    pub(crate) name: String,
    pub(crate) pick_new_dir_job: Option<TaskHandle<Option<String>>>,
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
            ui.label(format!("Destination directory: /{}", self.destination_dir));

            if ui.button("Pick directory...").clicked() && self.pick_new_dir_job.is_none() {
                let cur_destination_dir = project::asset_manager()
                    .asset_root()
                    .join(self.destination_dir.clone());

                self.pick_new_dir_job = Some(runtime::run_on_main_thread(async move {
                    let picked_folder = rfd::AsyncFileDialog::new()
                        .set_directory(cur_destination_dir)
                        .pick_folder()
                        .await?;

                    let picked_folder_path = picked_folder.path().canonicalize().unwrap();

                    if !picked_folder_path.starts_with(project::asset_manager().asset_root()) {
                        log::error!("Picked import destination is not within project");
                        return None;
                    }

                    let picked_path_relative = picked_folder_path
                        .strip_prefix(project::asset_manager().asset_root())
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();

                    Some(picked_path_relative)
                }))
            }
        });

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.separator();

        if ui.button("Done").clicked() {
            return true;
        }

        false
    }
}

impl ImportJob {
    pub(crate) fn new_from_path(path: PathBuf) -> Self {
        Self {
            file_type: Self::get_file_type_from_path(&path).unwrap_or_default(),
            destination_dir: String::new(),
            name: Self::get_file_name_from_path(&path).unwrap_or_default(),
            path,
            pick_new_dir_job: None,
        }
    }

    /// Returns the file type from a path, if it has an extension
    fn get_file_type_from_path(path: &Path) -> Option<String> {
        let ext = path.extension()?;

        ext.to_str().map(|s| s.to_string())
    }

    /// Returns the file type from a path, if it has an extension
    fn get_file_name_from_path(path: &Path) -> Option<String> {
        let stem = path.file_stem()?;

        stem.to_str().map(|s| s.to_string())
    }
}

pub(crate) fn import_asset_prompt() -> TaskHandle<Option<Vec<PathBuf>>> {
    runtime::run_on_main_thread(async {
        let picked = rfd::AsyncFileDialog::new()
            .set_title("Select the asset to import")
            .pick_files()
            .await?;

        Some(
            picked
                .into_iter()
                .map(|picked_file| picked_file.path().to_path_buf())
                .collect(),
        )
    })
}
