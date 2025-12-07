//! # govee-api2
//!
//! A Rust client for Govee's v2 router-based API.
//!
//! This crate provides a complete implementation of the Govee API v2 endpoints,
//! including support for:
//! - Device discovery and listing
//! - Device groups
//! - Device control (power, brightness, color, color temperature)
//! - Dynamic scenes and DIY scenes
//! - Segment color control for RGBIC devices
//! - Device state queries
//!
//! ## Features
//!
//! - **TLS Backend Selection**: By default uses `rustls` (pure Rust TLS). Enable
//!   `native-tls` feature to use the system's TLS implementation instead.
//!
//! ```toml
//! # Default (rustls)
//! govee-api2 = "0.1"
//!
//! # Use native TLS (OpenSSL on Linux, Secure Transport on macOS)
//! govee-api2 = { version = "0.1", default-features = false, features = ["native-tls"] }
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use govee_api2::GoveeClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = GoveeClient::new("your-api-key");
//!
//!     // List all devices
//!     let devices = client.get_devices().await?;
//!     println!("Found {} devices", devices.len());
//!
//!     // Control a device
//!     if let Some(device) = devices.first() {
//!         client.turn_on(&device.device, &device.sku).await?;
//!         client.set_brightness(&device.device, &device.sku, 80).await?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Custom Client Configuration
//!
//! Use the builder pattern for custom timeout and user-agent:
//!
//! ```rust,no_run
//! use govee_api2::GoveeClient;
//! use std::time::Duration;
//!
//! let client = GoveeClient::builder("your-api-key")
//!     .timeout(Duration::from_secs(10))
//!     .user_agent("my-app/1.0")
//!     .build()
//!     .expect("Failed to build client");
//! ```
//!
//! ## Getting a Govee API Key
//!
//! 1. Download the Govee Home app
//! 2. Go to Settings → About Us → Apply for API Key
//! 3. Follow the instructions to receive your key via email

pub mod client;
pub mod error;
pub mod types;

pub use client::{GoveeClient, GoveeClientBuilder};
pub use error::{Error, Result};
pub use types::*;
