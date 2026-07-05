//! List (and optionally apply) light scenes for a device.
//!
//! Usage:
//!   GOVEE_API_KEY=... cargo run --example scenes -- "<device name>" [scene name]

use govee_api2::GoveeClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GOVEE_API_KEY").expect("set GOVEE_API_KEY");
    let mut args = std::env::args().skip(1);
    let device_query = args
        .next()
        .expect("usage: scenes <device name> [scene name]");
    let apply = args.next();

    let client = GoveeClient::new(api_key);

    let devices = client.get_devices().await?;
    let device = devices
        .iter()
        .find(|d| {
            d.device_name
                .to_lowercase()
                .contains(&device_query.to_lowercase())
        })
        .expect("no device matching that name");
    println!("Device: {} ({})", device.device_name, device.sku);

    let dynamic = client
        .get_dynamic_scenes(&device.device, &device.sku)
        .await?;
    let diy = client.get_diy_scenes(&device.device, &device.sku).await?;
    println!("{} dynamic scenes, {} DIY scenes", dynamic.len(), diy.len());
    for scene in dynamic.iter().take(10) {
        println!(
            "  {} (id {}, paramId {:?})",
            scene.name, scene.id, scene.param_id
        );
    }
    for scene in &diy {
        println!("  {} (DIY, id {})", scene.name, scene.id);
    }

    if let Some(name) = apply {
        let scene = dynamic
            .iter()
            .chain(diy.iter())
            .find(|s| s.name.to_lowercase() == name.to_lowercase())
            .expect("no scene with that name");
        client.set_scene(&device.device, &device.sku, scene).await?;
        println!("Applied scene: {}", scene.name);
    }

    Ok(())
}
