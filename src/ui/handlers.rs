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
            (KeyCode::Char('/'), _) | (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                self.state.open_search();
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
            (KeyCode::Esc, _) => {
                // Clear search filter if active
                if !self.state.search_query.is_empty() {
                    self.state.search_query.clear();
                    self.state.search_active = false;
                    return;
                }
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
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection(-1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection(1);
            }
            // Vim-style jump to top/bottom
            KeyCode::Char('g') => {
                self.state.selected_index = 0;
                self.state.device_state = None;
            }
            KeyCode::Char('G') => {
                if !self.devices.is_empty() {
                    self.state.selected_index = self.devices.len() - 1;
                    self.state.device_state = None;
                }
            }
            // Enter detail view
            KeyCode::Enter => {
                self.state.focus = Focus::Detail;
                self.request_load_device_state();
            }
            // Quick power toggle from list
            KeyCode::Char(' ') => {
                self.quick_toggle_power();
            }
            // Multi-select
            KeyCode::Char('x') => {
                if let Some(device) = self.selected_device() {
                    let id = device.id.clone();
                    self.state.toggle_device_selection(&id);
                }
            }
            KeyCode::Char('a') => {
                // Select all devices
                for device in &self.devices {
                    self.state.selected_devices.insert(device.id.clone());
                }
            }
            KeyCode::Char('A') => {
                // Deselect all
                self.state.clear_selections();
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

            // Number keys for quick brightness (1-9 = 10-90%, 0 = 100%)
            (KeyCode::Char('1'), _) => self.set_brightness(10),
            (KeyCode::Char('2'), _) => self.set_brightness(20),
            (KeyCode::Char('3'), _) => self.set_brightness(30),
            (KeyCode::Char('4'), _) => self.set_brightness(40),
            (KeyCode::Char('5'), _) => self.set_brightness(50),
            (KeyCode::Char('6'), _) => self.set_brightness(60),
            (KeyCode::Char('7'), _) => self.set_brightness(70),
            (KeyCode::Char('8'), _) => self.set_brightness(80),
            (KeyCode::Char('9'), _) => self.set_brightness(90),
            (KeyCode::Char('0'), _) => self.set_brightness(100),

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

            // Scenes
            (KeyCode::Char('s'), _) => {
                self.state.open_scenes();
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
            Modal::Search => {
                match key {
                    KeyCode::Esc => {
                        self.state.close_modal();
                        // Keep search query active if user pressed Esc
                    }
                    KeyCode::Enter => {
                        self.state.close_modal();
                        // Keep search active with current query
                    }
                    KeyCode::Backspace => {
                        self.state.search_query.pop();
                    }
                    KeyCode::Char(c) => {
                        self.state.search_query.push(c);
                    }
                    _ => {}
                }
            }
            Modal::Scenes => {
                if matches!(key, KeyCode::Esc) {
                    self.state.close_modal();
                }
                // TODO: Handle scene selection
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
            Modal::None => {}
        }
    }

    fn adjust_brightness(&mut self, delta: i32) {
        if let Some(state) = &self.state.device_state {
            let current = state.brightness.unwrap_or(50) as i32;
            let new_brightness = (current + delta).clamp(0, 100) as u8;
            self.request_apply_brightness(new_brightness);
        }
    }

    fn set_brightness(&mut self, value: u8) {
        self.request_apply_brightness(value);
    }

    fn adjust_color_temp(&mut self, delta: i32) {
        if let Some(state) = &self.state.device_state {
            let current = state.color_temp.unwrap_or(4000) as i32;
            let new_temp = (current + delta).clamp(2000, 9000) as u16;
            self.request_apply_color_temp(new_temp);
        }
    }

    fn quick_toggle_power(&mut self) {
        // For quick toggle from list, we need to load state first or assume toggle
        // For now, just turn on if we don't know, or toggle if we do
        if let Some(device) = self.selected_device() {
            // We don't have state, so just turn on (most common use case)
            // User can go to detail view for more control
            let device_id = device.id.clone();
            let model = device.model.clone();

            // Check if we have any selected devices
            if self.state.selected_devices.is_empty() {
                // Just toggle the current device - assume ON since we don't know state
                self.loading = true;
                let _ = self.cmd_tx.send(super::async_ops::AsyncCommand::TogglePower {
                    device_ids: vec![(device_id, model)],
                    state: true, // Default to turning on
                });
                self.state.status_message = Some("Toggling power...".to_string());
            } else {
                // Toggle all selected devices
                let device_ids: Vec<(String, String)> = self
                    .devices
                    .iter()
                    .filter(|d| self.state.selected_devices.contains(&d.id))
                    .map(|d| (d.id.clone(), d.model.clone()))
                    .collect();

                self.loading = true;
                let _ = self.cmd_tx.send(super::async_ops::AsyncCommand::TogglePower {
                    device_ids,
                    state: true,
                });
                self.state.status_message = Some(format!(
                    "Toggling {} devices...",
                    self.state.selected_devices.len()
                ));
            }
        }
    }
}
