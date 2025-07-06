use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::error::{ObnizError, ObnizResult};
use crate::obniz::{CallbackType, ObnizCommand};

/// Mock WebSocket message for testing
#[derive(Debug, Clone)]
pub struct MockMessage {
    pub request: Value,
    pub response: Value,
    pub delay_ms: Option<u64>,
}

/// Mock WebSocket behavior configuration
#[derive(Debug, Clone)]
pub struct MockConfig {
    pub device_id: String,
    pub should_fail_connection: bool,
    pub should_timeout: bool,
    pub default_delay_ms: u64,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            device_id: "mock-device".to_string(),
            should_fail_connection: false,
            should_timeout: false,
            default_delay_ms: 10,
        }
    }
}

/// Mock WebSocket server for testing
#[derive(Debug)]
pub struct MockWebSocketServer {
    config: MockConfig,
    message_handlers: Arc<Mutex<HashMap<String, MockMessage>>>,
    sent_messages: Arc<Mutex<Vec<Value>>>,
    callbacks: Arc<Mutex<HashMap<String, CallbackType>>>,
}

impl MockWebSocketServer {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            message_handlers: Arc::new(Mutex::new(HashMap::new())),
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a mock response for a specific request pattern
    pub fn add_response(&self, request_key: &str, response: Value) {
        let mock_msg = MockMessage {
            request: json!({}),
            response,
            delay_ms: Some(self.config.default_delay_ms),
        };
        self.message_handlers
            .lock()
            .unwrap()
            .insert(request_key.to_string(), mock_msg);
    }

    /// Add a mock response with custom delay
    pub fn add_response_with_delay(&self, request_key: &str, response: Value, delay_ms: u64) {
        let mock_msg = MockMessage {
            request: json!({}),
            response,
            delay_ms: Some(delay_ms),
        };
        self.message_handlers
            .lock()
            .unwrap()
            .insert(request_key.to_string(), mock_msg);
    }

    /// Get all sent messages for verification
    pub fn get_sent_messages(&self) -> Vec<Value> {
        self.sent_messages.lock().unwrap().clone()
    }

    /// Clear sent message history
    pub fn clear_sent_messages(&self) {
        self.sent_messages.lock().unwrap().clear();
    }

    /// Mock WebSocket message processing
    pub async fn process_message(&self, message: Message) -> ObnizResult<Option<Value>> {
        if self.config.should_timeout {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }

        let text = message
            .to_text()
            .map_err(|_| ObnizError::Generic("Invalid message".to_string()))?;
        let request: Value =
            serde_json::from_str(text).map_err(|e| ObnizError::JsonParse(e.to_string()))?;

        // Store sent message
        self.sent_messages.lock().unwrap().push(request.clone());

        // Find matching response
        let response = self.find_mock_response(&request);

        if let Some(mock_msg) = response {
            if let Some(delay) = mock_msg.delay_ms {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }
            Ok(Some(mock_msg.response))
        } else {
            // Default responses for common patterns
            Ok(Some(self.generate_default_response(&request)))
        }
    }

    fn find_mock_response(&self, request: &Value) -> Option<MockMessage> {
        let handlers = self.message_handlers.lock().unwrap();

        // Try to match based on the request structure
        if let Some(array) = request.as_array() {
            if let Some(first_item) = array.first() {
                if let Some(obj) = first_item.as_object() {
                    for (key, _) in obj {
                        if let Some(mock_msg) = handlers.get(key) {
                            return Some(mock_msg.clone());
                        }

                        // Check for nested keys
                        if let Some(nested) = first_item.get(key) {
                            if let Some(nested_obj) = nested.as_object() {
                                for (nested_key, _) in nested_obj {
                                    let compound_key = format!("{key}.{nested_key}");
                                    if let Some(mock_msg) = handlers.get(&compound_key) {
                                        return Some(mock_msg.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn generate_default_response(&self, request: &Value) -> Value {
        if let Some(array) = request.as_array() {
            if let Some(first_item) = array.first() {
                if let Some(obj) = first_item.as_object() {
                    for (key, value) in obj {
                        match key.as_str() {
                            // IO responses
                            k if k.starts_with("io") => {
                                if value == "get" {
                                    return json!([{k: false}]); // Default pin state
                                } else {
                                    return json!([{k: {"state": "ok"}}]);
                                }
                            }
                            // AD responses
                            k if k.starts_with("ad") => {
                                if value == "get" {
                                    return json!([{k: 3.3}]); // Default voltage
                                } else {
                                    return json!([{k: {"state": "ok"}}]);
                                }
                            }
                            // PWM responses
                            k if k.starts_with("pwm") => {
                                return json!([{k: {"state": "ok"}}]);
                            }
                            // UART responses
                            k if k.starts_with("uart") => {
                                return json!([{k: {"state": "ok"}}]);
                            }
                            // Display responses
                            "display" => {
                                return json!([{"display": {"state": "ok"}}]);
                            }
                            // System responses
                            "system" => {
                                if let Some(system_obj) = value.as_object() {
                                    if system_obj.contains_key("info") {
                                        return json!([{
                                            "system": {
                                                "version": "3.5.0",
                                                "hardware": "mock",
                                                "device_type": "mock",
                                                "region": "test"
                                            }
                                        }]);
                                    }
                                }
                                return json!([{"system": {"state": "ok"}}]);
                            }
                            // Switch responses
                            "switch" => {
                                if value == "get" {
                                    return json!([{
                                        "switch": {
                                            "state": "none",
                                            "action": "get"
                                        }
                                    }]);
                                }
                                return json!([{"switch": {"state": "ok"}}]);
                            }
                            // WebSocket responses
                            "ws" => {
                                return json!([{
                                    "ws": {
                                        "ready": true,
                                        "obniz": {
                                            "hw": "mock",
                                            "firmware": "test",
                                            "connected_network": {
                                                "online_at": 1640995200,
                                                "wifi": {
                                                    "ssid": "test-wifi"
                                                }
                                            }
                                        }
                                    }
                                }]);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Default success response
        json!([{"status": "ok"}])
    }

    /// Simulate callback events
    pub async fn trigger_callback(&self, key: &str, data: Value) {
        if let Some(callback) = self.callbacks.lock().unwrap().get(key) {
            match callback {
                CallbackType::Persistent(callback_fn) => {
                    callback_fn(data);
                }
                CallbackType::OneShot(_) => {
                    // OneShot callbacks are harder to trigger in tests
                    // They would be consumed on first use
                }
            }
        }
    }
}

/// Mock Obniz device for testing
#[derive(Debug)]
pub struct MockObniz {
    device_id: String,
    server: Arc<MockWebSocketServer>,
    command_sender: mpsc::UnboundedSender<ObnizCommand>,
    #[allow(dead_code)] // Reserved for future mock command processing
    command_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<ObnizCommand>>>>,
}

impl MockObniz {
    pub fn new(config: MockConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let server = Arc::new(MockWebSocketServer::new(config.clone()));

        Self {
            device_id: config.device_id,
            server,
            command_sender: tx,
            command_receiver: Arc::new(Mutex::new(Some(rx))),
        }
    }

    pub fn server(&self) -> Arc<MockWebSocketServer> {
        self.server.clone()
    }

    pub async fn send_message(&self, message: Message) -> ObnizResult<()> {
        self.command_sender
            .send(ObnizCommand::Send {
                message,
                response_key: None,
            })
            .map_err(|e| ObnizError::Connection(e.to_string()))?;
        Ok(())
    }

    pub async fn send_await_response(
        &self,
        message: Message,
        _response_key: String,
    ) -> ObnizResult<Value> {
        // Process message through mock server
        if let Some(response) = self.server.process_message(message).await? {
            Ok(response)
        } else {
            Err(ObnizError::Timeout)
        }
    }

    pub fn register_callback<F>(&self, key: String, callback: F) -> ObnizResult<()>
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        self.server
            .callbacks
            .lock()
            .unwrap()
            .insert(key, CallbackType::Persistent(Box::new(callback)));
        Ok(())
    }

    pub fn unregister_callback(&self, key: String) -> ObnizResult<()> {
        self.server.callbacks.lock().unwrap().remove(&key);
        Ok(())
    }

    pub fn id(&self) -> &str {
        &self.device_id
    }
}

/// Helper functions for creating mock responses
pub mod responses {
    use super::*;

    pub fn io_pin_state(pin: u8, state: bool) -> Value {
        json!([{format!("io{}", pin): state}])
    }

    pub fn ad_voltage(channel: u8, voltage: f64) -> Value {
        json!([{format!("ad{}", channel): voltage}])
    }

    pub fn pwm_ok(channel: u8) -> Value {
        json!([{format!("pwm{}", channel): {"state": "ok"}}])
    }

    pub fn uart_data(channel: u8, data: Vec<u8>) -> Value {
        json!([{format!("uart{}", channel): {"data": data}}])
    }

    pub fn switch_state(state: &str, action: &str) -> Value {
        json!([{
            "switch": {
                "state": state,
                "action": action
            }
        }])
    }

    pub fn display_ok() -> Value {
        json!([{"display": {"state": "ok"}}])
    }

    pub fn system_info() -> Value {
        json!([{
            "system": {
                "version": "3.5.0",
                "hardware": "mock",
                "device_type": "mock",
                "region": "test"
            }
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_creation() {
        let config = MockConfig::default();
        let server = MockWebSocketServer::new(config);

        assert_eq!(server.config.device_id, "mock-device");
        assert!(!server.config.should_fail_connection);
    }

    #[tokio::test]
    async fn test_mock_responses() {
        let config = MockConfig::default();
        let server = MockWebSocketServer::new(config);

        // Add custom response
        server.add_response("io0", responses::io_pin_state(0, true));

        // Test message processing
        let request = json!([{"io0": "get"}]);
        let message = Message::from(request.to_string());

        let response = server.process_message(message).await.unwrap();
        assert!(response.is_some());

        let response_value = response.unwrap();
        assert_eq!(response_value, responses::io_pin_state(0, true));
    }

    #[tokio::test]
    async fn test_default_responses() {
        let config = MockConfig::default();
        let server = MockWebSocketServer::new(config);

        // Test default IO response
        let request = json!([{"io0": "get"}]);
        let message = Message::from(request.to_string());

        let response = server.process_message(message).await.unwrap();
        assert!(response.is_some());

        // Should return default pin state (false)
        let response_value = response.unwrap();
        assert_eq!(response_value, json!([{"io0": false}]));
    }

    #[tokio::test]
    async fn test_sent_message_tracking() {
        let config = MockConfig::default();
        let server = MockWebSocketServer::new(config);

        let request1 = json!([{"io0": true}]);
        let request2 = json!([{"io1": false}]);

        let message1 = Message::from(request1.to_string());
        let message2 = Message::from(request2.to_string());

        server.process_message(message1).await.unwrap();
        server.process_message(message2).await.unwrap();

        let sent_messages = server.get_sent_messages();
        assert_eq!(sent_messages.len(), 2);
        assert_eq!(sent_messages[0], request1);
        assert_eq!(sent_messages[1], request2);
    }
}
