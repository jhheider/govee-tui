use color_eyre::eyre::Result;
use tokio::sync::mpsc;

use crate::api::{Client, Command, Device, DeviceState, Scene};

/// Commands that can be sent to the background worker
#[derive(Debug, Clone)]
pub enum AsyncCommand {
    RefreshDevices,
    LoadDeviceState {
        device_id: String,
        model: String,
    },
    Control {
        device_id: String,
        model: String,
        command: Command,
    },
    LoadScenes {
        device_id: String,
        model: String,
    },
}

/// Responses from the background worker
#[derive(Debug)]
pub enum AsyncResponse {
    DevicesRefreshed(Result<Vec<Device>>),
    DeviceStateLoaded {
        device_id: String,
        result: Result<DeviceState>,
    },
    ControlApplied {
        device_id: String,
        command: Command,
        result: Result<()>,
    },
    ScenesLoaded {
        device_id: String,
        result: Result<Vec<Scene>>,
    },
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
                    AsyncResponse::DevicesRefreshed(client.get_devices().await)
                }

                AsyncCommand::LoadDeviceState { device_id, model } => {
                    let result = client.get_device_state(&device_id, &model).await;
                    AsyncResponse::DeviceStateLoaded { device_id, result }
                }

                AsyncCommand::Control {
                    device_id,
                    model,
                    command,
                } => {
                    let result = client
                        .control_device(&device_id, &model, command.clone())
                        .await;
                    AsyncResponse::ControlApplied {
                        device_id,
                        command,
                        result,
                    }
                }

                AsyncCommand::LoadScenes { device_id, model } => {
                    let result = client.get_scenes(&device_id, &model).await;
                    AsyncResponse::ScenesLoaded { device_id, result }
                }
            };

            // Send response back (ignore if receiver dropped)
            let _ = resp_tx.send(response);
        }
    });

    (cmd_tx, resp_rx)
}
