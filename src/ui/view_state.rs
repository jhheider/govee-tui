use std::collections::HashSet;

use super::focus::Focus;
use crate::api::models::DeviceState;
use crate::ui::widgets::color_picker::ColorPicker;

/// Modal overlays that can appear on top of the main view
#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    None,
    Help,
    ColorPicker(ColorPicker),
    Search,
    Scenes,
}

/// Complete application UI state
pub struct AppState {
    /// Which pane has focus (List or Detail)
    pub focus: Focus,

    /// Currently selected device index
    pub selected_index: usize,

    /// Current device state (for detail pane)
    pub device_state: Option<DeviceState>,

    /// Active modal overlay
    pub modal: Modal,

    /// Status message (green, temporary)
    pub status_message: Option<String>,

    /// Error message (red, persistent until cleared)
    pub error_message: Option<String>,

    /// Selected device IDs for multi-device operations
    pub selected_devices: HashSet<String>,

    /// Search/filter query
    pub search_query: String,

    /// Whether search mode is active
    pub search_active: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            focus: Focus::List,
            selected_index: 0,
            device_state: None,
            modal: Modal::None,
            status_message: None,
            error_message: None,
            selected_devices: HashSet::new(),
            search_query: String::new(),
            search_active: false,
        }
    }

    /// Toggle focus between list and detail panes
    pub fn toggle_focus(&mut self) {
        self.focus = self.focus.toggle();
    }

    /// Open help modal
    pub fn open_help(&mut self) {
        self.modal = Modal::Help;
    }

    /// Open search modal
    pub fn open_search(&mut self) {
        self.modal = Modal::Search;
        self.search_active = true;
    }

    /// Open scenes modal
    pub fn open_scenes(&mut self) {
        self.modal = Modal::Scenes;
    }

    /// Close any open modal
    pub fn close_modal(&mut self) {
        self.modal = Modal::None;
    }

    /// Check if a modal is currently open
    pub fn has_modal(&self) -> bool {
        self.modal != Modal::None
    }

    /// Toggle device selection
    pub fn toggle_device_selection(&mut self, device_id: &str) {
        if self.selected_devices.contains(device_id) {
            self.selected_devices.remove(device_id);
        } else {
            self.selected_devices.insert(device_id.to_string());
        }
    }

    /// Check if a device is selected
    #[allow(dead_code)]
    pub fn is_device_selected(&self, device_id: &str) -> bool {
        self.selected_devices.contains(device_id)
    }

    /// Clear all selections
    pub fn clear_selections(&mut self) {
        self.selected_devices.clear();
    }

    /// Get selection count
    #[allow(dead_code)]
    pub fn selection_count(&self) -> usize {
        self.selected_devices.len()
    }
}
