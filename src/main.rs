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

    // Initialize logging (only to file in non-verbose mode)
    init_logging(cli.verbose)?;

    // Load configuration
    let config = config::Config::load(cli.config.clone())?;
    let config_path = cli
        .config
        .map(std::path::PathBuf::from)
        .or_else(|| dirs::config_dir().map(|d| d.join("govee-tui").join("config.toml")))
        .unwrap();

    // Validate API key
    if config.api.key.is_empty() || config.api.key == "YOUR_API_KEY_HERE" {
        eprintln!("❌ Error: No Govee API key configured!");
        eprintln!();
        eprintln!("A default config file has been created at:");
        eprintln!("  {}", config_path.display());
        eprintln!();
        eprintln!("Please edit it and replace YOUR_API_KEY_HERE with your actual API key:");
        eprintln!();
        eprintln!("  [api]");
        eprintln!("  key = \"your-actual-api-key-here\"");
        eprintln!("  timeout_ms = 5000");
        eprintln!("  retry_attempts = 3");
        eprintln!();
        eprintln!("To get a Govee API key:");
        eprintln!("  1. Download the Govee Home app");
        eprintln!("  2. Go to Settings → About Us → Apply for API Key");
        eprintln!("  3. Follow the email instructions");
        eprintln!("  4. Run govee-tui again after updating the config");
        eprintln!();
        eprintln!("Documentation: https://developer.govee.com");
        eprintln!();
        std::process::exit(1);
    }

    info!("Starting govee-tui v{}", env!("CARGO_PKG_VERSION"));

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

    if verbose {
        // Verbose mode: log everything to stderr
        let filter = EnvFilter::new("govee_tui=debug,info");
        tracing_subscriber::registry().with(filter).with(tracing_subscriber::fmt::layer()).init();
    } else {
        // Non-verbose: suppress all logging (TUI mode should be clean)
        let filter = EnvFilter::new("off");
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::sink))
            .init();
    }

    Ok(())
}
