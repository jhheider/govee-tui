use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use crate::{api, cache};

use super::async_ops::{AsyncCommand, AsyncResponse};
use super::theme::Theme;
use super::view_state::AppState;
use crate::api::{Command, DeviceState};

/// How long after the last keypress a debounced control is sent
const DEBOUNCE: Duration = Duration::from_millis(300);
/// How long status messages stay visible
const STATUS_TTL: Duration = Duration::from_secs(4);

/// A control change waiting for the debounce window to close
struct PendingControl {
    device_id: String,
    model: String,
    command: Command,
    deadline: Instant,
}

pub struct App {
    pub theme: Theme,
    pub devices: Vec<api::Device>,
    /// Last confirmed state per device id (from state loads and control acks)
    pub known_states: HashMap<String, DeviceState>,
    /// Scene lists per device id (scene catalogs rarely change; fetched once)
    pub known_scenes: HashMap<String, Vec<api::Scene>>,
    pub state: AppState,
    pub should_quit: bool,
    pub needs_refresh: bool,
    pub cmd_tx: mpsc::UnboundedSender<AsyncCommand>,
    pub resp_rx: mpsc::UnboundedReceiver<AsyncResponse>,
    /// Device-list refresh in flight
    pub loading: bool,
    /// Device-state fetch in flight
    pub state_loading: bool,
    controls_inflight: u32,
    pending_brightness: Option<PendingControl>,
    pending_temp: Option<PendingControl>,
    pending_power: Option<PendingControl>,
    status_deadline: Option<Instant>,
}

impl App {
    pub fn new(client: api::Client) -> Self {
        let (cmd_tx, resp_rx) = super::async_ops::spawn_worker(client);

        // Paint the last-seen device list immediately; the first refresh
        // replaces it as soon as the API answers.
        let devices = cache::load_devices().unwrap_or_default();

        let mut app = Self {
            theme: Theme::dark(),
            devices,
            known_states: HashMap::new(),
            known_scenes: HashMap::new(),
            state: AppState::new(),
            should_quit: false,
            needs_refresh: false,
            cmd_tx,
            resp_rx,
            loading: false,
            state_loading: false,
            controls_inflight: 0,
            pending_brightness: None,
            pending_temp: None,
            pending_power: None,
            status_deadline: None,
        };
        if !app.devices.is_empty() {
            app.set_status("Showing cached devices, refreshing…".to_string());
        }
        app
    }

    pub fn set_status(&mut self, msg: String) {
        self.state.status_message = Some(msg);
        self.status_deadline = Some(Instant::now() + STATUS_TTL);
    }

    /// Per-frame housekeeping: expire status messages, flush debounced controls
    pub fn tick(&mut self) {
        if let Some(deadline) = self.status_deadline {
            if Instant::now() >= deadline {
                self.state.status_message = None;
                self.status_deadline = None;
            }
        }
        self.flush_pending();
    }

    fn flush_pending(&mut self) {
        // One control on the wire at a time keeps us inside Govee's
        // per-device rate limit even under key auto-repeat.
        if self.controls_inflight > 0 {
            return;
        }
        let now = Instant::now();
        for slot in [
            &mut self.pending_brightness,
            &mut self.pending_temp,
            &mut self.pending_power,
        ] {
            if slot.as_ref().is_some_and(|p| now >= p.deadline) {
                let p = slot.take().unwrap();
                self.controls_inflight += 1;
                let _ = self.cmd_tx.send(AsyncCommand::Control {
                    device_id: p.device_id,
                    model: p.model,
                    command: p.command,
                });
                break; // the other slot flushes once this one acks
            }
        }
    }

    pub fn request_refresh_devices(&mut self) {
        self.loading = true;
        let _ = self.cmd_tx.send(AsyncCommand::RefreshDevices);
    }

    pub fn request_load_device_state(&mut self) {
        if self.state_loading {
            return;
        }
        let Some(device) = self.selected_device() else {
            return;
        };
        let device_id = device.id.clone();
        let model = device.model.clone();
        self.state_loading = true;
        let _ = self
            .cmd_tx
            .send(AsyncCommand::LoadDeviceState { device_id, model });
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

        // Show the last confirmed state for the newly selected device, if any
        self.state.device_state = self
            .devices
            .get(new_index)
            .and_then(|d| self.known_states.get(&d.id))
            .cloned();
    }

    /// Send a control immediately (power, color - discrete actions)
    pub fn send_control_now(&mut self, command: Command) {
        if let Some(device) = self.selected_device() {
            let device_id = device.id.clone();
            let model = device.model.clone();
            self.controls_inflight += 1;
            let _ = self.cmd_tx.send(AsyncCommand::Control {
                device_id,
                model,
                command,
            });
        }
    }

    /// Debounced control: update the pending slot; sent after the debounce
    /// window closes (and any in-flight control acks)
    fn schedule_control(&mut self, command: Command) {
        let Some(device) = self.selected_device() else {
            return;
        };
        let pending = PendingControl {
            device_id: device.id.clone(),
            model: device.model.clone(),
            deadline: Instant::now() + DEBOUNCE,
            command: command.clone(),
        };
        match command {
            Command::Brightness(_) => self.pending_brightness = Some(pending),
            Command::ColorTemp(_) => self.pending_temp = Some(pending),
            Command::TurnOn | Command::TurnOff => self.pending_power = Some(pending),
            _ => unreachable!("only brightness/temp/power are debounced"),
        }
    }

    /// Optimistically adjust brightness locally; the API call follows debounced
    pub fn adjust_brightness(&mut self, delta: i32) {
        let Some(state) = &mut self.state.device_state else {
            return;
        };
        let current = state.brightness.unwrap_or(50) as i32;
        let new = (current + delta).clamp(0, 100) as u8;
        state.brightness = Some(new);
        self.schedule_control(Command::brightness(new));
    }

    /// Optimistically adjust color temperature locally; API call debounced
    pub fn adjust_color_temp(&mut self, delta: i32) {
        if !self
            .selected_device()
            .is_some_and(|d| d.supports_color_temp)
        {
            return;
        }
        let Some(state) = &mut self.state.device_state else {
            return;
        };
        let current = state.color_temp.unwrap_or(4000) as i32;
        let new = (current + delta).clamp(2000, 9000) as u16;
        state.color_temp = Some(new);
        self.schedule_control(Command::temperature(new));
    }

    /// Toggle power for the selected device, optimistically
    pub fn toggle_power(&mut self) {
        let Some(current) = self
            .state
            .device_state
            .as_ref()
            .map(|s| s.power)
            .or_else(|| {
                self.selected_device()
                    .and_then(|d| self.known_states.get(&d.id))
                    .map(|s| s.power)
            })
        else {
            // No known state yet - fetch it rather than guessing
            self.request_load_device_state();
            self.set_status("Power state unknown - loading, press Space again".to_string());
            return;
        };
        let new_power = !current;
        self.apply_power_locally(new_power);
        // Send the first toggle immediately for responsiveness, but if a control
        // is already on the wire (or a power send is queued), coalesce into the
        // debounced slot so held-Space auto-repeat collapses to a single send
        // instead of flooding the device with on/off flips.
        if self.controls_inflight == 0 && self.pending_power.is_none() {
            self.send_control_now(Command::turn(new_power));
        } else {
            self.schedule_control(Command::turn(new_power));
        }
    }

    fn apply_power_locally(&mut self, power: bool) {
        if let Some(state) = &mut self.state.device_state {
            state.power = power;
        }
        if let Some(id) = self.selected_device().map(|d| d.id.clone()) {
            if let Some(known) = self.known_states.get_mut(&id) {
                known.power = power;
            }
        }
    }

    pub fn request_apply_color(&mut self, r: u8, g: u8, b: u8) {
        if let Some(state) = &mut self.state.device_state {
            state.color = Some(crate::api::models::RgbColor::new(r, g, b));
        }
        self.send_control_now(Command::color(r, g, b));
    }

    /// Open the scene picker for the selected device, fetching the scene
    /// list on first use (cached afterwards)
    pub fn open_scene_picker(&mut self) {
        use crate::ui::widgets::scene_picker::ScenePicker;

        let Some(device) = self.selected_device() else {
            return;
        };
        if !device.supports_scenes {
            self.set_status("This device does not support scenes".to_string());
            return;
        }
        let device_id = device.id.clone();
        let model = device.model.clone();
        let name = device.name.clone();

        if let Some(scenes) = self.known_scenes.get(&device_id) {
            self.state.modal = super::view_state::Modal::ScenePicker(ScenePicker::with_scenes(
                name,
                scenes.clone(),
            ));
        } else {
            self.state.modal = super::view_state::Modal::ScenePicker(ScenePicker::loading(name));
            let _ = self
                .cmd_tx
                .send(AsyncCommand::LoadScenes { device_id, model });
        }
    }

    pub fn apply_selected_scene(&mut self) {
        let scene = match &self.state.modal {
            super::view_state::Modal::ScenePicker(picker) => picker.selected_scene().cloned(),
            _ => None,
        };
        if let Some(scene) = scene {
            self.send_control_now(Command::Scene(scene));
            self.state.close_modal();
        }
    }

    pub fn handle_async_response(&mut self, response: AsyncResponse) {
        match response {
            AsyncResponse::DevicesRefreshed(Ok(devices)) => {
                self.loading = false;
                self.devices = devices;
                if let Err(e) = cache::save_devices(&self.devices) {
                    tracing::warn!("Failed to write device cache: {e:#}");
                }
                if self.state.selected_index >= self.devices.len() {
                    self.state.selected_index = self.devices.len().saturating_sub(1);
                }
                self.set_status(format!("Refreshed {} devices", self.devices.len()));
                self.state.error_message = None;
            }
            AsyncResponse::DevicesRefreshed(Err(e)) => {
                self.loading = false;
                self.state.error_message = Some(format!("Failed to refresh devices: {e:#}"));
            }

            AsyncResponse::DeviceStateLoaded { device_id, result } => {
                self.state_loading = false;
                match result {
                    Ok(state) => {
                        let selected = self.selected_device().is_some_and(|d| d.id == device_id);
                        // Don't clobber optimistic local edits still in flight
                        if selected
                            && self.pending_brightness.is_none()
                            && self.pending_temp.is_none()
                            && self.pending_power.is_none()
                        {
                            self.state.device_state = Some(state.clone());
                        }
                        self.known_states.insert(device_id, state);
                        self.state.error_message = None;
                    }
                    Err(e) => {
                        self.state.error_message =
                            Some(format!("Failed to load device state: {e:#}"));
                    }
                }
            }

            AsyncResponse::ControlApplied {
                device_id,
                command,
                result,
            } => {
                self.controls_inflight = self.controls_inflight.saturating_sub(1);
                match result {
                    Ok(()) => {
                        if let Some(known) = self.known_states.get_mut(&device_id) {
                            apply_command(known, &command);
                        }
                        self.set_status(describe_success(&command));
                        self.state.error_message = None;
                    }
                    Err(e) => {
                        // Roll the optimistic local value back to the last
                        // confirmed state, so the UI never keeps showing a value
                        // the device did not accept (the error banner alone is
                        // transient and gets cleared by the next refresh).
                        let selected = self.selected_device().is_some_and(|d| d.id == device_id);
                        match command {
                            Command::TurnOn | Command::TurnOff => {
                                let attempted = matches!(command, Command::TurnOn);
                                if selected {
                                    if let Some(state) = &mut self.state.device_state {
                                        state.power = !attempted;
                                    }
                                }
                                if let Some(known) = self.known_states.get_mut(&device_id) {
                                    known.power = !attempted;
                                }
                            }
                            _ => {
                                // brightness/temp/color are written to known_states
                                // only on success, so it holds the last confirmed
                                // value: restore the failed field from it.
                                if selected {
                                    if let Some(known) = self.known_states.get(&device_id).cloned()
                                    {
                                        if let Some(state) = &mut self.state.device_state {
                                            revert_command(state, &known, &command);
                                        }
                                    }
                                }
                            }
                        }
                        self.state.error_message =
                            Some(format!("{}: {e:#}", describe_failure(&command)));
                    }
                }
            }

            AsyncResponse::ScenesLoaded { device_id, result } => {
                use super::view_state::Modal;
                use crate::ui::widgets::scene_picker::ScenePicker;

                match result {
                    Ok(scenes) => {
                        // Populate the picker if it's still open for this device
                        let selected = self.selected_device().is_some_and(|d| d.id == device_id);
                        if selected {
                            if let Modal::ScenePicker(picker) = &self.state.modal {
                                if picker.scenes.is_none() {
                                    self.state.modal =
                                        Modal::ScenePicker(ScenePicker::with_scenes(
                                            picker.device_name.clone(),
                                            scenes.clone(),
                                        ));
                                }
                            }
                        }
                        self.known_scenes.insert(device_id, scenes);
                        self.state.error_message = None;
                    }
                    Err(e) => {
                        // Close a still-loading picker rather than spinning forever
                        if let Modal::ScenePicker(picker) = &self.state.modal {
                            if picker.scenes.is_none() {
                                self.state.close_modal();
                            }
                        }
                        self.state.error_message = Some(format!("Failed to load scenes: {e:#}"));
                    }
                }
            }
        }
    }
}

fn apply_command(state: &mut DeviceState, command: &Command) {
    match command {
        Command::TurnOn => state.power = true,
        Command::TurnOff => state.power = false,
        Command::Brightness(v) => state.brightness = Some(*v),
        Command::Color(r, g, b) => {
            state.color = Some(crate::api::models::RgbColor::new(*r, *g, *b))
        }
        Command::ColorTemp(k) => state.color_temp = Some(*k),
        // Scenes change color/brightness in device-defined ways we can't
        // predict; the next state load reflects them
        Command::Scene(_) => {}
    }
}

/// Roll back the single field a failed command optimistically changed, using the
/// last confirmed state. Power is handled separately (its known state is written
/// optimistically); this covers the non-power controls.
fn revert_command(state: &mut DeviceState, known: &DeviceState, command: &Command) {
    match command {
        Command::Brightness(_) => state.brightness = known.brightness,
        Command::Color(..) => state.color = known.color,
        Command::ColorTemp(_) => state.color_temp = known.color_temp,
        Command::TurnOn | Command::TurnOff | Command::Scene(_) => {}
    }
}

fn describe_success(command: &Command) -> String {
    match command {
        Command::TurnOn => "Power ON".to_string(),
        Command::TurnOff => "Power OFF".to_string(),
        Command::Brightness(v) => format!("Brightness set to {v}%"),
        Command::Color(r, g, b) => {
            let name = color_name::css::Color::similar([*r, *g, *b]);
            format!("Color set to {name} RGB({r},{g},{b})")
        }
        Command::ColorTemp(k) => format!("Color temperature set to {k}K"),
        Command::Scene(scene) => format!("Scene set to {}", scene.name),
    }
}

fn describe_failure(command: &Command) -> &'static str {
    match command {
        Command::TurnOn | Command::TurnOff => "Failed to toggle power",
        Command::Brightness(_) => "Failed to set brightness",
        Command::Color(..) => "Failed to set color",
        Command::ColorTemp(_) => "Failed to set color temperature",
        Command::Scene(_) => "Failed to set scene",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::{DeviceState, RgbColor};

    fn state(
        power: bool,
        brightness: Option<u8>,
        color: Option<RgbColor>,
        color_temp: Option<u16>,
    ) -> DeviceState {
        DeviceState {
            power,
            brightness,
            color,
            color_temp,
        }
    }

    #[test]
    fn revert_restores_confirmed_brightness() {
        let known = state(true, Some(40), None, None);
        let mut cur = state(true, Some(90), None, None); // optimistic, unaccepted
        revert_command(&mut cur, &known, &Command::brightness(90));
        assert_eq!(cur.brightness, Some(40));
    }

    #[test]
    fn revert_restores_confirmed_color() {
        let known = state(true, None, Some(RgbColor::new(1, 2, 3)), None);
        let mut cur = state(true, None, Some(RgbColor::new(250, 0, 0)), None);
        revert_command(&mut cur, &known, &Command::color(250, 0, 0));
        assert_eq!(cur.color.map(|c| (c.r, c.g, c.b)), Some((1, 2, 3)));
    }

    #[test]
    fn revert_restores_confirmed_color_temp() {
        let known = state(true, None, None, Some(3000));
        let mut cur = state(true, None, None, Some(6500));
        revert_command(&mut cur, &known, &Command::temperature(6500));
        assert_eq!(cur.color_temp, Some(3000));
    }

    #[test]
    fn revert_only_touches_the_failed_field() {
        let known = state(true, Some(40), Some(RgbColor::new(1, 1, 1)), Some(3000));
        let mut cur = state(false, Some(90), Some(RgbColor::new(9, 9, 9)), Some(9000));
        revert_command(&mut cur, &known, &Command::brightness(90));
        assert_eq!(cur.brightness, Some(40)); // reverted
        assert!(!cur.power); // untouched
        assert_eq!(cur.color.map(|c| c.r), Some(9)); // untouched
        assert_eq!(cur.color_temp, Some(9000)); // untouched
    }

    #[test]
    fn revert_is_noop_for_power_and_scene() {
        // Power reverts via its own path (known state is optimistic); scenes
        // have no single field to restore. revert_command leaves fields alone.
        let known = state(true, Some(40), None, None);
        let mut cur = state(false, Some(90), None, None);
        revert_command(&mut cur, &known, &Command::turn(false));
        assert_eq!(cur.brightness, Some(90));
        assert!(!cur.power);
    }
}
