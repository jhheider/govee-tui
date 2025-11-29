use super::app::App;
use super::focus::Focus;
use super::view_state::Modal;
use crossterm::event::{KeyCode, KeyModifiers};

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
                    let new_power = !state.power;
                    self.request_toggle_power(new_power);
                }
            }

            // Brightness control - SHIFT for fine adjustment
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
                self.state.modal =
                    Modal::ColorPicker(crate::ui::widgets::color_picker::ColorPicker::new(r, g, b));
            }

            // Color temperature control
            (KeyCode::Char('t'), _) => {
                self.adjust_color_temp(-500); // Decrease (warmer)
            }
            (KeyCode::Char('T'), _) => {
                self.adjust_color_temp(500); // Increase (cooler)
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
                use crate::ui::widgets::color_picker::ColorPickerMode;

                match key {
                    KeyCode::Esc => {
                        self.state.close_modal();
                    }

                    // Tab toggles between RGB and Browser modes
                    KeyCode::Tab => {
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            picker.toggle_mode();
                        }
                    }

                    // Enter behavior depends on mode
                    KeyCode::Enter => {
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            // In browser mode, select the color first
                            if picker.mode == ColorPickerMode::Browser {
                                picker.select_current_color();
                            }

                            // Extract RGB values before borrowing self mutably
                            let (r, g, b) = (picker.r, picker.g, picker.b);

                            // Now we can borrow self mutably
                            self.request_apply_color(r, g, b);
                            self.state.close_modal();
                        }
                    }

                    // Up/Down behavior depends on mode
                    KeyCode::Up => {
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            match picker.mode {
                                ColorPickerMode::Rgb => picker.prev_channel(),
                                ColorPickerMode::Browser => picker.prev_color(),
                            }
                        }
                    }
                    KeyCode::Down => {
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            match picker.mode {
                                ColorPickerMode::Rgb => picker.next_channel(),
                                ColorPickerMode::Browser => picker.next_color(),
                            }
                        }
                    }

                    // Left/Right behavior depends on mode
                    KeyCode::Left => {
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            match picker.mode {
                                ColorPickerMode::Rgb => picker.adjust(-10),
                                ColorPickerMode::Browser => picker.prev_group(),
                            }
                        }
                    }
                    KeyCode::Right => {
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            match picker.mode {
                                ColorPickerMode::Rgb => picker.adjust(10),
                                ColorPickerMode::Browser => picker.next_group(),
                            }
                        }
                    }

                    _ => {}
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
            let current = state.brightness.unwrap_or(50) as i32;
            let new_brightness = (current + delta).clamp(0, 100) as u8;
            self.request_apply_brightness(new_brightness);
        }
    }

    fn adjust_color_temp(&mut self, delta: i32) {
        if let Some(state) = &self.state.device_state {
            let current = state.color_temp.unwrap_or(4000) as i32;
            let new_temp = (current + delta).clamp(2000, 9000) as u16;
            self.request_apply_color_temp(new_temp);
        }
    }
}
