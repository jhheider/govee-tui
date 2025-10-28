use crossterm::event::{KeyCode, KeyModifiers};

use super::app::App;
use super::view_state::ViewMode;

impl App {
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match &self.state.view_mode {
            ViewMode::List => self.handle_list_keys(key, modifiers),
            ViewMode::Detail => self.handle_detail_keys(key, modifiers),
            ViewMode::Brightness => self.handle_brightness_keys(key, modifiers),
            ViewMode::ColorPicker => self.handle_color_picker_keys(key, modifiers),
            ViewMode::Search => self.handle_search_keys(key, modifiers),
            ViewMode::Help => self.handle_help_keys(key, modifiers),
        }
    }

    fn handle_list_keys(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match (key, modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (KeyCode::Char('r'), _) => {
                self.needs_refresh = true;
            }
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                self.state.enter_search();
            }
            (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::Char('k'), _) => {
                self.move_selection(-1);
            }
            (KeyCode::Down, KeyModifiers::NONE) | (KeyCode::Char('j'), _) => {
                self.move_selection(1);
            }
            (KeyCode::Enter, _) => {
                self.state.enter_detail_view();
            }
            (KeyCode::Char(' '), _) => {
                self.state.toggle_selection();
            }
            (KeyCode::Char('x'), _) => {
                self.state.clear_selections();
            }
            (KeyCode::Char('?'), _) => {
                self.state.enter_help();
            }
            _ => {}
        }
    }

    fn handle_detail_keys(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match (key, modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('q'), _) => {
                self.state.exit_to_list();
            }
            // Direct brightness control with arrows (10% default, 5% fine-grained)
            (KeyCode::Up, KeyModifiers::SHIFT) | (KeyCode::Char('k'), KeyModifiers::SHIFT) => {
                if let Some(state) = &self.state.device_state {
                    let current = state.brightness.unwrap_or(50);
                    let new_brightness = (current + 5).min(100);
                    self.request_apply_brightness(new_brightness);
                }
            }
            (KeyCode::Down, KeyModifiers::SHIFT) | (KeyCode::Char('j'), KeyModifiers::SHIFT) => {
                if let Some(state) = &self.state.device_state {
                    let current = state.brightness.unwrap_or(50);
                    let new_brightness = current.saturating_sub(5);
                    self.request_apply_brightness(new_brightness);
                }
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                if let Some(state) = &self.state.device_state {
                    let current = state.brightness.unwrap_or(50);
                    let new_brightness = (current + 10).min(100);
                    self.request_apply_brightness(new_brightness);
                }
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                if let Some(state) = &self.state.device_state {
                    let current = state.brightness.unwrap_or(50);
                    let new_brightness = current.saturating_sub(10);
                    self.request_apply_brightness(new_brightness);
                }
            }
            // Toggle power with Space
            (KeyCode::Char(' '), _) => {
                if let Some(state) = &self.state.device_state {
                    let new_power = !state.power;
                    self.request_toggle_power(new_power);
                }
            }
            // Advanced controls
            (KeyCode::Char('c'), _) => {
                let (r, g, b) = self
                    .state
                    .device_state
                    .as_ref()
                    .and_then(|s| s.color)
                    .map(|c| (c.r, c.g, c.b))
                    .unwrap_or((255, 255, 255));
                self.state.enter_color_picker(r, g, b);
            }
            _ => {}
        }
    }

    fn handle_brightness_keys(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if let Some(brightness) = &mut self.state.brightness_control {
            match (key, modifiers) {
                (KeyCode::Esc, _) => {
                    self.state.exit_to_detail();
                }
                (KeyCode::Enter, _) => {
                    // Apply will be handled async in main loop
                }
                (KeyCode::Up, KeyModifiers::SHIFT) | (KeyCode::Char('k'), KeyModifiers::SHIFT) => {
                    brightness.adjust(5);
                }
                (KeyCode::Down, KeyModifiers::SHIFT)
                | (KeyCode::Char('j'), KeyModifiers::SHIFT) => {
                    brightness.adjust(-5);
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    brightness.adjust(10);
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    brightness.adjust(-10);
                }
                (KeyCode::Char(c), _) if c.is_ascii_digit() => {
                    let digit = c.to_digit(10).unwrap() as u8;
                    if digit > 0 {
                        brightness.set(digit * 10);
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_color_picker_keys(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if let Some(picker) = &mut self.state.color_picker {
            match (key, modifiers) {
                (KeyCode::Esc, _) => {
                    self.state.exit_to_detail();
                }
                (KeyCode::Enter, _) => {
                    // Apply will be handled async in main loop
                }
                (KeyCode::Tab, _) => {
                    picker.next_channel();
                }
                (KeyCode::BackTab, _) => {
                    picker.prev_channel();
                }
                (KeyCode::Up, KeyModifiers::SHIFT) | (KeyCode::Char('k'), KeyModifiers::SHIFT) => {
                    picker.adjust(5);
                }
                (KeyCode::Down, KeyModifiers::SHIFT)
                | (KeyCode::Char('j'), KeyModifiers::SHIFT) => {
                    picker.adjust(-5);
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    picker.adjust(10);
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    picker.adjust(-10);
                }
                _ => {}
            }
        }
    }

    fn handle_search_keys(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Esc => {
                self.state.exit_to_list();
            }
            KeyCode::Char(c) => {
                self.state.search_query.push(c);
            }
            KeyCode::Backspace => {
                self.state.search_query.pop();
            }
            KeyCode::Enter => {
                // Filter devices and return to list
                self.state.exit_to_list();
            }
            _ => {}
        }
    }

    fn handle_help_keys(&mut self, _key: KeyCode, _modifiers: KeyModifiers) {
        // Any key exits help
        self.state.exit_to_list();
    }
}
