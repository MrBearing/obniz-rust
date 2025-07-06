use obniz_rust::*;

#[tokio::test]
async fn test_error_validation() {
    // Test pin validation without actual connection
    let result = validate_pin(15);
    assert!(result.is_err());

    match result {
        Err(ObnizError::InvalidPin(pin)) => assert_eq!(pin, 15),
        _ => panic!("Expected InvalidPin error"),
    }
}

#[tokio::test]
async fn test_configuration_structures() {
    // Test IoConfig creation and validation
    let io_config = IoConfig {
        direction: Direction::Output,
        value: Some(true),
        output_type: Some(OutputType::PushPull5v),
        pull_type: Some(PullType::PullUp5v),
        stream: Some(false),
    };

    assert_eq!(io_config.direction, Direction::Output);
    assert_eq!(io_config.value, Some(true));

    // Test PwmConfig creation
    let pwm_config = PwmConfig {
        io_pin: 5,
        frequency: 1000,
        pulse_width_ms: 0.5,
    };

    assert_eq!(pwm_config.io_pin, 5);
    assert_eq!(pwm_config.frequency, 1000);
    assert_eq!(pwm_config.pulse_width_ms, 0.5);

    // Test UartConfig
    let uart_config = UartManager::simple_config(0, 1, 115200);
    assert_eq!(uart_config.rx_pin, 0);
    assert_eq!(uart_config.tx_pin, 1);
    assert_eq!(uart_config.baud_rate, 115200);
}

#[tokio::test]
async fn test_utility_calculations() {
    // Test AD utility functions
    assert_eq!(AdManager::voltage_to_percentage(0.0), 0.0);
    assert_eq!(AdManager::voltage_to_percentage(2.5), 50.0);
    assert_eq!(AdManager::voltage_to_percentage(5.0), 100.0);
    assert_eq!(AdManager::voltage_to_percentage(6.0), 100.0); // Clamped

    assert!(AdManager::is_voltage_safe(3.3));
    assert!(!AdManager::is_voltage_safe(-0.1));
    assert!(!AdManager::is_voltage_safe(5.1));

    // Test PWM utility functions
    let pulse_width = PwmManager::duty_cycle_to_pulse_width(1000, 25.0);
    assert_eq!(pulse_width, 0.25);

    let duty_cycle = PwmManager::pulse_width_to_duty_cycle(1000, 0.5);
    assert_eq!(duty_cycle, 50.0);
}

#[tokio::test]
async fn test_json_serialization() {
    use serde_json;

    // Test enum serialization
    assert_eq!(
        serde_json::to_string(&Direction::Input).unwrap(),
        "\"input\""
    );
    assert_eq!(
        serde_json::to_string(&Direction::Output).unwrap(),
        "\"output\""
    );

    assert_eq!(
        serde_json::to_string(&OutputType::PushPull5v).unwrap(),
        "\"push-pull5v\""
    );
    assert_eq!(
        serde_json::to_string(&OutputType::OpenDrain).unwrap(),
        "\"open-drain\""
    );

    assert_eq!(
        serde_json::to_string(&PullType::PullUp5v).unwrap(),
        "\"pull-up5v\""
    );
    assert_eq!(
        serde_json::to_string(&PullType::Float).unwrap(),
        "\"float\""
    );

    assert_eq!(serde_json::to_string(&Parity::Off).unwrap(), "\"off\"");
    assert_eq!(serde_json::to_string(&Parity::Even).unwrap(), "\"even\"");

    assert_eq!(serde_json::to_string(&FlowControl::Off).unwrap(), "\"off\"");
    assert_eq!(
        serde_json::to_string(&FlowControl::RtsCts).unwrap(),
        "\"rts-cts\""
    );

    assert_eq!(
        serde_json::to_string(&SwitchState::None).unwrap(),
        "\"none\""
    );
    assert_eq!(
        serde_json::to_string(&SwitchState::Push).unwrap(),
        "\"push\""
    );

    assert_eq!(
        serde_json::to_string(&QrCorrectionType::Low).unwrap(),
        "\"L\""
    );
    assert_eq!(
        serde_json::to_string(&QrCorrectionType::High).unwrap(),
        "\"H\""
    );
}

#[tokio::test]
async fn test_error_types() {
    // Test different error types
    let pin_error = ObnizError::InvalidPin(15);
    assert!(format!("{}", pin_error).contains("Invalid pin number: 15"));

    let connection_error = ObnizError::Connection("Failed to connect".to_string());
    assert!(format!("{}", connection_error).contains("Connection error"));

    let timeout_error = ObnizError::Timeout;
    assert_eq!(format!("{}", timeout_error), "Operation timed out");

    let json_error = ObnizError::JsonParse("Invalid JSON".to_string());
    assert!(format!("{}", json_error).contains("JSON parse error"));
}
