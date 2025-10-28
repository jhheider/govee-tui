use anyhow::Result;
use tokio::sync::mpsc;

use crate::api::{Client, Command, Device, DeviceState};

/// Commands that can be sent to the background worker
#[derive(Debug, Clone)]
pub enum AsyncCommand {
    RefreshDevices,
    LoadDeviceState {
        device_id: String,
        model: String,
    },
    LoadAllDeviceStates {
        devices: Vec<(String, String)>, // (id, model) pairs
    },
    ApplyBrightness {
        device_ids: Vec<(String, String)>,
        value: u8,
    },
    ApplyColor {
        device_ids: Vec<(String, String)>,
        r: u8,
        g: u8,
        b: u8,
    },
    TogglePower {
        device_ids: Vec<(String, String)>,
        state: bool,
    },
}

/// Responses from the background worker
#[derive(Debug)]
pub enum AsyncResponse {
    DevicesRefreshed(Result<Vec<Device>>),
    DeviceStateLoaded(Result<DeviceState>),
    AllDeviceStatesLoaded(Vec<Option<DeviceState>>),
    BrightnessApplied(Result<u8>),
    ColorApplied(Result<(u8, u8, u8)>),
    PowerToggled(Result<bool>),
}

/// Spawns a background worker task that processes API commands
pub fn spawn_worker(
    client: Client,
) -> (
    mpsc::UnboundedSender<AsyncCommand>,
    mpsc::UnboundedReceiver<AsyncResponse>,
) {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<AsyncCommand>();
    let (resp_tx, resp_rx) = mpsc::unbounded_channel::<AsyncResponse>();

    tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            let response = match cmd {
                AsyncCommand::RefreshDevices => {
                    let result = client.get_devices().await;
                    AsyncResponse::DevicesRefreshed(result)
                }

                AsyncCommand::LoadDeviceState { device_id, model } => {
                    let result = client.get_device_state(&device_id, &model).await;
                    AsyncResponse::DeviceStateLoaded(result)
                }

                AsyncCommand::LoadAllDeviceStates { devices } => {
                    // Load states for all devices in parallel
                    let futures: Vec<_> = devices
                        .iter()
                        .map(|(id, model)| client.get_device_state(id, model))
                        .collect();

                    let results = futures::future::join_all(futures).await;
                    let states: Vec<Option<DeviceState>> = results
                        .into_iter()
                        .map(|r| r.ok())
                        .collect();

                    AsyncResponse::AllDeviceStatesLoaded(states)
                }

                AsyncCommand::ApplyBrightness { device_ids, value } => {
                    let mut success = true;
                    // Run in parallel for all devices
                    let futures: Vec<_> = device_ids
                        .iter()
                        .map(|(id, model)| {
                            let cmd = Command::brightness(value);
                            client.control_device(id, model, cmd)
                        })
                        .collect();

                    for result in futures::future::join_all(futures).await {
                        if result.is_err() {
                            success = false;
                            break;
                        }
                    }

                    let result = if success {
                        Ok(value)
                    } else {
                        Err(anyhow::anyhow!("Failed"))
                    };
                    AsyncResponse::BrightnessApplied(result)
                }

                AsyncCommand::ApplyColor {
                    device_ids,
                    r,
                    g,
                    b,
                } => {
                    let mut success = true;
                    let futures: Vec<_> = device_ids
                        .iter()
                        .map(|(id, model)| {
                            let cmd = Command::color(r, g, b);
                            client.control_device(id, model, cmd)
                        })
                        .collect();

                    for result in futures::future::join_all(futures).await {
                        if result.is_err() {
                            success = false;
                            break;
                        }
                    }

                    let result = if success {
                        Ok((r, g, b))
                    } else {
                        Err(anyhow::anyhow!("Failed"))
                    };
                    AsyncResponse::ColorApplied(result)
                }

                AsyncCommand::TogglePower { device_ids, state } => {
                    let mut success = true;
                    let futures: Vec<_> = device_ids
                        .iter()
                        .map(|(id, model)| {
                            let cmd = Command::turn(state);
                            client.control_device(id, model, cmd)
                        })
                        .collect();

                    for result in futures::future::join_all(futures).await {
                        if result.is_err() {
                            success = false;
                            break;
                        }
                    }

                    let result = if success {
                        Ok(state)
                    } else {
                        Err(anyhow::anyhow!("Failed"))
                    };
                    AsyncResponse::PowerToggled(result)
                }
            };

            // Send response back (ignore if receiver dropped)
            let _ = resp_tx.send(response);
        }
    });

    (cmd_tx, resp_rx)
}
