use super::focus::Focus;
use crate::api::models::DeviceState;
use crate::ui::widgets::color_picker::ColorPicker;
use crate::ui::widgets::scene_picker::ScenePicker;

/// Modal overlays that can appear on top of the main view
#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    None,
    Help,
    ColorPicker(ColorPicker),
    ScenePicker(ScenePicker),
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

    /// Status message (green; expired after a few seconds by App::tick)
    pub status_message: Option<String>,

    /// Error message (red, persists until the next successful action)
    pub error_message: Option<String>,
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

    /// Close any open modal
    pub fn close_modal(&mut self) {
        self.modal = Modal::None;
    }

    /// Check if a modal is currently open
    pub fn has_modal(&self) -> bool {
        self.modal != Modal::None
    }
}
