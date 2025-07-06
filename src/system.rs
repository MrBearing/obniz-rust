use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::obniz::Obniz;
use crate::error::{ObnizError, ObnizResult};

/// System information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub hardware: String,
    pub device_type: String,
    pub region: Option<String>,
}

/// Reset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetConfig {
    pub reset_obniz_on_ws_disconnection: bool,
}

/// System manager for obniz device control
#[derive(Debug, Clone)]
pub struct SystemManager {
    obniz: Obniz,
}

impl SystemManager {
    pub fn new(obniz: Obniz) -> Self {
        Self { obniz }
    }

    /// Reset the obniz device
    pub async fn reset(&self) -> ObnizResult<()> {
        let request = json!([{"system": {"reset": true}}]);
        let message = Message::from(request.to_string());
        
        self.obniz.send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Get system information
    pub async fn info(&self) -> ObnizResult<SystemInfo> {
        let request = json!([{"system": {"info": "get"}}]);
        let message = Message::from(request.to_string());
        
        let response = self.obniz.send_await_response(message, "system".to_string()).await
            .map_err(|e| ObnizError::Connection(e.to_string()))?;
        
        // Parse the response to extract the system information
        // Response format is typically [{"system": {...}}]
        if let Some(array) = response.as_array() {
            if let Some(first_item) = array.first() {
                if let Some(system_info) = first_item.get("system") {
                    return serde_json::from_value(system_info.clone())
                        .map_err(|e| ObnizError::JsonParse(e.to_string()));
                }
            }
        }
        
        // Fallback: try direct object access
        if let Some(system_info) = response.get("system") {
            serde_json::from_value(system_info.clone())
                .map_err(|e| ObnizError::JsonParse(e.to_string()))
        } else {
            Err(ObnizError::Generic("No system information in response".to_string()))
        }
    }

    /// Configure reset behavior on WebSocket disconnection
    pub async fn reset_on_disconnect(&self, enable: bool) -> ObnizResult<()> {
        let request = json!([{"ws": {"reset_obniz_on_ws_disconnection": enable}}]);
        let message = Message::from(request.to_string());
        
        self.obniz.send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Keep alive - prevent auto sleep
    pub async fn keep_alive(&self) -> ObnizResult<()> {
        let request = json!([{"system": {"keep_working_at_offline": true}}]);
        let message = Message::from(request.to_string());
        
        self.obniz.send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Set ping interval for connection monitoring
    pub async fn ping_interval(&self, interval_ms: u32) -> ObnizResult<()> {
        if interval_ms == 0 {
            return Err(ObnizError::Generic("Ping interval must be greater than 0".to_string()));
        }

        let request = json!([{"ws": {"ping": {"interval": interval_ms}}}]);
        let message = Message::from(request.to_string());
        
        self.obniz.send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Get device status
    pub async fn status(&self) -> ObnizResult<serde_json::Value> {
        let request = json!([{"system": {"status": "get"}}]);
        let message = Message::from(request.to_string());
        
        let response = self.obniz.send_await_response(message, "system".to_string()).await
            .map_err(|e| ObnizError::Connection(e.to_string()))?;
        
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_config_serialization() {
        use serde_json;
        
        let config = ResetConfig {
            reset_obniz_on_ws_disconnection: true,
        };
        
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("reset_obniz_on_ws_disconnection"));
        assert!(serialized.contains("true"));
    }

    #[test]
    fn test_system_info_deserialization() {
        use serde_json;
        
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
    }
}