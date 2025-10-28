use tokio::sync::mpsc;

use crate::{api, config, db};

use super::async_ops::{AsyncCommand, AsyncResponse};
use super::theme::Theme;
use super::view_state::AppState;

pub struct App {
    pub client: api::Client,
    pub db: db::Database,
    #[allow(dead_code)]
    pub config: config::Config,
    pub theme: Theme,
    pub devices: Vec<api::Device>,
    pub state: AppState,
    pub should_quit: bool,
    pub needs_refresh: bool,
    pub cmd_tx: mpsc::UnboundedSender<AsyncCommand>,
    pub resp_rx: mpsc::UnboundedReceiver<AsyncResponse>,
    pub loading: bool,
}

impl App {
    pub fn new(client: api::Client, db: db::Database, config: config::Config) -> Self {
        let (cmd_tx, resp_rx) = super::async_ops::spawn_worker(client.clone());

        Self {
            client,
            db,
            config,
            theme: Theme::dark(),
            devices: Vec::new(),
            state: AppState::new(),
            should_quit: false,
            needs_refresh: false,
            cmd_tx,
            resp_rx,
            loading: false,
        }
    }

    pub fn request_refresh_devices(&mut self) {
        self.loading = true;
        let _ = self.cmd_tx.send(AsyncCommand::RefreshDevices);
    }

    pub fn request_load_device_state(&mut self) {
        if self.devices.is_empty() || self.state.selected_index >= self.devices.len() {
            return;
        }

        let device = &self.devices[self.state.selected_index];
        self.loading = true;
        let _ = self.cmd_tx.send(AsyncCommand::LoadDeviceState {
            device_id: device.id.clone(),
            model: device.model.clone(),
        });
    }

    pub fn selected_device(&self) -> Option<&api::Device> {
        self.devices.get(self.state.selected_index)
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.devices.len();
        if len == 0 {
            return;
        }

        let new_index =
            (self.state.selected_index as isize + delta).rem_euclid(len as isize) as usize;
        self.state.selected_index = new_index;

        // Clear device state when moving to a new device
        self.state.device_state = None;
    }

    pub fn request_apply_brightness(&mut self, value: u8) {
        if let Some(device) = self.selected_device() {
            let device_id = device.id.clone();
            let model = device.model.clone();
            self.loading = true;
            let _ = self.cmd_tx.send(AsyncCommand::ApplyBrightness {
                device_ids: vec![(device_id, model)],
                value,
            });
        }
    }

    pub fn request_apply_color(&mut self, r: u8, g: u8, b: u8) {
        if let Some(device) = self.selected_device() {
            let device_id = device.id.clone();
            let model = device.model.clone();
            self.loading = true;
            let _ = self.cmd_tx.send(AsyncCommand::ApplyColor {
                device_ids: vec![(device_id, model)],
                r,
                g,
                b,
            });
        }
    }

    pub fn request_toggle_power(&mut self, state: bool) {
        if let Some(device) = self.selected_device() {
            let device_id = device.id.clone();
            let model = device.model.clone();
            self.loading = true;
            let _ = self.cmd_tx.send(AsyncCommand::TogglePower {
                device_ids: vec![(device_id, model)],
                state,
            });
        }
    }

    pub fn handle_async_response(&mut self, response: AsyncResponse) {
        self.loading = false;

        match response {
            AsyncResponse::DevicesRefreshed(Ok(devices)) => {
                self.devices = devices;
                for device in &self.devices {
                    let _ = self.db.save_device(&device.id, &device.name, &device.model);
                }
                self.state.status_message =
                    Some(format!("Refreshed {} devices", self.devices.len()));
                self.state.error_message = None;
            }
            AsyncResponse::DevicesRefreshed(Err(e)) => {
                self.state.error_message = Some(format!("Failed to refresh devices: {e}"));
            }

            AsyncResponse::DeviceStateLoaded(Ok(state)) => {
                self.state.device_state = Some(state);
                self.state.error_message = None;
            }
            AsyncResponse::DeviceStateLoaded(Err(e)) => {
                self.state.error_message = Some(format!("Failed to load device state: {e}"));
            }

            AsyncResponse::AllDeviceStatesLoaded(_states) => {
                // Not used anymore
            }

            AsyncResponse::BrightnessApplied(Ok(value)) => {
                self.state.status_message = Some(format!("Brightness set to {value}%"));
                self.state.error_message = None;
                // Optimistically update local state for instant feedback
                if let Some(state) = &mut self.state.device_state {
                    state.brightness = Some(value);
                }
                // Don't refresh immediately - would overwrite optimistic update
                // User can press 'r' to manually refresh if needed
            }
            AsyncResponse::BrightnessApplied(Err(e)) => {
                self.state.error_message = Some(format!("Failed to set brightness: {e}"));
            }

            AsyncResponse::ColorApplied(Ok((r, g, b))) => {
                // Get closest color name
                let color_name = color_name::css::Color::similar([r, g, b]);
                self.state.status_message =
                    Some(format!("Color set to {color_name} RGB({r},{g},{b})"));
                self.state.error_message = None;
                // Optimistically update local state for instant feedback
                if let Some(state) = &mut self.state.device_state {
                    state.color = Some(crate::api::models::RgbColor::new(r, g, b));
                }
                // Don't refresh immediately - would overwrite optimistic update
            }
            AsyncResponse::ColorApplied(Err(e)) => {
                self.state.error_message = Some(format!("Failed to set color: {e}"));
            }

            AsyncResponse::PowerToggled(Ok(state)) => {
                self.state.status_message =
                    Some(format!("Power {}", if state { "ON" } else { "OFF" }));
                self.state.error_message = None;
                // Optimistically update local state for instant feedback
                if let Some(device_state) = &mut self.state.device_state {
                    device_state.power = state;
                }
                // Don't refresh immediately - would overwrite optimistic update
            }
            AsyncResponse::PowerToggled(Err(e)) => {
                self.state.error_message = Some(format!("Failed to toggle power: {e}"));
            }
        }
    }
}
