//! # govee-api2
//!
//! A Rust client for Govee's v2 router-based platform API
//! (`https://openapi.api.govee.com`).
//!
//! Supported:
//! - Device and group discovery
//! - Device state queries
//! - Device control (power, brightness, color, color temperature)
//! - Dynamic light scenes and DIY scenes (list + activate)
//! - Per-segment color and brightness for segmented lights
//! - Configurable timeout and retry with backoff, typed rate-limit errors
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
//!         client.turn_on(&device.device, &device.sku).await?;
//!         client.set_brightness(&device.device, &device.sku, 80).await?;
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

/// Compile the README's code examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

pub use client::{ClientConfig, GoveeClient};
pub use error::{Error, Result};
pub use types::*;
