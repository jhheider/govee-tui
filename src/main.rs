use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod api;
mod config;
mod db;
mod ui;

#[derive(Parser)]
#[command(name = "govee-tui")]
#[command(about = "Clean, colorful TUI for controlling Govee smart devices", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive TUI
    Tui,

    /// List all devices
    List,

    /// Control a device
    Control {
        /// Device name or ID
        device: String,

        /// Command to execute
        #[command(subcommand)]
        command: ControlCommand,
    },
}

#[derive(Subcommand)]
enum ControlCommand {
    /// Turn device on/off
    Turn { state: String },

    /// Set brightness (0-100)
    Brightness { value: u8 },

    /// Set RGB color
    Color { r: u8, g: u8, b: u8 },

    /// Set color temperature (2000-9000K)
    Temp { kelvin: u16 },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose)?;
    info!("Starting govee-tui v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = config::Config::load(cli.config)?;

    // Initialize database
    let db = db::Database::new(&config.database.path)?;

    // Initialize API client
    let client = api::Client::new(&config.api.key)?;

    // Execute command
    match cli.command {
        None | Some(Commands::Tui) => {
            ui::run(client, db, config).await?;
        }
        Some(Commands::List) => {
            let devices = client.get_devices().await?;
            for device in devices {
                println!("{}: {} ({})", device.id, device.name, device.model);
            }
        }
        Some(Commands::Control { device, command }) => {
            execute_control(&client, &device, command).await?;
        }
    }

    Ok(())
}

async fn execute_control(
    client: &api::Client,
    device_id: &str,
    command: ControlCommand,
) -> Result<()> {
    use api::Command;

    let cmd = match command {
        ControlCommand::Turn { state } => Command::turn(state.to_lowercase() == "on"),
        ControlCommand::Brightness { value } => Command::brightness(value),
        ControlCommand::Color { r, g, b } => Command::color(r, g, b),
        ControlCommand::Temp { kelvin } => Command::temperature(kelvin),
    };

    // Note: For CLI, we need the model. For now, use empty string
    // In a real scenario, we'd look up the device first
    client.control_device(device_id, "", cmd).await?;
    info!("Command executed successfully");
    Ok(())
}

fn init_logging(verbose: bool) -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = if verbose {
        EnvFilter::new("govee_tui=debug,info")
    } else {
        EnvFilter::new("govee_tui=info")
    };

    tracing_subscriber::registry().with(filter).with(tracing_subscriber::fmt::layer()).init();

    Ok(())
}
