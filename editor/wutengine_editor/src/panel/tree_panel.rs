use core::convert::Infallible;
use std::sync::Arc;

use wutengine::asset::assets::entity::EntityEntry;
use wutengine::asset::assets::entity::SerializedEntity;
use wutengine::asset::assets::level::LevelEntry;
use wutengine::asset::assets::level::SerializedLevel;
use wutengine::asset_server::GetAssetErr;
use wutengine::task::TaskHandle;
use wutengine_egui::egui;

use crate::assets;
use crate::assets::gui::AssetGui;
use crate::project;

use super::EditorPanel;

/// The panel showing the main level hierarchy tree
#[derive(Debug)]
pub(crate) struct TreePanel {
    open_levels: Vec<OpenLevel>,
}

impl TreePanel {
    fn update_open_levels(&mut self) {
        let proj_open_levels = project::open_levels();

        // Unload all levels that are not actually open (anymore)
        self.open_levels
            .retain(|open_level| proj_open_levels.contains(open_level.id()));

        // Start loading any new levels
        for level_id in &proj_open_levels {
            if !self.open_levels.iter().any(|lvl| lvl.id() == level_id) {
                let new_open_level = OpenLevel::Loading {
                    id: *level_id,
                    order_index: 0,
                    load_task: Some(assets::server::load_id::<SerializedLevel>(level_id)),
                };

                self.open_levels.push(new_open_level);
            }
        }

        // Order the levels by their index in the project open levels
        for (i, level_id) in proj_open_levels.into_iter().enumerate() {
            let open_level = self
                .open_levels
                .iter_mut()
                .find(|open_level| open_level.id() == &level_id)
                .expect("Missing open level");

            open_level.set_order_index(i);
        }
    }
}

#[derive(Debug)]
enum OpenLevel {
    Loading {
        id: uuid::NonNilUuid,
        order_index: usize,
        load_task: Option<TaskHandle<Result<Arc<SerializedLevel>, GetAssetErr<Infallible>>>>,
    },
    Ready {
        id: uuid::NonNilUuid,
        order_index: usize,
        level: Arc<SerializedLevel>,
    },
    LoadErr {
        id: uuid::NonNilUuid,
        order_index: usize,
        err: String,
    },
}

impl OpenLevel {
    fn id(&self) -> &uuid::NonNilUuid {
        match self {
            Self::Loading { id, .. } => id,
            Self::Ready { id, .. } => id,
            Self::LoadErr { id, .. } => id,
        }
    }

    fn set_order_index(&mut self, idx: usize) {
        match self {
            Self::Loading { order_index, .. } => *order_index = idx,
            Self::Ready { order_index, .. } => *order_index = idx,
            Self::LoadErr { order_index, .. } => *order_index = idx,
        }
    }

    fn check_loaded(&mut self) {
        let OpenLevel::Loading {
            id,
            order_index,
            load_task,
        } = self
        else {
            // Already loaded
            return;
        };

        let Some(done_load_task) = load_task.take_if(|task| task.ready()) else {
            // Still loading
            return;
        };

        *self = match done_load_task.get() {
            Ok(level) => OpenLevel::Ready {
                id: *id,
                order_index: *order_index,
                level,
            },
            Err(err) => OpenLevel::LoadErr {
                id: *id,
                order_index: *order_index,
                err: err.to_string(),
            },
        };
    }
}

impl EditorPanel for TreePanel {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Tree"
    }

    fn construct(_id: super::EditorPanelId) -> Box<dyn EditorPanel>
    where
        Self: Sized,
    {
        Box::new(Self {
            open_levels: Vec::new(),
        })
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        const LEVEL_ICON: &str = SerializedLevel::ICON;

        self.update_open_levels();

        if self.open_levels.is_empty() {
            ui.label("No levels loaded. Open a level from the project library panel");
            return;
        }

        for open_level in &mut self.open_levels {
            open_level.check_loaded();

            match open_level {
                OpenLevel::Loading { .. } => {
                    ui.label(format!("{LEVEL_ICON} Loading..."));
                }
                OpenLevel::Ready { level, id, .. } => {
                    egui::CollapsingHeader::new(format!("{LEVEL_ICON} {}", level.name))
                        .default_open(true)
                        .id_salt(*id)
                        .show(ui, |ui| {
                            for (i, entry) in level.entries.iter().enumerate() {
                                show_level_entry(entry, i, ui);
                            }
                        });
                }
                OpenLevel::LoadErr { err, id, .. } => {
                    ui.label(format!("{LEVEL_ICON} {id}"));
                    ui.label(format!("ERROR: {err}"));
                }
            }
        }
    }
}

fn show_level_entry(entry: &LevelEntry, idx: usize, ui: &mut egui::Ui) {
    match entry {
        LevelEntry::Entity(serialized_entity) => {
            show_serialized_entity(serialized_entity, idx, ui);
        }
        LevelEntry::Bundle(asset_ref) => todo!(),
    }
}

fn show_entity_entry(entry: &EntityEntry, idx: usize, ui: &mut egui::Ui) {
    match entry {
        EntityEntry::Entity(serialized_entity) => {
            show_serialized_entity(serialized_entity, idx, ui);
        }
        EntityEntry::Bundle(asset_ref) => todo!(),
    }
}
fn show_serialized_entity(entity: &SerializedEntity, idx: usize, ui: &mut egui::Ui) {
    if entity.children.is_empty() {
        ui.label(entity.name.as_str());
        return;
    }

    egui::CollapsingHeader::new(entity.name.as_str())
        .default_open(false)
        .id_salt(idx)
        .show(ui, |ui| {
            for (i, entry) in entity.children.iter().enumerate() {
                show_entity_entry(entry, i, ui);
            }
        });
}

fn show_bundle_entry() {}
