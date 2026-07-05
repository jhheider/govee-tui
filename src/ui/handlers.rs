use super::app::App;
use super::focus::Focus;
use super::view_state::Modal;
use crossterm::event::{KeyCode, KeyModifiers};

impl App {
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // Ctrl+C quits unconditionally, even with a modal open
        if key == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL {
            self.should_quit = true;
            return;
        }

        // Handle modals first
        if self.state.has_modal() {
            self.handle_modal_keys(key, modifiers);
            return;
        }

        // Global keys
        match (key, modifiers) {
            (KeyCode::Char('q'), _) => {
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
            KeyCode::Char(' ') => {
                self.toggle_power();
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

            // Reload state on demand (also makes the "press Enter to load"
            // hint true when this pane is focused)
            (KeyCode::Enter, _) => {
                self.request_load_device_state();
            }

            // Power control
            (KeyCode::Char(' '), _) => {
                self.toggle_power();
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

            // Color temperature - left is warmer, right is cooler;
            // SHIFT (or H/L) for fine adjustment
            (KeyCode::Left, KeyModifiers::SHIFT) | (KeyCode::Char('H'), _) => {
                self.adjust_color_temp(-100);
            }
            (KeyCode::Right, KeyModifiers::SHIFT) | (KeyCode::Char('L'), _) => {
                self.adjust_color_temp(100);
            }
            (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                self.adjust_color_temp(-500);
            }
            (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                self.adjust_color_temp(500);
            }

            // Scene picker
            (KeyCode::Char('s'), _) => {
                self.open_scene_picker();
            }

            // Color control
            (KeyCode::Char('c'), _) => {
                if !self.selected_device().is_some_and(|d| d.supports_color) {
                    self.set_status("This device does not support color".to_string());
                    return;
                }
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
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            match picker.mode {
                                ColorPickerMode::Rgb => picker.prev_channel(),
                                ColorPickerMode::Browser => picker.prev_color(),
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            match picker.mode {
                                ColorPickerMode::Rgb => picker.next_channel(),
                                ColorPickerMode::Browser => picker.next_color(),
                            }
                        }
                    }

                    // Left/Right behavior depends on mode
                    KeyCode::Left | KeyCode::Char('h') => {
                        if let Modal::ColorPicker(ref mut picker) = self.state.modal {
                            match picker.mode {
                                ColorPickerMode::Rgb => picker.adjust(-10),
                                ColorPickerMode::Browser => picker.prev_group(),
                            }
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
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
            Modal::ScenePicker(_) => match key {
                KeyCode::Esc => {
                    self.state.close_modal();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Modal::ScenePicker(ref mut picker) = self.state.modal {
                        picker.prev();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Modal::ScenePicker(ref mut picker) = self.state.modal {
                        picker.next();
                    }
                }
                KeyCode::PageUp => {
                    if let Modal::ScenePicker(ref mut picker) = self.state.modal {
                        picker.page_up();
                    }
                }
                KeyCode::PageDown => {
                    if let Modal::ScenePicker(ref mut picker) = self.state.modal {
                        picker.page_down();
                    }
                }
                KeyCode::Home => {
                    if let Modal::ScenePicker(ref mut picker) = self.state.modal {
                        picker.home();
                    }
                }
                KeyCode::End => {
                    if let Modal::ScenePicker(ref mut picker) = self.state.modal {
                        picker.end();
                    }
                }
                KeyCode::Enter => {
                    self.apply_selected_scene();
                }
                _ => {}
            },
            _ => {
                // Close other modals with Esc
                if matches!(key, KeyCode::Esc) {
                    self.state.close_modal();
                }
            }
        }
    }
}
