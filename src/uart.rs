use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::error::{ObnizError, ObnizResult};
use crate::obniz::Obniz;

/// UART parity settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Parity {
    Off,
    Odd,
    Even,
}

/// UART flow control settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowControl {
    Off,
    Rts,
    Cts,
    #[serde(rename = "rts-cts")]
    RtsCts,
}

/// UART configuration
#[derive(Debug, Clone)]
pub struct UartConfig {
    pub rx_pin: u8,
    pub tx_pin: u8,
    pub baud_rate: u32,
    pub stop_bits: f32,
    pub data_bits: u8,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub rts_pin: Option<u8>,
    pub cts_pin: Option<u8>,
}

impl Default for UartConfig {
    fn default() -> Self {
        Self {
            rx_pin: 0,
            tx_pin: 1,
            baud_rate: 115200,
            stop_bits: 1.0,
            data_bits: 8,
            parity: Parity::Off,
            flow_control: FlowControl::Off,
            rts_pin: None,
            cts_pin: None,
        }
    }
}

/// UART communication manager
#[derive(Debug)]
pub struct UartChannel {
    channel: u8,
    obniz: Obniz,
}

impl UartChannel {
    pub fn new(channel: u8, obniz: Obniz) -> Self {
        Self { channel, obniz }
    }

    pub fn channel_key(&self) -> String {
        format!("uart{}", self.channel)
    }

    /// Initialize UART with configuration
    pub async fn init(&self, config: UartConfig) -> ObnizResult<()> {
        // Validate pins
        if config.rx_pin > 11 || config.tx_pin > 11 {
            return Err(ObnizError::Generic("UART pins must be 0-11".to_string()));
        }

        if let Some(rts_pin) = config.rts_pin {
            if rts_pin > 11 {
                return Err(ObnizError::InvalidPin(rts_pin));
            }
        }

        if let Some(cts_pin) = config.cts_pin {
            if cts_pin > 11 {
                return Err(ObnizError::InvalidPin(cts_pin));
            }
        }

        // Validate baud rate
        if config.baud_rate == 0 || config.baud_rate > 5_000_000 {
            return Err(ObnizError::Generic(
                "Baud rate must be between 1 and 5,000,000".to_string(),
            ));
        }

        // Validate data bits
        if config.data_bits < 5 || config.data_bits > 8 {
            return Err(ObnizError::Generic(
                "Data bits must be 5, 6, 7, or 8".to_string(),
            ));
        }

        // Validate stop bits
        if config.stop_bits != 1.0 && config.stop_bits != 1.5 && config.stop_bits != 2.0 {
            return Err(ObnizError::Generic(
                "Stop bits must be 1, 1.5, or 2".to_string(),
            ));
        }

        let channel_key = self.channel_key();
        let mut uart_config = json!({
            "rx": config.rx_pin,
            "tx": config.tx_pin,
            "baud": config.baud_rate,
            "stop": config.stop_bits,
            "bits": config.data_bits,
            "parity": config.parity
        });

        // Add flow control pins if specified
        if let Some(rts_pin) = config.rts_pin {
            uart_config["rts"] = json!(rts_pin);
        }
        if let Some(cts_pin) = config.cts_pin {
            uart_config["cts"] = json!(cts_pin);
        }

        let request = json!([{&channel_key: uart_config}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Send data via UART
    pub async fn send(&self, data: Vec<u8>) -> ObnizResult<()> {
        if data.is_empty() {
            return Err(ObnizError::Generic("Data cannot be empty".to_string()));
        }

        // Data validation: u8 values are inherently 0-255, no additional check needed

        let channel_key = self.channel_key();
        let request = json!([{&channel_key: {"data": data}}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Send string data via UART (converted to UTF-8 bytes)
    pub async fn send_string(&self, text: &str) -> ObnizResult<()> {
        let data = text.as_bytes().to_vec();
        self.send(data).await
    }

    /// Register callback for received data
    pub async fn on_receive<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        let channel_key = self.channel_key();
        let channel_key_clone = channel_key.clone();

        self.obniz
            .register_callback(channel_key, move |response| {
                if let Some(uart_data) = response.get(&channel_key_clone) {
                    if let Some(data_array) = uart_data.get("data") {
                        if let Some(data_vec) = data_array.as_array() {
                            let bytes: Vec<u8> = data_vec
                                .iter()
                                .filter_map(|v| v.as_u64())
                                .map(|v| v as u8)
                                .collect();
                            callback(bytes);
                        }
                    }
                }
            })
            .map_err(|e| ObnizError::CallbackError(e.to_string()))?;

        Ok(())
    }

    /// Register callback for received string data
    pub async fn on_receive_string<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.on_receive(move |data| {
            if let Ok(text) = String::from_utf8(data) {
                callback(text);
            }
        })
        .await
    }

    /// Remove receive callback
    pub fn remove_callback(&self) -> ObnizResult<()> {
        let channel_key = self.channel_key();
        self.obniz
            .unregister_callback(channel_key)
            .map_err(|e| ObnizError::CallbackError(e.to_string()))
    }

    /// Deinitialize UART channel
    pub async fn deinit(&self) -> ObnizResult<()> {
        let channel_key = self.channel_key();
        let request = json!([{&channel_key: null}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }
}

/// UART manager for handling multiple channels
#[derive(Debug, Clone)]
pub struct UartManager {
    obniz: Obniz,
}

impl UartManager {
    pub fn new(obniz: Obniz) -> Self {
        Self { obniz }
    }

    /// Get specific UART channel (typically uart0)
    pub fn channel(&self, channel: u8) -> ObnizResult<UartChannel> {
        // Most obniz devices support uart0, some may support more
        if channel > 2 {
            return Err(ObnizError::Generic("UART channel must be 0-2".to_string()));
        }
        Ok(UartChannel::new(channel, self.obniz.clone()))
    }

    /// Get primary UART channel (uart0)
    pub fn uart0(&self) -> UartChannel {
        UartChannel::new(0, self.obniz.clone())
    }

    /// Initialize UART channel with configuration
    pub async fn init_channel(&self, channel: u8, config: UartConfig) -> ObnizResult<()> {
        self.channel(channel)?.init(config).await
    }

    /// Send data on specific channel
    pub async fn send_data(&self, channel: u8, data: Vec<u8>) -> ObnizResult<()> {
        self.channel(channel)?.send(data).await
    }

    /// Send string on specific channel
    pub async fn send_string(&self, channel: u8, text: &str) -> ObnizResult<()> {
        self.channel(channel)?.send_string(text).await
    }

    /// Set receive callback for specific channel
    pub async fn set_receive_callback<F>(&self, channel: u8, callback: F) -> ObnizResult<()>
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        self.channel(channel)?.on_receive(callback).await
    }

    /// Set string receive callback for specific channel
    pub async fn set_string_callback<F>(&self, channel: u8, callback: F) -> ObnizResult<()>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.channel(channel)?.on_receive_string(callback).await
    }

    /// Remove callback for specific channel
    pub fn remove_channel_callback(&self, channel: u8) -> ObnizResult<()> {
        self.channel(channel)?.remove_callback()
    }

    /// Deinitialize specific channel
    pub async fn deinit_channel(&self, channel: u8) -> ObnizResult<()> {
        self.channel(channel)?.deinit().await
    }

    /// Create simple UART configuration for common use cases
    pub fn simple_config(rx_pin: u8, tx_pin: u8, baud_rate: u32) -> UartConfig {
        UartConfig {
            rx_pin,
            tx_pin,
            baud_rate,
            ..Default::default()
        }
    }

    /// Create UART configuration with flow control
    pub fn flow_control_config(
        rx_pin: u8,
        tx_pin: u8,
        rts_pin: u8,
        cts_pin: u8,
        baud_rate: u32,
    ) -> UartConfig {
        UartConfig {
            rx_pin,
            tx_pin,
            baud_rate,
            flow_control: FlowControl::RtsCts,
            rts_pin: Some(rts_pin),
            cts_pin: Some(cts_pin),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uart_config_default() {
        let config = UartConfig::default();

        assert_eq!(config.rx_pin, 0);
        assert_eq!(config.tx_pin, 1);
        assert_eq!(config.baud_rate, 115200);
        assert_eq!(config.stop_bits, 1.0);
        assert_eq!(config.data_bits, 8);
        assert_eq!(config.parity, Parity::Off);
        assert_eq!(config.flow_control, FlowControl::Off);
    }

    #[test]
    fn test_uart_config_custom() {
        let config = UartConfig {
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

        assert_eq!(config.rx_pin, 2);
        assert_eq!(config.tx_pin, 3);
        assert_eq!(config.baud_rate, 9600);
        assert_eq!(config.parity, Parity::Even);
        assert_eq!(config.flow_control, FlowControl::RtsCts);
    }

    #[test]
    fn test_parity_serialization() {
        use serde_json;

        assert_eq!(serde_json::to_string(&Parity::Off).unwrap(), "\"off\"");
        assert_eq!(serde_json::to_string(&Parity::Odd).unwrap(), "\"odd\"");
        assert_eq!(serde_json::to_string(&Parity::Even).unwrap(), "\"even\"");
    }

    #[test]
    fn test_flow_control_serialization() {
        use serde_json;

        assert_eq!(serde_json::to_string(&FlowControl::Off).unwrap(), "\"off\"");
        assert_eq!(serde_json::to_string(&FlowControl::Rts).unwrap(), "\"rts\"");
        assert_eq!(serde_json::to_string(&FlowControl::Cts).unwrap(), "\"cts\"");
        assert_eq!(
            serde_json::to_string(&FlowControl::RtsCts).unwrap(),
            "\"rts-cts\""
        );
    }

    #[test]
    fn test_simple_config_creation() {
        let config = UartManager::simple_config(2, 3, 9600);

        assert_eq!(config.rx_pin, 2);
        assert_eq!(config.tx_pin, 3);
        assert_eq!(config.baud_rate, 9600);
        assert_eq!(config.parity, Parity::Off);
    }

    #[test]
    fn test_flow_control_config_creation() {
        let config = UartManager::flow_control_config(0, 1, 2, 3, 115200);

        assert_eq!(config.rx_pin, 0);
        assert_eq!(config.tx_pin, 1);
        assert_eq!(config.rts_pin, Some(2));
        assert_eq!(config.cts_pin, Some(3));
        assert_eq!(config.flow_control, FlowControl::RtsCts);
    }

    #[test]
    fn test_channel_key_generation() {
        assert_eq!(format!("uart{}", 0), "uart0");
        assert_eq!(format!("uart{}", 1), "uart1");
    }
}
