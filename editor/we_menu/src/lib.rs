#![doc = include_str!("../README.md")]

use std::sync::Mutex;

use wutengine_egui::egui;

/// The global [``MenuManager``]
static MENU_MANAGER: MenuManager = MenuManager::new();

/// The menu manager, contains the entries
struct MenuManager {
    /// The top level entries
    top_level_entries: Mutex<Vec<MenuEntry>>,
}

impl MenuManager {
    /// A new empty menu manager
    const fn new() -> Self {
        Self {
            top_level_entries: Mutex::new(Vec::new()),
        }
    }
}

/// A menu entry
#[derive(Debug)]
struct MenuEntry {
    /// The location relative to other entries. Lower is earlier
    location: u64,

    /// The name of this entry
    name: String,

    /// The contents of this entry
    content: MenuContent,
}

impl MenuEntry {
    /// The interval between `location` values between which a separator is inserted
    const SEPARATOR_INTERVAL: u64 = 100;

    /// Returns whether this entry has no children or actions
    fn is_empty(&self) -> bool {
        match &self.content {
            MenuContent::SubMenu(items) => items.is_empty(),
            MenuContent::Callback(_) | MenuContent::Ui(_) => false,
        }
    }

    /// Cleans this entry and its children
    fn clean(&mut self) {
        let MenuContent::SubMenu(entries) = &mut self.content else {
            return;
        };

        assert!(
            !entries.is_empty(),
            "Cannot have zero entries in a submenu entry"
        );

        for entry in entries.iter_mut() {
            entry.clean();
        }

        entries.sort_by_key(|entry| entry.location);

        self.location = entries
            .first()
            .map(|entry| entry.location)
            .unwrap_or_default();
    }

    /// Shows this entry
    fn show(&self, prev_location: &mut Option<u64>, ui: &mut egui::Ui) {
        if let Some(prev_location) = prev_location
            && (prev_location.saturating_add(Self::SEPARATOR_INTERVAL)) <= self.location
        {
            ui.separator();
        }

        *prev_location = Some(self.location);

        match &self.content {
            MenuContent::SubMenu(items) => {
                ui.menu_button(&self.name, |ui| {
                    let mut prev_location = None;

                    for item in items {
                        item.show(&mut prev_location, ui);
                    }
                });
            }
            MenuContent::Callback(cb) => {
                if ui.button(&self.name).clicked() {
                    cb();
                }
            }
            MenuContent::Ui(ui_cb) => {
                ui.menu_button(&self.name, ui_cb);
            }
        }
    }
}

/// The contents of a menu entry
#[derive(derive_more::Debug)]
enum MenuContent {
    /// A submenu
    SubMenu(Vec<MenuEntry>),

    /// A button that calls a callback
    #[debug("Callback")]
    Callback(Box<dyn Fn() + Send + Sync>),

    /// A submenu that shows a custom UI
    #[debug("Sub-UI")]
    Ui(Box<dyn Fn(&mut egui::Ui) + Send + Sync>),
}

/// Adds a menu entry at the given path
pub fn add_entry(path: &[&str], location: u64, callback: impl Fn() + Send + Sync + 'static) {
    let name = if let Some(last) = path.last() {
        last.to_string()
    } else {
        log::error!("Cannot insert menu button with empty path");
        return;
    };

    let new_entry = MenuEntry {
        location,
        name,
        content: MenuContent::Callback(Box::new(callback)),
    };

    add_entry_raw(path, new_entry);
}

/// Adds a menu entry containing a custom UI callback at the given path
pub fn add_entry_ui(
    path: &[&str],
    location: u64,
    callback: impl Fn(&mut egui::Ui) + Send + Sync + 'static,
) {
    let name = if let Some(last) = path.last() {
        last.to_string()
    } else {
        log::error!("Cannot insert menu button with empty path");
        return;
    };

    let new_entry = MenuEntry {
        location,
        name,
        content: MenuContent::Ui(Box::new(callback)),
    };

    add_entry_raw(path, new_entry);
}

/// Adds a raw menu entry
fn add_entry_raw(path: &[&str], new_entry: MenuEntry) {
    let mut top_entries = MENU_MANAGER.top_level_entries.lock().unwrap();

    if let Err(e) = insert_recursive(&mut top_entries, path, new_entry) {
        log::error!(
            "Failed to insert menu entry \"{}\": {e}",
            format_menu_path(path)
        );
        return;
    }

    clean_menu(&mut top_entries);
}

/// Formats a menu path for display
fn format_menu_path(path: &[&str]) -> String {
    path.join("/").clone()
}

/// An error while inserting a menu item
#[derive(Debug, Clone, Copy, derive_more::Error, derive_more::Display)]
enum InsertErr {
    /// Item already exists
    #[display("Entry already exists")]
    AlreadyExists,

    /// An insertion would replace a non-menu entry with a submenu
    #[display("An entry along the menu path already exists as a non-menu entry")]
    NotASubMenu,
}

/// Recursively travels down the menu list according to the path components given in `path` to insert the given entry
fn insert_recursive(
    entries: &mut Vec<MenuEntry>,
    path: &[&str],
    to_insert: MenuEntry,
) -> Result<(), InsertErr> {
    assert!(!path.is_empty(), "Cannot insert at empty path");

    let head = path.first().unwrap();
    let tail = &path[1..];

    if tail.is_empty() {
        // We've reached our insertion point. Check that the entry doesn't already exist and then insert it
        if entries.iter().any(|entry| entry.name == *head) {
            return Err(InsertErr::AlreadyExists);
        }

        entries.push(to_insert);
    } else {
        let parent_entry = match entries
            .iter_mut()
            .find(|entry| entry.name.as_str() == *head)
        {
            Some(existing_entry) => existing_entry,
            None => entries.push_mut(MenuEntry {
                location: to_insert.location,
                name: head.to_string(),
                content: MenuContent::SubMenu(Vec::with_capacity(1)),
            }),
        };

        let MenuContent::SubMenu(parent_entry_list) = &mut parent_entry.content else {
            return Err(InsertErr::NotASubMenu);
        };

        insert_recursive(parent_entry_list, tail, to_insert)?;
    }

    Ok(())
}

/// After menu entries have been changed, this function cleans up the list by removing empty menu's and
/// sorting menu's by their location field
fn clean_menu(entries: &mut Vec<MenuEntry>) {
    for entry in entries.iter_mut() {
        entry.clean();
    }

    entries.retain(|entry| !entry.is_empty());
    entries.sort_by_key(|e| e.location);
}

/// Shows the menu inside the current UI
pub fn show(ui: &mut egui::Ui) {
    egui::MenuBar::new().ui(ui, |ui| {
        {
            let visuals = ui.visuals_mut();
            visuals.button_frame = false;
        }

        let menu = MENU_MANAGER.top_level_entries.lock().unwrap();

        let mut prev_location = None;

        for entry in menu.iter() {
            entry.show(&mut prev_location, ui);
        }

        #[cfg(debug_assertions)]
        show_debug(ui);
    });
}

/// Shows the editor debug menu
#[cfg(debug_assertions)]
fn show_debug(ui: &mut egui::Ui) {
    ui.scope_builder(
        egui::UiBuilder::new().layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
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
        },
    );
}
