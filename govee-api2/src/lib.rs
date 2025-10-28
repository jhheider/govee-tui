//! # govee-api2
//!
//! A Rust client for Govee's v2 router-based API.
//!
//! This crate provides a complete implementation of the Govee API v2 endpoints,
//! including support for:
//! - Device discovery and listing
//! - Device groups
//! - Device control (power, brightness, color, color temperature)
//! - Device state queries
//!
//! ## Example
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
//!         client.turn_on(&device.device).await?;
//!         client.set_brightness(&device.device, 80).await?;
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

pub use client::GoveeClient;
pub use error::{Error, Result};
pub use types::*;
