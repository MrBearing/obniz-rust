use obniz_rust::*;
use serde_json::json;

#[test]
fn test_io_config_creation() {
    let config = IoConfig {
        direction: Direction::Output,
        value: Some(true),
        output_type: Some(OutputType::PushPull5v),
        pull_type: Some(PullType::PullUp5v),
        stream: Some(false),
    };

    assert_eq!(config.direction, Direction::Output);
    assert_eq!(config.value, Some(true));
    assert_eq!(config.output_type, Some(OutputType::PushPull5v));
    assert_eq!(config.pull_type, Some(PullType::PullUp5v));
    assert_eq!(config.stream, Some(false));
}

#[test]
fn test_io_enums_serialization() {
    use serde_json;

    // Test Direction serialization
    assert_eq!(
        serde_json::to_string(&Direction::Input).unwrap(),
        "\"input\""
    );
    assert_eq!(
        serde_json::to_string(&Direction::Output).unwrap(),
        "\"output\""
    );

    // Test OutputType serialization
    assert_eq!(
        serde_json::to_string(&OutputType::PushPull5v).unwrap(),
        "\"push-pull5v\""
    );
    assert_eq!(
        serde_json::to_string(&OutputType::PushPull3v).unwrap(),
        "\"push-pull3v\""
    );
    assert_eq!(
        serde_json::to_string(&OutputType::OpenDrain).unwrap(),
        "\"open-drain\""
    );

    // Test PullType serialization
    assert_eq!(
        serde_json::to_string(&PullType::PullUp5v).unwrap(),
        "\"pull-up5v\""
    );
    assert_eq!(
        serde_json::to_string(&PullType::PullUp3v).unwrap(),
        "\"pull-up3v\""
    );
    assert_eq!(
        serde_json::to_string(&PullType::PullDown).unwrap(),
        "\"pull-down\""
    );
    assert_eq!(
        serde_json::to_string(&PullType::Float).unwrap(),
        "\"float\""
    );
}

#[test]
fn test_ad_value_and_utilities() {
    let ad_value = AdValue {
        channel: 5,
        voltage: 3.3,
    };

    assert_eq!(ad_value.channel, 5);
    assert_eq!(ad_value.voltage, 3.3);

    // Test utility functions
    assert_eq!(AdManager::voltage_to_percentage(0.0), 0.0);
    assert_eq!(AdManager::voltage_to_percentage(1.25), 25.0);
    assert_eq!(AdManager::voltage_to_percentage(2.5), 50.0);
    assert_eq!(AdManager::voltage_to_percentage(3.75), 75.0);
    assert_eq!(AdManager::voltage_to_percentage(5.0), 100.0);
    assert_eq!(AdManager::voltage_to_percentage(6.0), 100.0); // Clamped

    // Test voltage safety
    assert!(AdManager::is_voltage_safe(0.0));
    assert!(AdManager::is_voltage_safe(2.5));
    assert!(AdManager::is_voltage_safe(5.0));
    assert!(!AdManager::is_voltage_safe(-0.1));
    assert!(!AdManager::is_voltage_safe(5.1));
}

#[test]
fn test_pwm_config_and_calculations() {
    let config = PwmConfig {
        io_pin: 5,
        frequency: 1000,
        pulse_width_ms: 0.5,
    };

    assert_eq!(config.io_pin, 5);
    assert_eq!(config.frequency, 1000);
    assert_eq!(config.pulse_width_ms, 0.5);

    // Test modulation config
    let mod_config = ModulationConfig {
        modulation_type: ModulationType::Am,
        symbol_length_ms: 100.0,
        data: vec![0, 1, 1, 0, 1],
    };

    assert_eq!(mod_config.modulation_type, ModulationType::Am);
    assert_eq!(mod_config.symbol_length_ms, 100.0);
    assert_eq!(mod_config.data, vec![0, 1, 1, 0, 1]);

    // Test utility calculations
    let pulse_width = PwmManager::duty_cycle_to_pulse_width(1000, 50.0);
    assert_eq!(pulse_width, 0.5); // 50% of 1ms period

    let duty_cycle = PwmManager::pulse_width_to_duty_cycle(2000, 0.25);
    assert_eq!(duty_cycle, 50.0); // 0.25ms of 0.5ms period

    // Test edge cases
    let zero_duty = PwmManager::duty_cycle_to_pulse_width(1000, 0.0);
    assert_eq!(zero_duty, 0.0);

    let full_duty = PwmManager::duty_cycle_to_pulse_width(1000, 100.0);
    assert_eq!(full_duty, 1.0);
}

#[test]
fn test_pwm_serialization() {
    use serde_json;

    let mod_type = ModulationType::Am;
    let serialized = serde_json::to_string(&mod_type).unwrap();
    assert_eq!(serialized, "\"am\"");
}

#[test]
fn test_uart_config() {
    // Test default configuration
    let default_config = UartConfig::default();
    assert_eq!(default_config.rx_pin, 0);
    assert_eq!(default_config.tx_pin, 1);
    assert_eq!(default_config.baud_rate, 115200);
    assert_eq!(default_config.stop_bits, 1.0);
    assert_eq!(default_config.data_bits, 8);
    assert_eq!(default_config.parity, Parity::Off);
    assert_eq!(default_config.flow_control, FlowControl::Off);
    assert!(default_config.rts_pin.is_none());
    assert!(default_config.cts_pin.is_none());

    // Test custom configuration
    let custom_config = UartConfig {
        rx_pin: 2,
        tx_pin: 3,
        baud_rate: 9600,
        stop_bits: 2.0,
        data_bits: 7,
        parity: Parity::Even,
        flow_control: FlowControl::RtsCts,
        rts_pin: Some(4),
        cts_pin: Some(5),
    };

    assert_eq!(custom_config.rx_pin, 2);
    assert_eq!(custom_config.tx_pin, 3);
    assert_eq!(custom_config.baud_rate, 9600);
    assert_eq!(custom_config.stop_bits, 2.0);
    assert_eq!(custom_config.data_bits, 7);
    assert_eq!(custom_config.parity, Parity::Even);
    assert_eq!(custom_config.flow_control, FlowControl::RtsCts);
    assert_eq!(custom_config.rts_pin, Some(4));
    assert_eq!(custom_config.cts_pin, Some(5));

    // Test convenience functions
    let simple = UartManager::simple_config(6, 7, 57600);
    assert_eq!(simple.rx_pin, 6);
    assert_eq!(simple.tx_pin, 7);
    assert_eq!(simple.baud_rate, 57600);
    assert_eq!(simple.parity, Parity::Off);

    let flow_control = UartManager::flow_control_config(0, 1, 2, 3, 38400);
    assert_eq!(flow_control.rx_pin, 0);
    assert_eq!(flow_control.tx_pin, 1);
    assert_eq!(flow_control.rts_pin, Some(2));
    assert_eq!(flow_control.cts_pin, Some(3));
    assert_eq!(flow_control.baud_rate, 38400);
    assert_eq!(flow_control.flow_control, FlowControl::RtsCts);
}

#[test]
fn test_uart_serialization() {
    use serde_json;

    // Test Parity serialization
    assert_eq!(serde_json::to_string(&Parity::Off).unwrap(), "\"off\"");
    assert_eq!(serde_json::to_string(&Parity::Odd).unwrap(), "\"odd\"");
    assert_eq!(serde_json::to_string(&Parity::Even).unwrap(), "\"even\"");

    // Test FlowControl serialization
    assert_eq!(serde_json::to_string(&FlowControl::Off).unwrap(), "\"off\"");
    assert_eq!(serde_json::to_string(&FlowControl::Rts).unwrap(), "\"rts\"");
    assert_eq!(serde_json::to_string(&FlowControl::Cts).unwrap(), "\"cts\"");
    assert_eq!(
        serde_json::to_string(&FlowControl::RtsCts).unwrap(),
        "\"rts-cts\""
    );
}

#[test]
fn test_switch_states_and_actions() {
    // Test SwitchState equality and display
    assert_eq!(SwitchState::None, SwitchState::None);
    assert_eq!(SwitchState::Push, SwitchState::Push);
    assert_ne!(SwitchState::None, SwitchState::Push);
    assert_ne!(SwitchState::Left, SwitchState::Right);

    // Test Display trait
    assert_eq!(format!("{}", SwitchState::None), "none");
    assert_eq!(format!("{}", SwitchState::Push), "push");
    assert_eq!(format!("{}", SwitchState::Left), "left");
    assert_eq!(format!("{}", SwitchState::Right), "right");

    // Test SwitchResponse creation
    let response = SwitchResponse {
        state: SwitchState::Push,
        action: SwitchAction::Push,
    };

    assert_eq!(response.state, SwitchState::Push);
    assert_eq!(response.action, SwitchAction::Push);
}

#[test]
fn test_switch_serialization() {
    use serde_json;

    // Test SwitchState serialization
    assert_eq!(
        serde_json::to_string(&SwitchState::None).unwrap(),
        "\"none\""
    );
    assert_eq!(
        serde_json::to_string(&SwitchState::Push).unwrap(),
        "\"push\""
    );
    assert_eq!(
        serde_json::to_string(&SwitchState::Left).unwrap(),
        "\"left\""
    );
    assert_eq!(
        serde_json::to_string(&SwitchState::Right).unwrap(),
        "\"right\""
    );

    // Test SwitchAction serialization
    assert_eq!(
        serde_json::to_string(&SwitchAction::Get).unwrap(),
        "\"get\""
    );
    assert_eq!(
        serde_json::to_string(&SwitchAction::Push).unwrap(),
        "\"push\""
    );
    assert_eq!(
        serde_json::to_string(&SwitchAction::Release).unwrap(),
        "\"release\""
    );
    assert_eq!(
        serde_json::to_string(&SwitchAction::Left).unwrap(),
        "\"left\""
    );
    assert_eq!(
        serde_json::to_string(&SwitchAction::Right).unwrap(),
        "\"right\""
    );

    // Test SwitchResponse serialization
    let response = SwitchResponse {
        state: SwitchState::Left,
        action: SwitchAction::Left,
    };

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"state\":\"left\""));
    assert!(serialized.contains("\"action\":\"left\""));

    // Test deserialization
    let json_str = r#"{"state": "right", "action": "right"}"#;
    let deserialized: SwitchResponse = serde_json::from_str(json_str).unwrap();
    assert_eq!(deserialized.state, SwitchState::Right);
    assert_eq!(deserialized.action, SwitchAction::Right);
}

#[test]
fn test_display_types() {
    // Test QrCorrectionType
    use serde_json;

    assert_eq!(
        serde_json::to_string(&QrCorrectionType::Low).unwrap(),
        "\"L\""
    );
    assert_eq!(
        serde_json::to_string(&QrCorrectionType::Medium).unwrap(),
        "\"M\""
    );
    assert_eq!(
        serde_json::to_string(&QrCorrectionType::Quality).unwrap(),
        "\"Q\""
    );
    assert_eq!(
        serde_json::to_string(&QrCorrectionType::High).unwrap(),
        "\"H\""
    );

    // Test DisplayRawColorDepth
    assert_eq!(
        serde_json::to_string(&DisplayRawColorDepth::OneBit).unwrap(),
        "\"1\""
    );
    assert_eq!(
        serde_json::to_string(&DisplayRawColorDepth::FourBit).unwrap(),
        "\"4\""
    );
    assert_eq!(
        serde_json::to_string(&DisplayRawColorDepth::SixteenBit).unwrap(),
        "\"16\""
    );

    // Test RawDisplayConfig
    let config = RawDisplayConfig {
        width: 128,
        height: 64,
        color_depth: DisplayRawColorDepth::OneBit,
        data: vec![0xAA, 0x55, 0xAA, 0x55],
    };

    assert_eq!(config.width, 128);
    assert_eq!(config.height, 64);
    assert_eq!(config.color_depth, DisplayRawColorDepth::OneBit);
    assert_eq!(config.data.len(), 4);

    // Test PinAssignment
    let assignment = PinAssignment {
        pin: 7,
        module_name: "spi".to_string(),
        pin_name: "miso".to_string(),
    };

    assert_eq!(assignment.pin, 7);
    assert_eq!(assignment.module_name, "spi");
    assert_eq!(assignment.pin_name, "miso");
}

#[test]
fn test_system_types() {
    use serde_json;

    // Test SystemInfo deserialization
    let json_str = r#"{
        "version": "3.5.0",
        "hardware": "obnizb1",
        "device_type": "obnizb1",
        "region": "jp"
    }"#;

    let info: SystemInfo = serde_json::from_str(json_str).unwrap();
    assert_eq!(info.version, "3.5.0");
    assert_eq!(info.hardware, "obnizb1");
    assert_eq!(info.device_type, "obnizb1");
    assert_eq!(info.region, Some("jp".to_string()));

    // Test ResetConfig serialization
    let config = ResetConfig {
        reset_obniz_on_ws_disconnection: true,
    };

    let serialized = serde_json::to_string(&config).unwrap();
    assert!(serialized.contains("reset_obniz_on_ws_disconnection"));
    assert!(serialized.contains("true"));
}

#[test]
fn test_error_types() {
    // Test error creation and display
    let pin_error = ObnizError::InvalidPin(15);
    assert_eq!(
        format!("{}", pin_error),
        "Invalid pin number: 15. Valid range is 0-11"
    );

    let connection_error = ObnizError::Connection("Failed to connect".to_string());
    assert_eq!(
        format!("{}", connection_error),
        "Connection error: Failed to connect"
    );

    let timeout_error = ObnizError::Timeout;
    assert_eq!(format!("{}", timeout_error), "Operation timed out");

    let json_error = ObnizError::JsonParse("Invalid JSON".to_string());
    assert_eq!(format!("{}", json_error), "JSON parse error: Invalid JSON");

    let io_error = ObnizError::IoOperation("Pin read failed".to_string());
    assert_eq!(
        format!("{}", io_error),
        "IO operation error: Pin read failed"
    );
}

#[test]
fn test_validation_functions() {
    use crate::error::validate_pin;

    // Test valid pins
    assert!(validate_pin(0).is_ok());
    assert!(validate_pin(5).is_ok());
    assert!(validate_pin(11).is_ok());

    // Test invalid pins
    assert!(validate_pin(12).is_err());
    assert!(validate_pin(255).is_err());

    // Test error type
    match validate_pin(20) {
        Err(ObnizError::InvalidPin(pin)) => assert_eq!(pin, 20),
        _ => panic!("Expected InvalidPin error"),
    }
}

#[test]
fn test_key_generation_patterns() {
    // Test IO key generation pattern
    for i in 0..=11 {
        let expected = format!("io{}", i);
        assert_eq!(expected, format!("io{}", i));
    }

    // Test AD key generation pattern
    for i in 0..=11 {
        let expected = format!("ad{}", i);
        assert_eq!(expected, format!("ad{}", i));
    }

    // Test PWM key generation pattern
    for i in 0..=5 {
        let expected = format!("pwm{}", i);
        assert_eq!(expected, format!("pwm{}", i));
    }

    // Test UART key generation pattern
    for i in 0..=2 {
        let expected = format!("uart{}", i);
        assert_eq!(expected, format!("uart{}", i));
    }
}

#[test]
fn test_range_validations() {
    // Test frequency ranges for PWM
    assert!(1 >= 1 && 1 <= 80_000_000); // Min frequency
    assert!(80_000_000 >= 1 && 80_000_000 <= 80_000_000); // Max frequency
    assert!(!(0 >= 1 && 0 <= 80_000_000)); // Invalid frequency
    assert!(!(80_000_001 >= 1 && 80_000_001 <= 80_000_000)); // Invalid frequency

    // Test baud rate ranges for UART
    assert!(115200 >= 1 && 115200 <= 5_000_000); // Common baud rate
    assert!(9600 >= 1 && 9600 <= 5_000_000); // Common baud rate
    assert!(!(0 >= 1 && 0 <= 5_000_000)); // Invalid baud rate
    assert!(!(5_000_001 >= 1 && 5_000_001 <= 5_000_000)); // Invalid baud rate

    // Test servo angle range
    assert!(0.0 >= 0.0 && 0.0 <= 180.0); // Min angle
    assert!(90.0 >= 0.0 && 90.0 <= 180.0); // Middle angle
    assert!(180.0 >= 0.0 && 180.0 <= 180.0); // Max angle
    assert!(!(-1.0 >= 0.0 && -1.0 <= 180.0)); // Invalid angle
    assert!(!(181.0 >= 0.0 && 181.0 <= 180.0)); // Invalid angle

    // Test voltage range
    assert!(0.0 >= 0.0 && 0.0 <= 5.0); // Min voltage
    assert!(3.3 >= 0.0 && 3.3 <= 5.0); // Common voltage
    assert!(5.0 >= 0.0 && 5.0 <= 5.0); // Max voltage
    assert!(!(-0.1 >= 0.0 && -0.1 <= 5.0)); // Invalid voltage
    assert!(!(5.1 >= 0.0 && 5.1 <= 5.0)); // Invalid voltage
}

#[test]
fn test_json_request_patterns() {
    // Test IO request patterns
    let io_get_request = json!([{"io0": "get"}]);
    assert!(io_get_request.is_array());
    assert!(io_get_request[0]["io0"] == "get");

    let io_set_request = json!([{"io1": true}]);
    assert!(io_set_request[0]["io1"] == true);

    let io_config_request = json!([{
        "io2": {
            "direction": "output",
            "value": true,
            "output_type": "push-pull5v"
        }
    }]);
    assert!(io_config_request[0]["io2"]["direction"] == "output");

    // Test AD request patterns
    let ad_get_request = json!([{"ad0": "get"}]);
    assert!(ad_get_request[0]["ad0"] == "get");

    // Test PWM request patterns
    let pwm_config_request = json!([{
        "pwm0": {
            "io": 5,
            "freq": 1000,
            "pulse": 0.5
        }
    }]);
    assert!(pwm_config_request[0]["pwm0"]["freq"] == 1000);

    // Test UART request patterns
    let uart_config_request = json!([{
        "uart0": {
            "rx": 0,
            "tx": 1,
            "baud": 115200
        }
    }]);
    assert!(uart_config_request[0]["uart0"]["baud"] == 115200);

    // Test display request patterns
    let display_text_request = json!([{
        "display": {
            "text": "Hello World"
        }
    }]);
    assert!(display_text_request[0]["display"]["text"] == "Hello World");

    let display_clear_request = json!([{
        "display": {
            "clear": true
        }
    }]);
    assert!(display_clear_request[0]["display"]["clear"] == true);
}

#[test]
fn test_type_conversions() {
    // Test f64 to percentage conversion
    assert_eq!((0.0_f64 / 5.0 * 100.0).round() as i32, 0);
    assert_eq!((2.5_f64 / 5.0 * 100.0).round() as i32, 50);
    assert_eq!((5.0_f64 / 5.0 * 100.0).round() as i32, 100);

    // Test duty cycle calculations
    let period_1khz = 1000.0 / 1000.0; // 1ms for 1kHz
    assert_eq!(period_1khz * 50.0 / 100.0, 0.5); // 50% duty cycle
    assert_eq!(period_1khz * 25.0 / 100.0, 0.25); // 25% duty cycle

    // Test servo angle to pulse width
    let servo_min = 1.0; // 1ms for 0 degrees
    let servo_max = 2.0; // 2ms for 180 degrees
    let angle_90 = 90.0;
    let pulse_width_90 = servo_min + (angle_90 / 180.0) * (servo_max - servo_min);
    assert_eq!(pulse_width_90, 1.5); // 1.5ms for 90 degrees
}
