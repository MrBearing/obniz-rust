use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::error::{validate_pin, ObnizError, ObnizResult};
use crate::obniz::Obniz;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputType {
    PushPull5v,
    PushPull3v,
    OpenDrain,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PullType {
    PullUp5v,
    PullUp3v,
    PullDown,
    Float,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub struct IoConfig {
    pub direction: Direction,
    pub value: Option<bool>,
    pub output_type: Option<OutputType>,
    pub pull_type: Option<PullType>,
    pub stream: Option<bool>,
}

#[derive(Debug)]
pub struct IoPin {
    pin: u8,
    obniz: Obniz,
}

impl IoPin {
    pub fn new(pin: u8, obniz: Obniz) -> Self {
        Self { pin, obniz }
    }

    pub fn pin_key(&self) -> String {
        format!("io{}", self.pin)
    }

    /// Get the current state of the pin
    pub async fn get(&self) -> ObnizResult<bool> {
        validate_pin(self.pin)?;

        let pin_key = self.pin_key();
        let request = json!([{&pin_key: "get"}]);
        let message = Message::from(request.to_string());

        let response = self
            .obniz
            .send_await_response(message, pin_key.clone())
            .await
            .map_err(|e| ObnizError::Connection(e.to_string()))?;

        // Parse the response to extract the boolean value
        // Response format is typically [{"io1": false}]
        if let Some(array) = response.as_array() {
            if let Some(first_item) = array.first() {
                if let Some(value) = first_item.get(&pin_key) {
                    return value.as_bool().ok_or_else(|| {
                        ObnizError::IoOperation("Invalid response format".to_string())
                    });
                }
            }
        }

        // Fallback: try direct object access
        if let Some(value) = response.get(&pin_key) {
            value
                .as_bool()
                .ok_or_else(|| ObnizError::IoOperation("Invalid response format".to_string()))
        } else {
            Err(ObnizError::IoOperation(format!(
                "No response for pin {}",
                self.pin
            )))
        }
    }

    /// Set the pin to a specific value
    pub async fn set(&self, value: bool) -> ObnizResult<()> {
        validate_pin(self.pin)?;

        let pin_key = self.pin_key();
        let request = json!([{&pin_key: value}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))?;
        Ok(())
    }

    /// Configure the pin with detailed settings
    pub async fn configure(&self, config: IoConfig) -> ObnizResult<()> {
        validate_pin(self.pin)?;
        let pin_key = self.pin_key();
        let mut pin_config = json!({
            "direction": config.direction
        });

        if let Some(value) = config.value {
            pin_config["value"] = json!(value);
        }

        if let Some(output_type) = config.output_type {
            pin_config["output_type"] = json!(output_type);
        }

        if let Some(pull_type) = config.pull_type {
            pin_config["pull_type"] = json!(pull_type);
        }

        if let Some(stream) = config.stream {
            pin_config["stream"] = json!(stream);
        }

        let request = json!([{&pin_key: pin_config}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))?;
        Ok(())
    }

    /// Set the pin as input with optional stream mode
    pub async fn set_as_input(&self, enable_stream: bool) -> ObnizResult<()> {
        validate_pin(self.pin)?;
        let config = IoConfig {
            direction: Direction::Input,
            value: None,
            output_type: None,
            pull_type: None,
            stream: Some(enable_stream),
        };
        self.configure(config).await
    }

    /// Set the pin as output with a specific value
    pub async fn set_as_output(&self, value: bool) -> ObnizResult<()> {
        validate_pin(self.pin)?;
        let config = IoConfig {
            direction: Direction::Output,
            value: Some(value),
            output_type: None,
            pull_type: None,
            stream: None,
        };
        self.configure(config).await
    }

    /// Set the output type of the pin
    pub async fn set_output_type(&self, output_type: OutputType) -> ObnizResult<()> {
        validate_pin(self.pin)?;
        let pin_key = self.pin_key();
        let request = json!([{&pin_key: {"output_type": output_type}}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))?;
        Ok(())
    }

    /// Set the pull type of the pin
    pub async fn set_pull_type(&self, pull_type: PullType) -> ObnizResult<()> {
        validate_pin(self.pin)?;
        let pin_key = self.pin_key();
        let request = json!([{&pin_key: {"pull_type": pull_type}}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))?;
        Ok(())
    }

    /// Register a callback for this pin's state changes (stream mode)
    /// This will automatically enable stream mode for the pin
    pub async fn on_change<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        validate_pin(self.pin)?;
        // First, enable stream mode for this pin
        self.set_as_input(true).await?;

        let pin_key = self.pin_key();
        let pin_key_clone = pin_key.clone();

        self.obniz
            .register_callback(pin_key, move |response| {
                // Parse the response to extract the pin value
                if let Some(value) = response.get(&pin_key_clone) {
                    if let Some(bool_value) = value.as_bool() {
                        callback(bool_value);
                    }
                }
            })
            .map_err(|e| ObnizError::CallbackError(e.to_string()))?;

        Ok(())
    }

    /// Enable stream mode for this pin without setting up a callback
    pub async fn enable_stream(&self) -> ObnizResult<()> {
        self.set_as_input(true).await
    }

    /// Disable stream mode for this pin
    pub async fn disable_stream(&self) -> ObnizResult<()> {
        self.set_as_input(false).await
    }

    /// Remove the callback for this pin
    pub fn remove_callback(&self) -> ObnizResult<()> {
        validate_pin(self.pin)?;
        let pin_key = self.pin_key();
        self.obniz
            .unregister_callback(pin_key)
            .map_err(|e| ObnizError::CallbackError(e.to_string()))
    }

    /// Deinitialize the pin
    pub async fn deinit(&self) -> ObnizResult<()> {
        validate_pin(self.pin)?;
        let pin_key = self.pin_key();
        let request = json!([{&pin_key: null}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))?;
        Ok(())
    }
}

/// IO Manager for handling multiple pins
#[derive(Debug)]
pub struct IoManager {
    obniz: Obniz,
}

impl IoManager {
    pub fn new(obniz: Obniz) -> Self {
        Self { obniz }
    }

    /// Get a specific pin (0-11)
    pub fn pin(&self, pin: u8) -> ObnizResult<IoPin> {
        validate_pin(pin)?;
        Ok(IoPin::new(pin, self.obniz.clone()))
    }

    /// Get the current state of a pin
    pub async fn get_pin(&self, pin: u8) -> ObnizResult<bool> {
        self.pin(pin)?.get().await
    }

    /// Set a pin to a specific value
    pub async fn set_pin(&self, pin: u8, value: bool) -> ObnizResult<()> {
        self.pin(pin)?.set(value).await
    }

    /// Configure a pin with detailed settings
    pub async fn configure_pin(&self, pin: u8, config: IoConfig) -> ObnizResult<()> {
        self.pin(pin)?.configure(config).await
    }

    /// Set a pin as input with optional stream mode
    pub async fn set_pin_as_input(&self, pin: u8, enable_stream: bool) -> ObnizResult<()> {
        self.pin(pin)?.set_as_input(enable_stream).await
    }

    /// Set a pin as output with a specific value
    pub async fn set_pin_as_output(&self, pin: u8, value: bool) -> ObnizResult<()> {
        self.pin(pin)?.set_as_output(value).await
    }

    /// Register a callback for a pin's state changes (stream mode)
    /// This will automatically enable stream mode for the pin
    pub async fn set_pin_callback<F>(&self, pin: u8, callback: F) -> ObnizResult<()>
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.pin(pin)?.on_change(callback).await
    }

    /// Remove the callback for a pin
    pub fn remove_pin_callback(&self, pin: u8) -> ObnizResult<()> {
        self.pin(pin)?.remove_callback()
    }

    /// Enable stream mode for a pin without setting up a callback
    pub async fn enable_pin_stream(&self, pin: u8) -> ObnizResult<()> {
        self.pin(pin)?.enable_stream().await
    }

    /// Disable stream mode for a pin
    pub async fn disable_pin_stream(&self, pin: u8) -> ObnizResult<()> {
        self.pin(pin)?.disable_stream().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::validate_pin;

    #[test]
    fn test_pin_validation() {
        // Valid pins
        assert!(validate_pin(0).is_ok());
        assert!(validate_pin(5).is_ok());
        assert!(validate_pin(11).is_ok());

        // Invalid pins
        assert!(validate_pin(12).is_err());
        assert!(validate_pin(255).is_err());
    }

    // #[test]
    // fn test_pin_key_generation() {
    //     let obniz = create_mock_obniz();
    //     let pin = IoPin::new(5, obniz);
    //     assert_eq!(pin.pin_key(), "io5");
    //
    //     let pin = IoPin::new(0, obniz);
    //     assert_eq!(pin.pin_key(), "io0");
    // }

    #[test]
    fn test_io_config_serialization() {
        let config = IoConfig {
            direction: Direction::Output,
            value: Some(true),
            output_type: Some(OutputType::PushPull5v),
            pull_type: Some(PullType::PullUp5v),
            stream: Some(false),
        };

        // Test that the config can be created without errors
        assert_eq!(config.direction, Direction::Output);
        assert_eq!(config.value, Some(true));
    }

    #[test]
    fn test_output_type_serialization() {
        use serde_json;

        let output_type = OutputType::PushPull5v;
        let serialized = serde_json::to_string(&output_type).unwrap();
        assert_eq!(serialized, "\"push-pull5v\"");

        let output_type = OutputType::OpenDrain;
        let serialized = serde_json::to_string(&output_type).unwrap();
        assert_eq!(serialized, "\"open-drain\"");
    }

    #[test]
    fn test_pull_type_serialization() {
        use serde_json;

        let pull_type = PullType::PullUp5v;
        let serialized = serde_json::to_string(&pull_type).unwrap();
        assert_eq!(serialized, "\"pull-up5v\"");

        let pull_type = PullType::Float;
        let serialized = serde_json::to_string(&pull_type).unwrap();
        assert_eq!(serialized, "\"float\"");
    }

    #[test]
    fn test_direction_serialization() {
        use serde_json;

        let direction = Direction::Input;
        let serialized = serde_json::to_string(&direction).unwrap();
        assert_eq!(serialized, "\"input\"");

        let direction = Direction::Output;
        let serialized = serde_json::to_string(&direction).unwrap();
        assert_eq!(serialized, "\"output\"");
    }

    // Helper function to create a mock Obniz instance for testing
    // Note: This would need to be implemented with proper mocking
    // fn create_mock_obniz() -> Obniz {
    //     // This is a placeholder - in a real implementation, you'd use a mock
    //     // or test framework to create a mock Obniz instance
    //     unimplemented!("Mock Obniz creation for testing")
    // }
}
