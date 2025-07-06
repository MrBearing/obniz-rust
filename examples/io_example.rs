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

    // Get IO manager
    let io = obniz.io();

    // Example 1: Basic pin operations
    println!("Setting pin 0 to output high");
    io.set_pin_as_output(0, true).await?;

    sleep(Duration::from_millis(1000)).await;

    println!("Setting pin 0 to output low");
    io.set_pin(0, false).await?;

    // Example 2: Reading pin state
    println!("Setting pin 1 as input");
    io.set_pin_as_input(1, false).await?;

    let state = io.get_pin(1).await?;
    println!("Pin 1 state: {state}");

    // Example 3: Stream mode with callback
    println!("Setting up stream mode on pin 2");
    io.set_pin_callback(2, |state| {
        println!("Pin 2 changed to: {state}");
    })
    .await?;

    // Example 4: Advanced pin configuration
    let config = IoConfig {
        direction: Direction::Output,
        value: Some(true),
        output_type: Some(OutputType::PushPull5v),
        pull_type: Some(PullType::PullUp5v),
        stream: Some(false),
    };

    println!("Configuring pin 3 with advanced settings");
    io.configure_pin(3, config).await?;

    // Example 5: Using individual pin instances
    let pin4 = io.pin(4)?;
    pin4.set_as_output(true).await?;

    println!("Set pin 4 as output high");

    // Wait for some time to see callbacks
    println!("Waiting for 10 seconds to observe pin changes...");
    sleep(Duration::from_secs(10)).await;

    // Clean up
    io.remove_pin_callback(2)?;

    println!("Example completed successfully!");

    Ok(())
}
