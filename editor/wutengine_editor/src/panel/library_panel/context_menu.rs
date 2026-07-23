use std::path::Path;

use wutengine_egui::egui;

use crate::assets;
use crate::assets::path::AssetPath;

pub(super) fn dir(path: &AssetPath, ui: &mut egui::Ui) {
    if ui.button("Import here...").clicked() {
        assets::import::import_asset_prompt(Some(path.clone()));
    }

    ui.menu_button("New asset", |ui| {
        assets::create::create_asset_buttons(path, ui);
    });

    ui.separator();

    shared_menu(path, ui);
}

pub(super) fn asset(asset_id: &uuid::NonNilUuid, path: &AssetPath, ui: &mut egui::Ui) {
    shared_menu(path, ui);
}

fn shared_menu(path: &AssetPath, ui: &mut egui::Ui) {
    if ui.button("Show in explorer").clicked() {
        show_in_explorer(path.absolute());
    }

    if ui.button("Delete").clicked() {
        log::warn!("Not yet implemented: delete");
    }

    if ui.button("Rename").clicked() {
        log::warn!("Not yet implemented: rename");
    }
}

fn show_in_explorer(path: &Path) {
    cfg_select! {
        windows => {
            std::process::Command::new("explorer")
                .arg("/select,")
                .arg(path.as_os_str())
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
        target_os = "macos" => {
            std::process::Command::new("open")
                .arg("-R")
                .arg(path.as_os_str())
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
        target_os = "linux" => {
            let mut path_string = std::ffi::OsString::new();
            path_string.push("['");
            path_string.push(path.as_os_str());
            path_string.push("']");

            std::process::Command::new("gdbus")
                .arg("call")
                .arg("--session")
                .arg("--dest")
                .arg("org.freedesktop.FileManager1")
                .arg("--object-path")
                .arg("/org/freedesktop/FileManager1")
                .arg("--method")
                .arg("org.freedesktop.FileManager1.ShowItems")
                .arg(path_string)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
        _ => {
            unimplemented!("Not yet implemented for current platform")
        }
    }
}
