use crate::api::models::DeviceState;
use crate::ui::widgets::{brightness::BrightnessControl, color_picker::ColorPicker};

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    List,
    Detail,
    Brightness,
    ColorPicker,
    Search,
}

pub struct AppState {
    pub view_mode: ViewMode,
    pub selected_index: usize,
    pub selected_devices: Vec<usize>, // Multi-select indices
    pub search_query: String,
    pub device_state: Option<DeviceState>,
    pub brightness_control: Option<BrightnessControl>,
    pub color_picker: Option<ColorPicker>,
    pub status_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::List,
            selected_index: 0,
            selected_devices: Vec::new(),
            search_query: String::new(),
            device_state: None,
            brightness_control: None,
            color_picker: None,
            status_message: None,
        }
    }

    pub fn enter_detail_view(&mut self) {
        self.view_mode = ViewMode::Detail;
    }

    pub fn enter_brightness_control(&mut self, current: u8) {
        self.brightness_control = Some(BrightnessControl::new(current));
        self.view_mode = ViewMode::Brightness;
    }

    pub fn enter_color_picker(&mut self, r: u8, g: u8, b: u8) {
        self.color_picker = Some(ColorPicker::new(r, g, b));
        self.view_mode = ViewMode::ColorPicker;
    }

    pub fn enter_search(&mut self) {
        self.search_query.clear();
        self.view_mode = ViewMode::Search;
    }

    pub fn exit_to_list(&mut self) {
        self.view_mode = ViewMode::List;
        self.brightness_control = None;
        self.color_picker = None;
        self.search_query.clear();
    }

    pub fn exit_to_detail(&mut self) {
        self.view_mode = ViewMode::Detail;
        self.brightness_control = None;
        self.color_picker = None;
    }

    pub fn toggle_selection(&mut self) {
        if let Some(pos) = self.selected_devices.iter().position(|&i| i == self.selected_index) {
            self.selected_devices.remove(pos);
        } else {
            self.selected_devices.push(self.selected_index);
        }
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected_devices.contains(&index)
    }

    pub fn clear_selections(&mut self) {
        self.selected_devices.clear();
    }

    pub fn get_selected_or_current(&self) -> Vec<usize> {
        if self.selected_devices.is_empty() {
            vec![self.selected_index]
        } else {
            self.selected_devices.clone()
        }
    }
}
