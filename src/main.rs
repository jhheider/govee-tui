use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod api;
mod cache;
mod config;
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
    /// Launch interactive TUI (default)
    Tui,

    /// List all devices with details
    Devices,

    /// Get device state and capabilities
    Status {
        /// Device name (fuzzy match) or exact ID
        device: String,
    },

    /// List available light scenes for a device
    Scenes {
        /// Device name (fuzzy match) or exact ID
        device: String,
    },

    /// Control a device
    Control {
        /// Device name (fuzzy match) or exact ID
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

    /// Apply a light scene by name (see `scenes` for the list)
    Scene { name: String },
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
        .unwrap_or_else(|| std::path::PathBuf::from("./config.toml"));

    // Validate API key
    if config.api.key.is_empty() || config.api.key == "YOUR_API_KEY_HERE" {
        eprintln!("❌ Error: No Govee API key configured");
        eprintln!();
        eprintln!("Config file:");
        eprintln!("  {}", config_path.display());
        eprintln!();
        eprintln!("Edit it and replace YOUR_API_KEY_HERE with your actual API key:");
        eprintln!();
        eprintln!("  [api]");
        eprintln!("  key = \"your-actual-api-key-here\"");
        eprintln!();
        eprintln!("(or set the GOVEE_API_KEY environment variable)");
        eprintln!();
        eprintln!("To get a Govee API key:");
        eprintln!("  1. Download the Govee Home app");
        eprintln!("  2. Go to Settings → About Us → Apply for API Key");
        eprintln!("  3. Follow the email instructions");
        eprintln!();
        eprintln!("Then run govee-tui again.");
        eprintln!();
        eprintln!("Documentation: https://developer.govee.com");
        eprintln!();
        std::process::exit(1);
    }

    info!("Starting govee-tui v{}", env!("CARGO_PKG_VERSION"));

    // Initialize API client (config carries timeout and retry policy)
    let client = api::Client::new(&config.api)?;

    // Execute command
    match cli.command {
        None | Some(Commands::Tui) => {
            ui::run(client, config).await?;
        }
        Some(Commands::Devices) => {
            cmd_list_devices(&client).await?;
        }
        Some(Commands::Status { device }) => {
            cmd_device_status(&client, &device).await?;
        }
        Some(Commands::Scenes { device }) => {
            cmd_list_scenes(&client, &device).await?;
        }
        Some(Commands::Control { device, command }) => {
            cmd_control_device(&client, &device, command).await?;
        }
    }

    Ok(())
}

// ==================== CLI COMMANDS ====================

async fn cmd_list_devices(client: &api::Client) -> Result<()> {
    println!("📡 Fetching devices from Govee API...\n");

    let devices = client.get_devices().await?;

    if devices.is_empty() {
        println!("⚠️  No devices found");
        return Ok(());
    }

    println!("Found {} device(s):\n", devices.len());
    println!(
        "{:<4} {:<30} {:<20} {:<12} ID",
        "#", "Name", "Model", "Controllable"
    );
    println!("{}", "=".repeat(72));

    for (i, device) in devices.iter().enumerate() {
        let controllable = if device.controllable { "Yes" } else { "No" };
        println!(
            "{:<4} {:<30} {:<20} {:<12} {}",
            i + 1,
            truncate(&device.name, 30),
            truncate(&device.model, 20),
            controllable,
            device.id
        );
    }

    Ok(())
}

async fn cmd_device_status(client: &api::Client, device_query: &str) -> Result<()> {
    println!("🔍 Searching for device: {device_query}\n");

    let devices = client.get_devices().await?;
    let device = find_device(&devices, device_query)?;

    println!("📱 Device: {}", device.name);
    println!("   ID: {}", device.id);
    println!("   Model: {}", device.model);
    println!(
        "   Controllable: {}",
        if device.controllable {
            "✓ Yes"
        } else {
            "⚪ No"
        }
    );
    println!(
        "   Retrievable: {}",
        if device.retrievable {
            "✓ Yes"
        } else {
            "⚪ No"
        }
    );

    if !device.retrievable {
        println!("\n⚠️  Device does not support state retrieval");
        return Ok(());
    }

    println!("\n📊 Fetching current state...");

    match client.get_device_state(&device.id, &device.model).await {
        Ok(state) => {
            println!(
                "\n   Power: {}",
                if state.power { "✅ ON" } else { "⭕ OFF" }
            );

            if let Some(brightness) = state.brightness {
                println!("   Brightness: {brightness}%");
            }

            if let Some(color) = state.color {
                println!(
                    "   Color: RGB({}, {}, {}) #{:02X}{:02X}{:02X}",
                    color.r, color.g, color.b, color.r, color.g, color.b
                );
            }

            // 0K means the device is in color mode, not temperature mode
            if let Some(temp) = state.color_temp.filter(|t| *t > 0) {
                println!("   Color Temp: {temp}K");
            }
        }
        Err(e) => {
            println!("\n❌ Failed to get device state: {e}");
        }
    }

    Ok(())
}

async fn cmd_control_device(
    client: &api::Client,
    device_query: &str,
    command: ControlCommand,
) -> Result<()> {
    use api::Command;

    println!("🔍 Searching for device: {device_query}\n");

    let devices = client.get_devices().await?;
    let device = find_device(&devices, device_query)?;

    if !device.controllable {
        anyhow::bail!("❌ Device '{}' is not controllable", device.name);
    }

    let cmd = match command {
        ControlCommand::Scene { name } => {
            let scenes = client.get_scenes(&device.id, &device.model).await?;
            let name_lower = name.to_lowercase();
            let scene = scenes
                .iter()
                .find(|s| s.name.to_lowercase() == name_lower)
                .or_else(|| {
                    scenes
                        .iter()
                        .find(|s| s.name.to_lowercase().contains(&name_lower))
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "❌ No scene matching '{name}' on '{}' (try `govee-tui scenes \"{}\"`)",
                        device.name,
                        device.name
                    )
                })?;
            println!("🎬 Applying scene '{}'...", scene.name);
            Command::Scene(scene.clone())
        }
        ControlCommand::Turn { state } => {
            let on = state.to_lowercase() == "on";
            println!(
                "💡 Turning {} {}...",
                device.name,
                if on { "ON" } else { "OFF" }
            );
            Command::turn(on)
        }
        ControlCommand::Brightness { value } => {
            println!("🔆 Setting brightness to {value}%...");
            Command::brightness(value)
        }
        ControlCommand::Color { r, g, b } => {
            println!("🌈 Setting color to RGB({r}, {g}, {b}) #{r:02X}{g:02X}{b:02X}...");
            Command::color(r, g, b)
        }
        ControlCommand::Temp { kelvin } => {
            println!("🌡️  Setting color temperature to {kelvin}K...");
            Command::temperature(kelvin)
        }
    };

    client
        .control_device(&device.id, &device.model, cmd)
        .await?;

    println!("✅ Command executed successfully!");
    Ok(())
}

async fn cmd_list_scenes(client: &api::Client, device_query: &str) -> Result<()> {
    println!("🔍 Searching for device: {device_query}\n");

    let devices = client.get_devices().await?;
    let device = find_device(&devices, device_query)?;

    println!("🎬 Fetching scenes for {}...\n", device.name);
    let scenes = client.get_scenes(&device.id, &device.model).await?;

    if scenes.is_empty() {
        println!("⚠️  No scenes available for this device");
        return Ok(());
    }

    for scene in &scenes {
        if scene.param_id.is_none() {
            println!("  {}  (DIY)", scene.name);
        } else {
            println!("  {}", scene.name);
        }
    }
    println!(
        "\n💡 Apply one with: govee-tui control \"{}\" scene \"<name>\"",
        device.name
    );
    Ok(())
}

// ==================== HELPERS ====================

fn find_device<'a>(devices: &'a [api::Device], query: &str) -> Result<&'a api::Device> {
    // Try exact ID match first
    if let Some(device) = devices.iter().find(|d| d.id == query) {
        return Ok(device);
    }

    // Try fuzzy name match
    let query_lower = query.to_lowercase();
    let matches: Vec<&api::Device> = devices
        .iter()
        .filter(|d| d.name.to_lowercase().contains(&query_lower))
        .collect();

    match matches.len() {
        0 => anyhow::bail!("❌ No device found matching '{query}'"),
        1 => Ok(matches[0]),
        _ => {
            println!("⚠️  Multiple devices match '{query}':");
            for device in matches {
                println!("   - {} ({})", device.name, device.id);
            }
            anyhow::bail!("Please be more specific or use the exact device ID");
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{cut}...")
    }
}

fn init_logging(verbose: bool) -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    if verbose {
        // Verbose mode: log everything to stderr
        let filter = EnvFilter::new("govee_tui=debug,info");
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
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
