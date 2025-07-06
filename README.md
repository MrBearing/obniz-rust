# Obniz Rust

🦀 A comprehensive Rust library for controlling obniz IoT devices via WebSocket communication.

## Features

- **Async/Await Support**: Fully asynchronous API using tokio
- **Type Safety**: Strong typing with custom error handling
- **Comprehensive Coverage**: Supports all major obniz features
- **Callback System**: Real-time event handling with callbacks
- **Memory Safe**: Built with Rust's memory safety guarantees

## Supported Modules

| Module | Status | Description |
|--------|--------|-------------|
| 🔧 **IO** | ✅ Complete | Digital pin control, stream mode, callbacks |
| 📺 **Display** | ✅ Complete | Text, graphics, QR codes, raw pixel data |
| ⚡ **AD** | ✅ Complete | Analog-to-digital conversion, voltage reading |
| 🌊 **PWM** | ✅ Complete | PWM generation, servo control, modulation |
| 📡 **UART** | ✅ Complete | Serial communication with flow control |
| 🔘 **Switch** | ✅ Complete | Built-in switch state monitoring |
| ⚙️ **System** | ✅ Complete | Device control, reset, configuration |
| 🔌 **SPI** | 🔄 Planned | SPI master/slave communication |
| 📶 **I2C** | 🔄 Planned | I2C bus communication |
| 📊 **Logic Analyzer** | 🔄 Planned | Digital signal analysis |
| 📏 **Measurement** | 🔄 Planned | Advanced measurement functions |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
obniz-rust = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use obniz_rust::*;

#[tokio::main]
async fn main() -> ObnizResult<()> {
    // Connect to your obniz device
    let obniz = connect_async("1234-5678").await?;
    
    // Digital IO
    let io = obniz.io();
    io.set_pin(0, true).await?;  // Set pin 0 high
    let state = io.get_pin(1).await?;  // Read pin 1
    
    // Display
    let display = obniz.display();
    display.text("Hello, Rust!").await?;
    display.qr("https://obniz.io", QrCorrectionType::Medium).await?;
    
    // Analog reading
    let ad = obniz.ad();
    let voltage = ad.get_voltage(0).await?;
    println!("Voltage: {:.2}V", voltage);
    
    // PWM for LED control
    let pwm = obniz.pwm();
    pwm.square_wave(0, 2, 1000).await?;  // 1kHz on pin 2
    
    Ok(())
}
```

### Advanced Example with Callbacks

```rust
use obniz_rust::*;

#[tokio::main]
async fn main() -> ObnizResult<()> {
    let obniz = connect_async("1234-5678").await?;
    
    // Stream mode for real-time pin monitoring
    let io = obniz.io();
    io.set_pin_callback(0, |state| {
        println!("Pin 0 changed to: {}", state);
    }).await?;
    
    // Analog sensor monitoring
    let ad = obniz.ad();
    ad.set_channel_callback(0, |voltage| {
        println!("Sensor reading: {:.2}V", voltage);
    }).await?;
    
    // Switch event handling
    let switch = obniz.switch();
    switch.on_push(|| {
        println!("Switch pressed!");
    }).await?;
    
    // Keep the program running to receive callbacks
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    
    Ok(())
}
```

## Module Documentation

### IO Control

```rust
let io = obniz.io();

// Basic pin operations
io.set_pin(0, true).await?;           // Set pin high
io.set_pin_as_output(1, false).await?; // Configure as output
io.set_pin_as_input(2, true).await?;   // Configure as input with stream

// Advanced configuration
let config = IoConfig {
    direction: Direction::Output,
    value: Some(true),
    output_type: Some(OutputType::PushPull5v),
    pull_type: Some(PullType::PullUp5v),
    stream: Some(false),
};
io.configure_pin(3, config).await?;

// Real-time monitoring
io.set_pin_callback(0, |state| {
    println!("Pin state changed: {}", state);
}).await?;
```

### Display Control

```rust
let display = obniz.display();

// Text and basic operations
display.text("Hello World").await?;
display.clear().await?;
display.brightness(75).await?;

// Graphics
display.rect(10, 10, 50, 30, true, true).await?;  // Filled rectangle
display.circle(80, 40, 20, false, true).await?;   // Circle outline
display.line(0, 0, 127, 63, true).await?;         // Diagonal line

// QR codes
display.qr("https://obniz.io", QrCorrectionType::High).await?;

// Raw pixel data
let config = RawDisplayConfig {
    width: 128,
    height: 64,
    color_depth: DisplayRawColorDepth::OneBit,
    data: pixel_data,
};
display.raw(config).await?;
```

### Analog Input (AD)

```rust
let ad = obniz.ad();

// Single reading
let voltage = ad.get_voltage(0).await?;
println!("Voltage: {:.2}V", voltage);

// Multiple channels
let readings = ad.get_voltages(vec![0, 1, 2]).await?;
for reading in readings {
    println!("AD{}: {:.2}V", reading.channel, reading.voltage);
}

// Stream mode with callback
ad.set_channel_callback(0, |voltage| {
    let percentage = AdManager::voltage_to_percentage(voltage);
    println!("Sensor: {:.2}V ({:.1}%)", voltage, percentage);
}).await?;

// Utility functions
let percentage = AdManager::voltage_to_percentage(3.3); // 66%
let is_safe = AdManager::is_voltage_safe(4.8); // true
```

### PWM Generation

```rust
let pwm = obniz.pwm();

// Basic PWM
pwm.configure_channel(0, PwmConfig {
    io_pin: 5,
    frequency: 1000,
    pulse_width_ms: 0.5,
}).await?;

// Servo control (0-180 degrees)
pwm.servo(1, 6, 90.0).await?;

// Square wave generation
pwm.square_wave(2, 7, 2000).await?; // 2kHz on pin 7

// Duty cycle control
pwm.set_channel_duty_cycle(0, 1000, 25.0).await?; // 25% duty

// Modulation for communication
let mod_config = ModulationConfig {
    modulation_type: ModulationType::Am,
    symbol_length_ms: 100.0,
    data: vec![0, 1, 1, 0, 1],
};
pwm.channel(0)?.modulate(mod_config).await?;
```

### UART Communication

```rust
let uart = obniz.uart();

// Simple configuration
let config = UartManager::simple_config(0, 1, 115200); // RX, TX, baud
uart.init_channel(0, config).await?;

// Send data
uart.send_string(0, "Hello UART!").await?;
uart.send_data(0, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]).await?;

// Receive callback
uart.set_string_callback(0, |data| {
    println!("Received: {}", data);
}).await?;

// Advanced configuration with flow control
let advanced_config = UartManager::flow_control_config(0, 1, 2, 3, 9600);
uart.init_channel(0, advanced_config).await?;
```

### Switch Monitoring

```rust
let switch = obniz.switch();

// Current state
let state = switch.get_state().await?;
println!("Switch: {}", state);

// Event callbacks
switch.on_push(|| println!("Pushed!")).await?;
switch.on_left(|| println!("Left!")).await?;
switch.on_right(|| println!("Right!")).await?;
switch.on_release(|| println!("Released!")).await?;

// Blocking wait for events
let pressed_state = switch.wait_for_press(Some(5000)).await?; // 5s timeout
switch.wait_for_release(None).await?; // No timeout
```

## Error Handling

The library provides comprehensive error handling with the `ObnizResult<T>` type:

```rust
match obniz.io().get_pin(0).await {
    Ok(state) => println!("Pin state: {}", state),
    Err(ObnizError::InvalidPin(pin)) => println!("Invalid pin: {}", pin),
    Err(ObnizError::Connection(msg)) => println!("Connection error: {}", msg),
    Err(ObnizError::Timeout) => println!("Operation timed out"),
    Err(e) => println!("Other error: {}", e),
}
```

## Examples

The repository includes comprehensive examples:

- **`basic_example.rs`** - Simple IO and display operations
- **`io_example.rs`** - Complete IO functionality demonstration
- **`display_example.rs`** - Display and graphics features
- **`comprehensive_example.rs`** - All modules working together
- **`device_test_template.rs`** - Template for testing with real devices
- **`mock_integration_example.rs`** - Mock system demonstration

Run examples with:

```bash
cargo run --example comprehensive_example
```

### Testing with Real Devices

For testing with actual obniz devices:

1. Copy `device_test_template.rs` to a new file
2. Replace `"YOUR-OBNIZ-ID"` with your actual device ID
3. **Important**: Never commit files containing real device IDs to version control

```bash
# Copy template and edit device ID
cp examples/device_test_template.rs examples/my_device_test.rs
# Edit the OBNIZ_ID constant in my_device_test.rs
cargo run --example my_device_test
```

## Device Compatibility

Tested with:
- obniz Board
- obniz Board 1Y
- M5StickC (obniz firmware)

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [obniz](https://obniz.io) for the excellent IoT platform
- The Rust community for amazing async ecosystem