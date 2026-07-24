//! Main editor menu API

use std::sync::Mutex;
use std::sync::Once;

use wutengine_egui::egui;

/// The main menu container
static MAIN_MENU: Mutex<we_menu::Menu<()>> = Mutex::new(we_menu::Menu::new());

/// Adds an entry to the main editor menu
pub(crate) fn add_entry(path: &[&str], location: u64, callback: impl Fn() + Send + Sync + 'static) {
    let mut menu_lock = MAIN_MENU.lock().unwrap();

    if let Err(e) = menu_lock.add_entry(path, location, move |()| callback()) {
        log::error!("Failed to add menu entry: {e}");
    }
}

/// Adds a custom UI entry to the main editor menu
pub(crate) fn add_entry_ui(
    path: &[&str],
    location: u64,
    callback: impl Fn(&mut egui::Ui) + Send + Sync + 'static,
) {
    let mut menu_lock = MAIN_MENU.lock().unwrap();

    if let Err(e) = menu_lock.add_entry_ui(path, location, move |(), ui| callback(ui)) {
        log::error!("Failed to add menu entry: {e}");
    }
}

/// Shows the main editor menu within the given UI
pub(crate) fn show(ui: &mut egui::Ui) {
    let mut menu_lock = MAIN_MENU.lock().unwrap();

    #[cfg(debug_assertions)]
    add_debug_menu_once(&mut menu_lock);

    menu_lock.show(&(), ui);
}

#[cfg(debug_assertions)]
fn add_debug_menu_once(menu: &mut we_menu::Menu<()>) {
    static ONCE: Once = Once::new();

    ONCE.call_once(move || {
        menu.debug_menu = Some(Box::new(|(), ui| {
            ui.menu_button(
                egui::RichText::new("Editor Debug")
                    .background_color(egui::Color32::LIGHT_RED)
                    .color(egui::Color32::BLACK),
                |ui| {
                    ui.menu_button("egui", |ui| {
                        let mut cur_debug_opts = ui.ctx().global_style().debug;

                        cur_debug_opts.ui(ui);

                        ui.ctx()
                            .global_style_mut(|style| style.debug = cur_debug_opts);
                    });
                },
            );
        }));
    });
}
