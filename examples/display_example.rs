use obniz_rust::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> ObnizResult<()> {
    // Replace with your actual obniz device ID
    let device_id = "1234-5678";

    // Connect to the obniz device
    let obniz = connect_async(device_id).await?;
    println!("Connected to obniz device: {}", obniz.id());

    // Get display manager
    let display = obniz.display();

    // Example 1: Basic text display
    println!("Displaying 'Hello, World!'");
    display.text("Hello, World!").await?;

    sleep(Duration::from_secs(2)).await;

    // Example 2: Clear display
    println!("Clearing display");
    display.clear().await?;

    sleep(Duration::from_secs(1)).await;

    // Example 3: QR code display
    println!("Displaying QR code");
    display
        .qr("https://obniz.io", QrCorrectionType::Medium)
        .await?;

    sleep(Duration::from_secs(3)).await;

    // Example 4: Adjust brightness
    println!("Setting brightness to 50%");
    display.brightness(50).await?;

    sleep(Duration::from_secs(1)).await;

    // Example 5: Drawing operations
    display.clear().await?;

    println!("Drawing geometric shapes");

    // Draw a rectangle
    display.rect(10, 10, 50, 30, false, true).await?;

    // Draw a filled circle
    display.circle(80, 25, 15, true, true).await?;

    // Draw a line
    display.line(0, 0, 127, 63, true).await?;

    sleep(Duration::from_secs(2)).await;

    // Example 6: Pixel manipulation
    display.clear().await?;

    println!("Drawing individual pixels");

    for x in 0..128 {
        for y in 0..64 {
            if (x + y) % 10 == 0 {
                display.pixel(x, y, true).await?;
            }
        }
    }

    sleep(Duration::from_secs(2)).await;

    // Example 7: Text with custom positioning and size
    display.clear().await?;

    println!("Custom text positioning and size");

    display.text_size(2).await?;
    display.text_pos(10, 10).await?;
    display.text("BIG TEXT").await?;

    display.text_size(1).await?;
    display.text_pos(10, 40).await?;
    display.text("Small text below").await?;

    sleep(Duration::from_secs(3)).await;

    // Example 8: Raw display data (create a simple pattern)
    display.clear().await?;

    println!("Displaying raw pixel data");

    // Create a simple checkerboard pattern (1-bit depth, 128x64 display)
    let mut raw_data = vec![0u16; 1024]; // 128 * 64 / 8 = 1024 bytes

    for i in 0..raw_data.len() {
        raw_data[i] = if (i / 8) % 2 == 0 { 0xAA } else { 0x55 };
    }

    let raw_config = RawDisplayConfig {
        width: 128,
        height: 64,
        color_depth: DisplayRawColorDepth::OneBit,
        data: raw_data,
    };

    display.raw(raw_config).await?;

    sleep(Duration::from_secs(3)).await;

    // Example 9: Pin assignment for external display modules
    println!("Setting up pin assignments for external display");

    let pin_assignments = vec![
        PinAssignment {
            pin: 0,
            module_name: "spi".to_string(),
            pin_name: "clk".to_string(),
        },
        PinAssignment {
            pin: 1,
            module_name: "spi".to_string(),
            pin_name: "mosi".to_string(),
        },
        PinAssignment {
            pin: 2,
            module_name: "display".to_string(),
            pin_name: "cs".to_string(),
        },
    ];

    display.pin_assign(pin_assignments).await?;

    // Example 10: Using legacy trait (backward compatibility)
    println!("Using legacy ObnizDisplay trait");

    // This uses the blocking interface
    obniz.display_clear()?;
    obniz.display_text("Legacy interface")?;

    sleep(Duration::from_secs(2)).await;

    // Final cleanup
    display.clear().await?;
    display.text("Display Demo Complete!").await?;

    sleep(Duration::from_secs(2)).await;
    display.clear().await?;

    println!("Display example completed successfully!");

    Ok(())
}
