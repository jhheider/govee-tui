use crossterm::event::{KeyCode, KeyModifiers};
use super::app::App;
use super::focus::Focus;
use super::view_state_v2::Modal;

impl App {
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // Handle modals first
        if self.state.has_modal() {
            self.handle_modal_keys(key, modifiers);
            return;
        }

        // Global keys
        match (key, modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return;
            }
            (KeyCode::Char('?'), _) => {
                self.state.open_help();
                return;
            }
            (KeyCode::Char('r'), _) => {
                self.needs_refresh = true;
                return;
            }
            (KeyCode::Tab, _) => {
                self.state.toggle_focus();
                // Load state when focusing detail pane
                if self.state.focus == Focus::Detail && self.state.device_state.is_none() {
                    self.request_load_device_state();
                }
                return;
            }
            _ => {}
        }

        // Focus-specific keys
        match self.state.focus {
            Focus::List => self.handle_list_focus(key, modifiers),
            Focus::Detail => self.handle_detail_focus(key, modifiers),
        }
    }

    fn handle_list_focus(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection(-1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection(1);
            }
            KeyCode::Enter => {
                self.state.focus = Focus::Detail;
                self.request_load_device_state();
            }
            _ => {}
        }
    }

    fn handle_detail_focus(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match (key, modifiers) {
            (KeyCode::Esc, _) => {
                self.state.focus = Focus::List;
            }

            // Power control
            (KeyCode::Char(' '), _) => {
                if let Some(state) = &self.state.device_state {
                    let new_power = !state.power_state.map(|p| p.is_on()).unwrap_or(false);
                    self.request_toggle_power(new_power);
                }
            }

            // Brightness control
            (KeyCode::Up, KeyModifiers::SHIFT) | (KeyCode::Char('K'), _) => {
                self.adjust_brightness(5);
            }
            (KeyCode::Down, KeyModifiers::SHIFT) | (KeyCode::Char('J'), _) => {
                self.adjust_brightness(-5);
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                self.adjust_brightness(10);
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                self.adjust_brightness(-10);
            }

            // Color control
            (KeyCode::Char('c'), _) => {
                let (r, g, b) = self
                    .state
                    .device_state
                    .as_ref()
                    .and_then(|s| s.color)
                    .map(|c| (c.r, c.g, c.b))
                    .unwrap_or((255, 255, 255));
                self.state.modal = Modal::ColorPicker(
                    crate::ui::widgets::color_picker::ColorPicker::new(r, g, b)
                );
            }

            _ => {}
        }
    }

    fn handle_modal_keys(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match &self.state.modal {
            Modal::Help => {
                // Any key closes help
                self.state.close_modal();
            }
            Modal::ColorPicker(_) => {
                match key {
                    KeyCode::Esc => {
                        self.state.close_modal();
                    }
                    KeyCode::Enter => {
                        if let Modal::ColorPicker(picker) = &self.state.modal {
                            self.request_apply_color(picker.r, picker.g, picker.b);
                        }
                        self.state.close_modal();
                    }
                    _ => {
                        // Handle color picker navigation
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            match key {
                                KeyCode::Tab => picker.next_channel(),
                                KeyCode::BackTab => picker.prev_channel(),
                                KeyCode::Up => picker.adjust(10),
                                KeyCode::Down => picker.adjust(-10),
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {
                // Close other modals with Esc
                if matches!(key, KeyCode::Esc) {
                    self.state.close_modal();
                }
            }
        }
    }

    fn adjust_brightness(&mut self, delta: i32) {
        if let Some(state) = &self.state.device_state {
            let current = state.brightness.unwrap_or(50);
            let new_brightness = (current + delta).clamp(0, 100) as u8;
            self.request_apply_brightness(new_brightness);
        }
    }
}
