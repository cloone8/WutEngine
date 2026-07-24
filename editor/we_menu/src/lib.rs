#![doc = include_str!("../README.md")]

use wutengine_egui::egui;

#[cfg(debug_assertions)]
/// Type of the callback for the debug submenu
pub type DebugFn<T> = dyn Fn(&T, &mut egui::Ui) + Send + Sync;

/// A generic GUI menu container
#[derive(derive_more::Debug)]
pub struct Menu<T>
where
    T: ?Sized,
{
    /// The top-level menu entries
    entries: Vec<MenuEntry<T>>,

    /// The interval between `location` values between which a separator is inserted
    pub separator_interval: u64,

    #[cfg(debug_assertions)]
    #[debug("debug menu: {}", debug_menu.is_some())]
    /// The debug menu, if any
    pub debug_menu: Option<Box<DebugFn<T>>>,
}

impl<T> Menu<T>
where
    T: ?Sized,
{
    /// The default interval between menu items before a separator is inserted
    pub const DEFAULT_SEPARATOR_INTERVAL: u64 = 100;

    /// Create a new empty [`Menu`]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            separator_interval: Self::DEFAULT_SEPARATOR_INTERVAL,

            #[cfg(debug_assertions)]
            debug_menu: None,
        }
    }

    /// Adds a menu entry at the given path
    pub fn add_entry(
        &mut self,
        path: &[&str],
        location: u64,
        callback: impl Fn(&T) + Send + Sync + 'static,
    ) -> Result<(), InsertErr> {
        let name = if let Some(last) = path.last() {
            last.to_string()
        } else {
            return Err(InsertErr::EmptyPath);
        };

        let new_entry = MenuEntry {
            location,
            name,
            content: MenuContent::Callback(Box::new(callback)),
        };

        self.add_entry_raw(path, new_entry)
    }

    /// Adds a menu entry containing a custom UI callback at the given path
    pub fn add_entry_ui(
        &mut self,
        path: &[&str],
        location: u64,
        callback: impl Fn(&T, &mut egui::Ui) + Send + Sync + 'static,
    ) -> Result<(), InsertErr> {
        let name = if let Some(last) = path.last() {
            last.to_string()
        } else {
            return Err(InsertErr::EmptyPath);
        };

        let new_entry = MenuEntry {
            location,
            name,
            content: MenuContent::Ui(Box::new(callback)),
        };

        self.add_entry_raw(path, new_entry)
    }

    /// Adds a raw menu entry
    fn add_entry_raw(&mut self, path: &[&str], new_entry: MenuEntry<T>) -> Result<(), InsertErr> {
        Self::insert_recursive(&mut self.entries, path, new_entry).map_err(|mut e| {
            e.insert_path(path);
            e
        })?;

        Self::clean_menu(&mut self.entries);

        Ok(())
    }

    /// After menu entries have been changed, this function cleans up the list by removing empty menu's and
    /// sorting menu's by their location field
    fn clean_menu(entries: &mut Vec<MenuEntry<T>>) {
        for entry in entries.iter_mut() {
            entry.clean();
        }

        entries.retain(|entry| !entry.is_empty());
        entries.sort_by_key(|e| e.location);
    }

    /// Recursively travels down the menu list according to the path components given in `path` to insert the given entry
    fn insert_recursive(
        entries: &mut Vec<MenuEntry<T>>,
        path: &[&str],
        to_insert: MenuEntry<T>,
    ) -> Result<(), InsertErr> {
        assert!(!path.is_empty(), "Cannot insert at empty path");

        let head = path.first().unwrap();
        let tail = &path[1..];

        if tail.is_empty() {
            // We've reached our insertion point. Check that the entry doesn't already exist and then insert it
            if entries.iter().any(|entry| entry.name == *head) {
                return Err(InsertErr::AlreadyExists(String::new())); // We add the actual path at the top-level function
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
                return Err(InsertErr::NotASubMenu(String::new())); // We add the actual path at the top-level function
            };

            Self::insert_recursive(parent_entry_list, tail, to_insert)?;
        }

        Ok(())
    }

    /// Shows the menu inside the current UI. Note that this does not create a menu bar container. This must be done by the user
    pub fn show(&self, param: &T, ui: &mut egui::Ui) {
        let mut prev_location = None;

        for entry in &self.entries {
            entry.show(param, &mut prev_location, self.separator_interval, ui);
        }

        #[cfg(debug_assertions)]
        self.show_debug(param, ui);
    }

    /// Shows the editor debug menu
    #[cfg(debug_assertions)]
    fn show_debug(&self, param: &T, ui: &mut egui::Ui) {
        let Some(debug_ui_callback) = self.debug_menu.as_deref() else {
            return;
        };

        let new_layout = if ui.layout().is_horizontal() {
            egui::Layout::right_to_left(egui::Align::Center)
        } else {
            *ui.layout()
        };

        ui.scope_builder(egui::UiBuilder::new().layout(new_layout), |ui| {
            debug_ui_callback(param, ui);
        });
    }
}

impl<T> Default for Menu<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// An error while inserting a menu item
#[derive(Debug, Clone, derive_more::Error, derive_more::Display)]
pub enum InsertErr {
    /// Empty path
    #[display("Cannot insert entry at empty path")]
    EmptyPath,

    /// Item already exists
    #[display("Entry already exists: {}", _0)]
    AlreadyExists(#[error(not(source))] String),

    /// An insertion would replace a non-menu entry with a submenu
    #[display(
        "An entry along the menu path already exists as a non-menu entry: {}",
        _0
    )]
    NotASubMenu(#[error(not(source))] String),
}

impl InsertErr {
    /// If this error is one that contains a menu path, formats it and inserts it
    fn insert_path(&mut self, path: &[&str]) {
        match self {
            Self::EmptyPath => {}
            Self::AlreadyExists(p) => *p = format_menu_path(path),
            Self::NotASubMenu(p) => *p = format_menu_path(path),
        }
    }
}

/// A menu entry
#[derive(Debug)]
struct MenuEntry<T>
where
    T: ?Sized,
{
    /// The location relative to other entries. Lower is earlier
    location: u64,

    /// The name of this entry
    name: String,

    /// The contents of this entry
    content: MenuContent<T>,
}

/// Menu callback function type
type CallbackFn<T> = dyn Fn(&T) + Send + Sync;

/// Menu sub-ui callback funnction type
type SubUIFn<T> = dyn Fn(&T, &mut egui::Ui) + Send + Sync;

/// The contents of a menu entry
#[derive(derive_more::Debug)]
enum MenuContent<T>
where
    T: ?Sized,
{
    /// A submenu
    SubMenu(Vec<MenuEntry<T>>),

    /// A button that calls a callback
    #[debug("Callback")]
    Callback(Box<CallbackFn<T>>),

    /// A submenu that shows a custom UI
    #[debug("Sub-UI")]
    Ui(Box<SubUIFn<T>>),
}

impl<T> MenuEntry<T>
where
    T: ?Sized,
{
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
    fn show(
        &self,
        param: &T,
        prev_location: &mut Option<u64>,
        separator_interval: u64,
        ui: &mut egui::Ui,
    ) {
        if let Some(prev_location) = prev_location
            && (prev_location.saturating_add(separator_interval)) <= self.location
        {
            ui.separator();
        }

        *prev_location = Some(self.location);

        match &self.content {
            MenuContent::SubMenu(items) => {
                ui.menu_button(&self.name, |ui| {
                    let mut prev_location = None;

                    for item in items {
                        item.show(param, &mut prev_location, separator_interval, ui);
                    }
                });
            }
            MenuContent::Callback(cb) => {
                if ui.button(&self.name).clicked() {
                    cb(param);
                }
            }
            MenuContent::Ui(ui_cb) => {
                ui.menu_button(&self.name, |ui| ui_cb(param, ui));
            }
        }
    }
}

/// Formats a menu path for display
fn format_menu_path(path: &[&str]) -> String {
    path.join("/").clone()
}
