//! Editor top-bar menu API

use std::sync::Mutex;

use wutengine_egui::egui;

mod default;
pub(crate) use default::*;

static MENU_MANAGER: MenuManager = MenuManager::new();

struct MenuManager {
    top_level_entries: Mutex<Vec<MenuEntry>>,
}

impl MenuManager {
    const fn new() -> Self {
        Self {
            top_level_entries: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Debug)]
struct MenuEntry {
    location: u64,
    name: String,
    content: MenuContent,
}

impl MenuEntry {
    const SEPARATOR_INTERVAL: u64 = 100;
    fn is_empty(&self) -> bool {
        match &self.content {
            MenuContent::SubMenu(items) => items.is_empty(),
            MenuContent::Callback(_) => false,
            MenuContent::Ui(_) => false,
        }
    }

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
                    for item in items {
                        item.show(prev_location, ui);
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

#[derive(derive_more::Debug)]
enum MenuContent {
    SubMenu(Vec<MenuEntry>),

    #[debug("Callback")]
    Callback(Box<dyn Fn() + Send + Sync>),

    #[debug("Sub-UI")]
    Ui(Box<dyn Fn(&mut egui::Ui) + Send + Sync>),
}

/// Adds a menu entry at the given path
pub(crate) fn add_entry(path: &[&str], location: u64, callback: impl Fn() + Send + Sync + 'static) {
    if path.is_empty() {
        log::error!("Cannot insert menu button with empty path");
        return;
    }

    let new_entry = MenuEntry {
        location,
        name: path.last().unwrap().to_string(),
        content: MenuContent::Callback(Box::new(callback)),
    };

    add_entry_raw(path, new_entry);
}

/// Adds a menu entry containing a custom UI callback at the given path
pub(crate) fn add_entry_ui(
    path: &[&str],
    location: u64,
    callback: impl Fn(&mut egui::Ui) + Send + Sync + 'static,
) {
    let new_entry = MenuEntry {
        location,
        name: path.last().unwrap().to_string(),
        content: MenuContent::Ui(Box::new(callback)),
    };

    add_entry_raw(path, new_entry);
}

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

fn format_menu_path(path: &[&str]) -> String {
    path.join("/").to_string()
}

#[derive(Debug, Clone, Copy, derive_more::Error, derive_more::Display)]
enum InsertErr {
    #[display("Entry already exists")]
    AlreadyExists,

    #[display("An entry along the menu path already exists as a non-menu entry")]
    NotASubMenu,
}

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

fn clean_menu(entries: &mut Vec<MenuEntry>) {
    for entry in entries.iter_mut() {
        entry.clean();
    }

    entries.retain(|entry| !entry.is_empty());
    entries.sort_by_key(|e| e.location);
}

/// Shows the menu inside the current UI
pub(crate) fn show(ui: &mut egui::Ui) {
    egui::MenuBar::new().ui(ui, |ui| {
        let menu = MENU_MANAGER.top_level_entries.lock().unwrap();

        let mut prev_location = None;

        for entry in menu.iter() {
            entry.show(&mut prev_location, ui);
        }
    });
}
