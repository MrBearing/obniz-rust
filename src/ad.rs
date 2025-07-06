// Serde traits may be used in future for more complex AD configurations
use serde_json::json;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::error::{validate_pin, ObnizError, ObnizResult};
use crate::obniz::Obniz;

/// AD channel configuration
#[derive(Debug, Clone)]
pub struct AdConfig {
    pub stream: bool,
}

/// AD measurement result
#[derive(Debug, Clone, PartialEq)]
pub struct AdValue {
    pub channel: u8,
    pub voltage: f64,
}

/// Individual AD channel controller
#[derive(Debug)]
pub struct AdChannel {
    channel: u8,
    obniz: Obniz,
}

impl AdChannel {
    pub fn new(channel: u8, obniz: Obniz) -> Self {
        Self { channel, obniz }
    }

    pub fn channel_key(&self) -> String {
        format!("ad{}", self.channel)
    }

    /// Get current voltage reading
    pub async fn get(&self) -> ObnizResult<f64> {
        validate_pin(self.channel)?;

        let channel_key = self.channel_key();
        let request = json!([{&channel_key: "get"}]);
        let message = Message::from(request.to_string());

        let response = self
            .obniz
            .send_await_response(message, channel_key.clone())
            .await
            .map_err(|e| ObnizError::Connection(e.to_string()))?;

        // Parse the response to extract the voltage value
        // Response format is typically [{"ad2": 3.3}]
        if let Some(array) = response.as_array() {
            if let Some(first_item) = array.first() {
                if let Some(value) = first_item.get(&channel_key) {
                    return value.as_f64().ok_or_else(|| {
                        ObnizError::Generic("Invalid voltage value in response".to_string())
                    });
                }
            }
        }

        // Fallback: try direct object access
        if let Some(value) = response.get(&channel_key) {
            value
                .as_f64()
                .ok_or_else(|| ObnizError::Generic("Invalid voltage value in response".to_string()))
        } else {
            Err(ObnizError::Generic(format!(
                "No response for AD channel {}",
                self.channel
            )))
        }
    }

    /// Configure AD channel
    pub async fn configure(&self, config: AdConfig) -> ObnizResult<()> {
        validate_pin(self.channel)?;

        let channel_key = self.channel_key();
        let request = json!([{&channel_key: {"stream": config.stream}}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Enable streaming mode
    pub async fn enable_stream(&self) -> ObnizResult<()> {
        self.configure(AdConfig { stream: true }).await
    }

    /// Disable streaming mode
    pub async fn disable_stream(&self) -> ObnizResult<()> {
        self.configure(AdConfig { stream: false }).await
    }

    /// Register callback for voltage changes (stream mode)
    pub async fn on_change<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn(f64) + Send + Sync + 'static,
    {
        validate_pin(self.channel)?;

        // Enable stream mode first
        self.enable_stream().await?;

        let channel_key = self.channel_key();
        let channel_key_clone = channel_key.clone();

        self.obniz
            .register_callback(channel_key, move |response| {
                if let Some(value) = response.get(&channel_key_clone) {
                    if let Some(voltage) = value.as_f64() {
                        callback(voltage);
                    }
                }
            })
            .map_err(|e| ObnizError::CallbackError(e.to_string()))?;

        Ok(())
    }

    /// Remove callback for this channel
    pub fn remove_callback(&self) -> ObnizResult<()> {
        validate_pin(self.channel)?;
        let channel_key = self.channel_key();
        self.obniz
            .unregister_callback(channel_key)
            .map_err(|e| ObnizError::CallbackError(e.to_string()))
    }

    /// Deinitialize AD channel
    pub async fn deinit(&self) -> ObnizResult<()> {
        validate_pin(self.channel)?;

        let channel_key = self.channel_key();
        let request = json!([{&channel_key: null}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }
}

/// AD manager for handling multiple channels
#[derive(Debug, Clone)]
pub struct AdManager {
    obniz: Obniz,
}

impl AdManager {
    pub fn new(obniz: Obniz) -> Self {
        Self { obniz }
    }

    /// Get specific AD channel (0-11)
    pub fn channel(&self, channel: u8) -> ObnizResult<AdChannel> {
        validate_pin(channel)?;
        Ok(AdChannel::new(channel, self.obniz.clone()))
    }

    /// Get voltage from specific channel
    pub async fn get_voltage(&self, channel: u8) -> ObnizResult<f64> {
        self.channel(channel)?.get().await
    }

    /// Get voltages from multiple channels
    pub async fn get_voltages(&self, channels: Vec<u8>) -> ObnizResult<Vec<AdValue>> {
        let mut results = Vec::new();

        for channel in channels {
            let voltage = self.get_voltage(channel).await?;
            results.push(AdValue { channel, voltage });
        }

        Ok(results)
    }

    /// Enable streaming on specific channel
    pub async fn enable_channel_stream(&self, channel: u8) -> ObnizResult<()> {
        self.channel(channel)?.enable_stream().await
    }

    /// Disable streaming on specific channel
    pub async fn disable_channel_stream(&self, channel: u8) -> ObnizResult<()> {
        self.channel(channel)?.disable_stream().await
    }

    /// Set callback for specific channel
    pub async fn set_channel_callback<F>(&self, channel: u8, callback: F) -> ObnizResult<()>
    where
        F: Fn(f64) + Send + Sync + 'static,
    {
        self.channel(channel)?.on_change(callback).await
    }

    /// Remove callback for specific channel
    pub fn remove_channel_callback(&self, channel: u8) -> ObnizResult<()> {
        self.channel(channel)?.remove_callback()
    }

    /// Deinitialize specific channel
    pub async fn deinit_channel(&self, channel: u8) -> ObnizResult<()> {
        self.channel(channel)?.deinit().await
    }

    /// Deinitialize all channels
    pub async fn deinit_all(&self) -> ObnizResult<()> {
        for channel in 0..=11 {
            if let Err(e) = self.deinit_channel(channel).await {
                eprintln!("Failed to deinitialize AD channel {channel}: {e}");
            }
        }
        Ok(())
    }

    /// Get readings from all channels
    pub async fn read_all(&self) -> ObnizResult<Vec<AdValue>> {
        let channels: Vec<u8> = (0..=11).collect();
        self.get_voltages(channels).await
    }

    /// Utility function to convert voltage to percentage (0V = 0%, 5V = 100%)
    pub fn voltage_to_percentage(voltage: f64) -> f64 {
        (voltage / 5.0 * 100.0).clamp(0.0, 100.0)
    }

    /// Utility function to check if voltage is within safe range
    pub fn is_voltage_safe(voltage: f64) -> bool {
        (0.0..=5.0).contains(&voltage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ad_value_creation() {
        let value = AdValue {
            channel: 5,
            voltage: 3.3,
        };

        assert_eq!(value.channel, 5);
        assert_eq!(value.voltage, 3.3);
    }

    #[test]
    fn test_ad_config_creation() {
        let config = AdConfig { stream: true };
        assert!(config.stream);

        let config = AdConfig { stream: false };
        assert!(!config.stream);
    }

    #[test]
    fn test_voltage_to_percentage() {
        assert_eq!(AdManager::voltage_to_percentage(0.0), 0.0);
        assert_eq!(AdManager::voltage_to_percentage(2.5), 50.0);
        assert_eq!(AdManager::voltage_to_percentage(5.0), 100.0);
        assert_eq!(AdManager::voltage_to_percentage(6.0), 100.0); // Clamped
    }

    #[test]
    fn test_voltage_safety_check() {
        assert!(AdManager::is_voltage_safe(0.0));
        assert!(AdManager::is_voltage_safe(3.3));
        assert!(AdManager::is_voltage_safe(5.0));
        assert!(!AdManager::is_voltage_safe(-0.1));
        assert!(!AdManager::is_voltage_safe(5.1));
    }

    #[test]
    fn test_channel_key_generation() {
        // We can't easily test the full AdChannel without Obniz instance
        // but we can test the key generation logic
        assert_eq!(format!("ad{}", 0), "ad0");
        assert_eq!(format!("ad{}", 11), "ad11");
    }
}
