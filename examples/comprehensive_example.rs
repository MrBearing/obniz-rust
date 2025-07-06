use obniz_rust::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> ObnizResult<()> {
    // Replace with your actual obniz device ID
    let device_id = "1234-5678";

    println!("=== Obniz Rust Comprehensive Example ===");

    // Connect to the obniz device
    let obniz = connect_async(device_id).await?;
    println!("✓ Connected to obniz device: {}", obniz.id());

    // Example 1: System Information
    println!("\n--- System Information ---");
    let system = obniz.system();

    // Get system info (this might not work on all devices)
    match system.info().await {
        Ok(info) => {
            println!("Hardware: {}", info.hardware);
            println!("Firmware: {}", info.version);
        }
        Err(e) => println!("Could not get system info: {}", e),
    }

    // Configure auto-reset on disconnect
    system.reset_on_disconnect(true).await?;
    println!("✓ Configured auto-reset on disconnect");

    // Example 2: Display Operations
    println!("\n--- Display Operations ---");
    let display = obniz.display();

    display.clear().await?;
    display.text("Obniz Rust Demo").await?;
    println!("✓ Displayed welcome message");

    sleep(Duration::from_secs(2)).await;

    // Draw some graphics
    display.clear().await?;
    display.rect(10, 10, 40, 20, false, true).await?;
    display.circle(80, 30, 10, true, true).await?;
    display.line(0, 0, 127, 63, true).await?;
    println!("✓ Drew geometric shapes");

    sleep(Duration::from_secs(2)).await;

    // Example 3: IO Operations
    println!("\n--- IO Operations ---");
    let io = obniz.io();

    // Set pin 0 as output and toggle it
    io.set_pin_as_output(0, true).await?;
    println!("✓ Set pin 0 high");
    sleep(Duration::from_millis(500)).await;

    io.set_pin(0, false).await?;
    println!("✓ Set pin 0 low");

    // Configure pin 1 as input
    io.set_pin_as_input(1, false).await?;
    let pin1_state = io.get_pin(1).await?;
    println!("✓ Pin 1 state: {pin1_state}");

    // Example 4: AD (Analog) Operations
    println!("\n--- AD (Analog) Operations ---");
    let ad = obniz.ad();

    // Read voltage from AD channel 0
    let voltage = ad.get_voltage(0).await?;
    println!(
        "✓ AD0 voltage: {:.2}V ({}%)",
        voltage,
        AdManager::voltage_to_percentage(voltage)
    );

    // Read from multiple channels
    let voltages = ad.get_voltages(vec![0, 1, 2]).await?;
    for reading in voltages {
        println!("  AD{}: {:.2}V", reading.channel, reading.voltage);
    }

    // Example 5: PWM Operations
    println!("\n--- PWM Operations ---");
    let pwm = obniz.pwm();

    // Generate 1kHz square wave on pin 2
    pwm.square_wave(0, 2, 1000).await?;
    println!("✓ Started 1kHz square wave on pin 2 (PWM0)");

    sleep(Duration::from_secs(1)).await;

    // Control a servo on pin 3
    pwm.servo(1, 3, 90.0).await?; // 90 degrees
    println!("✓ Set servo to 90 degrees on pin 3 (PWM1)");

    sleep(Duration::from_secs(1)).await;

    // Set custom duty cycle
    pwm.set_channel_duty_cycle(0, 2000, 25.0).await?; // 25% duty cycle at 2kHz
    println!("✓ Set 25% duty cycle at 2kHz on PWM0");

    // Example 6: UART Operations
    println!("\n--- UART Operations ---");
    let uart = obniz.uart();

    // Configure UART with default settings (pins 4 and 5)
    let uart_config = UartManager::simple_config(4, 5, 9600);
    uart.init_channel(0, uart_config).await?;
    println!("✓ Initialized UART0 (RX: pin4, TX: pin5, 9600 baud)");

    // Send some data
    uart.send_string(0, "Hello from Rust!").await?;
    println!("✓ Sent 'Hello from Rust!' via UART");

    // Example 7: Switch Operations (if available on device)
    println!("\n--- Switch Operations ---");
    let switch = obniz.switch();

    // Get current switch state
    let switch_state = switch.get_state().await?;
    println!("✓ Switch state: {switch_state}");

    // Set up switch callbacks
    switch
        .on_push(|| {
            println!("Switch pushed!");
        })
        .await?;

    switch
        .on_left(|| {
            println!("Switch moved left!");
        })
        .await?;

    switch
        .on_right(|| {
            println!("Switch moved right!");
        })
        .await?;

    println!("✓ Switch callbacks configured");

    // Example 8: Combined Operations - LED Breathing Effect
    println!("\n--- Combined Demo: LED Breathing Effect ---");
    display.clear().await?;
    display.text("LED Breathing").await?;

    // Use PWM to create breathing effect on pin 6
    pwm.configure_channel(
        2,
        PwmConfig {
            io_pin: 6,
            frequency: 1000,
            pulse_width_ms: 0.0,
        },
    )
    .await?;

    println!("✓ Starting LED breathing effect on pin 6...");

    for cycle in 0..3 {
        // Breathe in
        for i in 0..=100 {
            let duty = i as f64;
            pwm.set_channel_duty_cycle(2, 1000, duty).await?;
            sleep(Duration::from_millis(20)).await;
        }

        // Breathe out
        for i in (0..=100).rev() {
            let duty = i as f64;
            pwm.set_channel_duty_cycle(2, 1000, duty).await?;
            sleep(Duration::from_millis(20)).await;
        }

        println!("  Breathing cycle {} completed", cycle + 1);
    }

    // Example 9: Sensor Reading Simulation
    println!("\n--- Sensor Reading Simulation ---");
    display.clear().await?;
    display.text("Sensor Monitor").await?;

    for i in 1..=5 {
        // Read multiple analog sensors
        let sensor_readings = ad.read_all().await?;

        println!("Reading {}: ", i);
        for reading in sensor_readings.iter().take(4) {
            // Show first 4 channels
            let percentage = AdManager::voltage_to_percentage(reading.voltage);
            println!(
                "  Sensor AD{}: {:.2}V ({:.1}%)",
                reading.channel, reading.voltage, percentage
            );
        }

        // Update display with first sensor value
        if let Some(first_reading) = sensor_readings.first() {
            let percentage = AdManager::voltage_to_percentage(first_reading.voltage);
            display.clear().await?;
            display.text(&format!("Sensor: {:.1}%", percentage)).await?;
        }

        sleep(Duration::from_secs(1)).await;
    }

    // Example 10: Cleanup and Final Display
    println!("\n--- Cleanup ---");

    // Stop all PWM channels
    pwm.deinit_all().await?;
    println!("✓ Stopped all PWM channels");

    // Clear callbacks
    switch.remove_callback()?;
    uart.remove_channel_callback(0)?;
    println!("✓ Removed callbacks");

    // Final display message
    display.clear().await?;
    display.text("Demo Complete!").await?;
    display.brightness(100).await?;

    // Show QR code with project info
    sleep(Duration::from_secs(2)).await;
    display
        .qr(
            "https://github.com/your-username/obniz-rust",
            QrCorrectionType::Medium,
        )
        .await?;

    println!("\n=== Demo Completed Successfully! ===");
    println!("Check your obniz device for the final QR code.");

    Ok(())
}
