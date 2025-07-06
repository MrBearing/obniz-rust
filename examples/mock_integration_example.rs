use obniz_rust::*;
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Obniz Mock Integration Example");

    // Create mock configuration
    let mock_config = MockConfig {
        device_id: "mock-obniz-1234".to_string(),
        should_fail_connection: false,
        should_timeout: false,
        default_delay_ms: 10,
    };

    // Create mock device
    let mock_device = MockObniz::new(mock_config);
    let server = mock_device.server();

    // Configure mock responses for different modules
    setup_mock_responses(&server);

    println!("✅ Mock server configured with responses");

    // Test IO operations
    println!("\n📌 Testing IO Operations");
    test_io_operations(&server).await?;

    // Test AD operations
    println!("\n🔬 Testing AD Operations");
    test_ad_operations(&server).await?;

    // Test PWM operations
    println!("\n⚡ Testing PWM Operations");
    test_pwm_operations(&server).await?;

    // Test UART operations
    println!("\n📡 Testing UART Operations");
    test_uart_operations(&server).await?;

    // Test Display operations
    println!("\n🖥️ Testing Display Operations");
    test_display_operations(&server).await?;

    // Test Switch operations
    println!("\n🔘 Testing Switch Operations");
    test_switch_operations(&server).await?;

    // Test System operations
    println!("\n🔧 Testing System Operations");
    test_system_operations(&server).await?;

    // Test callback functionality
    println!("\n🔔 Testing Callback System");
    test_callback_system(&mock_device).await?;

    // Show message history
    println!("\n📝 Message History:");
    let sent_messages = server.get_sent_messages();
    for (i, msg) in sent_messages.iter().enumerate() {
        println!("  {}. {}", i + 1, msg);
    }

    println!("\n🎉 All tests completed successfully!");
    println!("💡 Mock system is ready for unit testing without hardware");

    Ok(())
}

fn setup_mock_responses(server: &Arc<MockWebSocketServer>) {
    // IO responses
    server.add_response("io0", responses::io_pin_state(0, false));
    server.add_response("io1", responses::io_pin_state(1, true));
    server.add_response("io2", json!([{"io2": {"state": "ok"}}]));

    // AD responses
    server.add_response("ad0", responses::ad_voltage(0, 3.3));
    server.add_response("ad1", responses::ad_voltage(1, 1.65));
    server.add_response("ad2", responses::ad_voltage(2, 0.0));

    // PWM responses
    server.add_response("pwm0", responses::pwm_ok(0));
    server.add_response("pwm1", responses::pwm_ok(1));

    // UART responses
    server.add_response(
        "uart0",
        responses::uart_data(0, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]),
    ); // "Hello"
    server.add_response("uart1", json!([{"uart1": {"state": "ok"}}]));

    // Display responses
    server.add_response("display", responses::display_ok());
    server.add_response("display.text", responses::display_ok());
    server.add_response("display.clear", responses::display_ok());

    // Switch responses
    server.add_response("switch", responses::switch_state("none", "get"));

    // System responses
    server.add_response("system", responses::system_info());
    server.add_response("system.info", responses::system_info());
}

async fn test_io_operations(server: &Arc<MockWebSocketServer>) -> ObnizResult<()> {
    use tokio_tungstenite::tungstenite::Message;

    // Test IO get
    let get_request = json!([{"io0": "get"}]);
    let message = Message::from(get_request.to_string());
    let response = server.process_message(message).await?;
    println!("  📥 IO get: {:?}", response);

    // Test IO set
    let set_request = json!([{"io1": true}]);
    let message = Message::from(set_request.to_string());
    let response = server.process_message(message).await?;
    println!("  📤 IO set: {:?}", response);

    // Test IO config
    let config_request = json!([{
        "io2": {
            "direction": "output",
            "value": true,
            "output_type": "push-pull5v"
        }
    }]);
    let message = Message::from(config_request.to_string());
    let response = server.process_message(message).await?;
    println!("  ⚙️ IO config: {:?}", response);

    Ok(())
}

async fn test_ad_operations(server: &Arc<MockWebSocketServer>) -> ObnizResult<()> {
    use tokio_tungstenite::tungstenite::Message;

    // Test AD get
    let get_request = json!([{"ad0": "get"}]);
    let message = Message::from(get_request.to_string());
    let response = server.process_message(message).await?;
    println!("  📊 AD get: {:?}", response);

    // Test AD stream
    let stream_request = json!([{
        "ad1": {
            "stream": true,
            "interval": 100
        }
    }]);
    let message = Message::from(stream_request.to_string());
    let response = server.process_message(message).await?;
    println!("  📈 AD stream: {:?}", response);

    Ok(())
}

async fn test_pwm_operations(server: &Arc<MockWebSocketServer>) -> ObnizResult<()> {
    use tokio_tungstenite::tungstenite::Message;

    // Test PWM config
    let config_request = json!([{
        "pwm0": {
            "io": 5,
            "freq": 1000,
            "pulse": 0.5
        }
    }]);
    let message = Message::from(config_request.to_string());
    let response = server.process_message(message).await?;
    println!("  ⚡ PWM config: {:?}", response);

    // Test PWM servo
    let servo_request = json!([{
        "pwm1": {
            "io": 6,
            "freq": 50,
            "pulse": 1.5
        }
    }]);
    let message = Message::from(servo_request.to_string());
    let response = server.process_message(message).await?;
    println!("  🤖 PWM servo: {:?}", response);

    Ok(())
}

async fn test_uart_operations(server: &Arc<MockWebSocketServer>) -> ObnizResult<()> {
    use tokio_tungstenite::tungstenite::Message;

    // Test UART config
    let config_request = json!([{
        "uart0": {
            "rx": 0,
            "tx": 1,
            "baud": 115200
        }
    }]);
    let message = Message::from(config_request.to_string());
    let response = server.process_message(message).await?;
    println!("  📡 UART config: {:?}", response);

    // Test UART write
    let write_request = json!([{
        "uart0": {
            "data": [0x48, 0x65, 0x6C, 0x6C, 0x6F] // "Hello"
        }
    }]);
    let message = Message::from(write_request.to_string());
    let response = server.process_message(message).await?;
    println!("  📝 UART write: {:?}", response);

    Ok(())
}

async fn test_display_operations(server: &Arc<MockWebSocketServer>) -> ObnizResult<()> {
    use tokio_tungstenite::tungstenite::Message;

    // Test display text
    let text_request = json!([{
        "display": {
            "text": "Hello Obniz!"
        }
    }]);
    let message = Message::from(text_request.to_string());
    let response = server.process_message(message).await?;
    println!("  📝 Display text: {:?}", response);

    // Test display clear
    let clear_request = json!([{
        "display": {
            "clear": true
        }
    }]);
    let message = Message::from(clear_request.to_string());
    let response = server.process_message(message).await?;
    println!("  🧹 Display clear: {:?}", response);

    Ok(())
}

async fn test_switch_operations(server: &Arc<MockWebSocketServer>) -> ObnizResult<()> {
    use tokio_tungstenite::tungstenite::Message;

    // Test switch get
    let get_request = json!([{"switch": "get"}]);
    let message = Message::from(get_request.to_string());
    let response = server.process_message(message).await?;
    println!("  🔘 Switch get: {:?}", response);

    // Test switch callback
    let callback_request = json!([{
        "switch": {
            "callback": true
        }
    }]);
    let message = Message::from(callback_request.to_string());
    let response = server.process_message(message).await?;
    println!("  🔔 Switch callback: {:?}", response);

    Ok(())
}

async fn test_system_operations(server: &Arc<MockWebSocketServer>) -> ObnizResult<()> {
    use tokio_tungstenite::tungstenite::Message;

    // Test system info
    let info_request = json!([{
        "system": {
            "info": true
        }
    }]);
    let message = Message::from(info_request.to_string());
    let response = server.process_message(message).await?;
    println!("  ℹ️ System info: {:?}", response);

    // Test system reset
    let reset_request = json!([{
        "system": {
            "reset": true
        }
    }]);
    let message = Message::from(reset_request.to_string());
    let response = server.process_message(message).await?;
    println!("  🔄 System reset: {:?}", response);

    Ok(())
}

async fn test_callback_system(mock_device: &MockObniz) -> ObnizResult<()> {
    use std::sync::atomic::{AtomicBool, Ordering};

    // Test callback registration
    let callback_triggered = Arc::new(AtomicBool::new(false));
    let callback_triggered_clone = callback_triggered.clone();

    mock_device.register_callback("test_callback".to_string(), move |data| {
        println!("  🔔 Callback triggered with data: {:?}", data);
        callback_triggered_clone.store(true, Ordering::SeqCst);
    })?;

    // Simulate callback trigger
    let test_data = json!({"pin": 0, "value": true});
    mock_device
        .server()
        .trigger_callback("test_callback", test_data)
        .await;

    // Wait a bit for callback to process
    sleep(Duration::from_millis(10)).await;

    if callback_triggered.load(Ordering::SeqCst) {
        println!("  ✅ Callback system working correctly");
    } else {
        println!("  ❌ Callback system not working");
    }

    // Unregister callback
    mock_device.unregister_callback("test_callback".to_string())?;
    println!("  🗑️ Callback unregistered");

    Ok(())
}
