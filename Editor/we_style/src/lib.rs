#![doc = include_str!("../README.md")]

use wutengine_egui::egui;

/// Color of the main menu bar
pub const MENU_COLOR: egui::Color32 = egui::Color32::from_gray(90);

/// Color of the panel tab bar
pub const PANEL_TAB_BAR_COLOR: egui::Color32 = egui::Color32::from_gray(35);

/// Color of the inactive panel tabs
pub const PANEL_TAB_INACTIVE_COLOR: egui::Color32 = egui::Color32::from_gray(45);

/// Color of the active panel tabs
pub const PANEL_TAB_ACTIVE_COLOR: egui::Color32 = egui::Color32::from_gray(70);

/// Background color of selected items
pub const SELECTION_BACKGROUND_COLOR: egui::Color32 = egui::Color32::from_gray(70);
