use anyhow::Result;

use crate::{api, config, db};

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
}

impl App {
    pub fn new(client: api::Client, db: db::Database, config: config::Config) -> Self {
        Self {
            client,
            db,
            config,
            theme: Theme::dark(),
            devices: Vec::new(),
            state: AppState::new(),
            should_quit: false,
            needs_refresh: false,
        }
    }

    pub async fn refresh_devices(&mut self) -> Result<()> {
        self.devices = self.client.get_devices().await?;

        for device in &self.devices {
            self.db.save_device(&device.id, &device.name, &device.model)?;
        }

        Ok(())
    }

    pub async fn load_device_state(&mut self) -> Result<()> {
        if self.devices.is_empty() || self.state.selected_index >= self.devices.len() {
            return Ok(());
        }

        let device = &self.devices[self.state.selected_index];
        match self.client.get_device_state(&device.id, &device.model).await {
            Ok(state) => {
                self.state.device_state = Some(state);
                Ok(())
            }
            Err(e) => {
                self.state.status_message = Some(format!("Failed to load state: {}", e));
                Ok(())
            }
        }
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
    }

    pub async fn apply_brightness(&mut self, value: u8) -> Result<()> {
        let indices = self.state.get_selected_or_current();

        for &idx in &indices {
            if let Some(device) = self.devices.get(idx) {
                let cmd = api::Command::brightness(value);
                self.client.control_device(&device.id, &device.model, cmd).await?;
            }
        }

        self.state.status_message = Some(format!("Brightness set to {}%", value));
        Ok(())
    }

    pub async fn apply_color(&mut self, r: u8, g: u8, b: u8) -> Result<()> {
        let indices = self.state.get_selected_or_current();

        for &idx in &indices {
            if let Some(device) = self.devices.get(idx) {
                let cmd = api::Command::color(r, g, b);
                self.client.control_device(&device.id, &device.model, cmd).await?;
            }
        }

        self.state.status_message = Some(format!("Color set to RGB({},{},{})", r, g, b));
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn toggle_power(&mut self) -> Result<()> {
        let indices = self.state.get_selected_or_current();
        let new_state = !self.state.device_state.as_ref().map(|s| s.power).unwrap_or(false);

        for &idx in &indices {
            if let Some(device) = self.devices.get(idx) {
                let cmd = api::Command::turn(new_state);
                self.client.control_device(&device.id, &device.model, cmd).await?;
            }
        }

        self.state.status_message = Some(format!("Power {}", if new_state { "ON" } else { "OFF" }));
        Ok(())
    }
}
