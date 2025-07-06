use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::obniz::Obniz;
use crate::error::{ObnizError, ObnizResult};

/// PWM modulation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModulationType {
    Am, // Amplitude Modulation
}

/// PWM configuration
#[derive(Debug, Clone)]
pub struct PwmConfig {
    pub io_pin: u8,
    pub frequency: u32,
    pub pulse_width_ms: f64,
}

/// Modulation configuration
#[derive(Debug, Clone)]
pub struct ModulationConfig {
    pub modulation_type: ModulationType,
    pub symbol_length_ms: f64,
    pub data: Vec<u8>,
}

/// Individual PWM channel controller
#[derive(Debug)]
pub struct PwmChannel {
    channel: u8,
    obniz: Obniz,
}

impl PwmChannel {
    pub fn new(channel: u8, obniz: Obniz) -> Self {
        Self { channel, obniz }
    }

    pub fn channel_key(&self) -> String {
        format!("pwm{}", self.channel)
    }

    /// Initialize PWM channel with IO pin
    pub async fn init(&self, io_pin: u8) -> ObnizResult<()> {
        if self.channel > 5 {
            return Err(ObnizError::Generic("PWM channel must be 0-5".to_string()));
        }
        
        if io_pin > 11 {
            return Err(ObnizError::InvalidPin(io_pin));
        }

        let channel_key = self.channel_key();
        let request = json!([{&channel_key: {"io": io_pin}}]);
        let message = Message::from(request.to_string());
        
        self.obniz.send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Set PWM frequency (1 Hz to 80,000,000 Hz)
    pub async fn set_frequency(&self, frequency: u32) -> ObnizResult<()> {
        if frequency == 0 || frequency > 80_000_000 {
            return Err(ObnizError::Generic("Frequency must be between 1 and 80,000,000 Hz".to_string()));
        }

        let channel_key = self.channel_key();
        let request = json!([{&channel_key: {"freq": frequency}}]);
        let message = Message::from(request.to_string());
        
        self.obniz.send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Set pulse width in milliseconds
    pub async fn set_pulse_width(&self, pulse_width_ms: f64) -> ObnizResult<()> {
        if pulse_width_ms < 0.0 {
            return Err(ObnizError::Generic("Pulse width must be >= 0".to_string()));
        }

        let channel_key = self.channel_key();
        let request = json!([{&channel_key: {"pulse": pulse_width_ms}}]);
        let message = Message::from(request.to_string());
        
        self.obniz.send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Set duty cycle as percentage (0.0 to 100.0)
    pub async fn set_duty_cycle(&self, frequency: u32, duty_percent: f64) -> ObnizResult<()> {
        if duty_percent < 0.0 || duty_percent > 100.0 {
            return Err(ObnizError::Generic("Duty cycle must be between 0 and 100%".to_string()));
        }

        // Calculate pulse width from duty cycle and frequency
        let period_ms = 1000.0 / frequency as f64;
        let pulse_width_ms = period_ms * duty_percent / 100.0;
        
        self.set_frequency(frequency).await?;
        self.set_pulse_width(pulse_width_ms).await
    }

    /// Configure PWM with all parameters
    pub async fn configure(&self, config: PwmConfig) -> ObnizResult<()> {
        self.init(config.io_pin).await?;
        self.set_frequency(config.frequency).await?;
        self.set_pulse_width(config.pulse_width_ms).await
    }

    /// Set up modulation
    pub async fn modulate(&self, config: ModulationConfig) -> ObnizResult<()> {
        if config.symbol_length_ms < 0.05 || config.symbol_length_ms > 1000.0 {
            return Err(ObnizError::Generic("Symbol length must be between 0.05 and 1000 ms".to_string()));
        }

        if config.data.is_empty() {
            return Err(ObnizError::Generic("Modulation data cannot be empty".to_string()));
        }

        // Validate data values (should be 0 or 1 for binary data)
        for &value in &config.data {
            if value > 1 {
                return Err(ObnizError::Generic("Modulation data must contain only 0 and 1".to_string()));
            }
        }

        let channel_key = self.channel_key();
        let request = json!([{&channel_key: {
            "modulate": {
                "type": config.modulation_type,
                "symbol_length": config.symbol_length_ms,
                "data": config.data
            }
        }}]);
        let message = Message::from(request.to_string());
        
        self.obniz.send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Generate square wave with 50% duty cycle
    pub async fn square_wave(&self, io_pin: u8, frequency: u32) -> ObnizResult<()> {
        let config = PwmConfig {
            io_pin,
            frequency,
            pulse_width_ms: 500.0 / frequency as f64, // 50% duty cycle
        };
        self.configure(config).await
    }

    /// Generate servo control signal (20ms period, 1-2ms pulse width)
    pub async fn servo(&self, io_pin: u8, angle: f64) -> ObnizResult<()> {
        if angle < 0.0 || angle > 180.0 {
            return Err(ObnizError::Generic("Servo angle must be between 0 and 180 degrees".to_string()));
        }

        // Standard servo: 1ms = 0°, 1.5ms = 90°, 2ms = 180°
        let pulse_width_ms = 1.0 + (angle / 180.0);
        
        let config = PwmConfig {
            io_pin,
            frequency: 50, // 20ms period
            pulse_width_ms,
        };
        self.configure(config).await
    }

    /// Deinitialize PWM channel
    pub async fn deinit(&self) -> ObnizResult<()> {
        let channel_key = self.channel_key();
        let request = json!([{&channel_key: null}]);
        let message = Message::from(request.to_string());
        
        self.obniz.send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }
}

/// PWM manager for handling multiple channels
#[derive(Debug, Clone)]
pub struct PwmManager {
    obniz: Obniz,
}

impl PwmManager {
    pub fn new(obniz: Obniz) -> Self {
        Self { obniz }
    }

    /// Get specific PWM channel (0-5)
    pub fn channel(&self, channel: u8) -> ObnizResult<PwmChannel> {
        if channel > 5 {
            return Err(ObnizError::Generic("PWM channel must be 0-5".to_string()));
        }
        Ok(PwmChannel::new(channel, self.obniz.clone()))
    }

    /// Configure PWM channel
    pub async fn configure_channel(&self, channel: u8, config: PwmConfig) -> ObnizResult<()> {
        self.channel(channel)?.configure(config).await
    }

    /// Set frequency for specific channel
    pub async fn set_channel_frequency(&self, channel: u8, frequency: u32) -> ObnizResult<()> {
        self.channel(channel)?.set_frequency(frequency).await
    }

    /// Set pulse width for specific channel
    pub async fn set_channel_pulse_width(&self, channel: u8, pulse_width_ms: f64) -> ObnizResult<()> {
        self.channel(channel)?.set_pulse_width(pulse_width_ms).await
    }

    /// Set duty cycle for specific channel
    pub async fn set_channel_duty_cycle(&self, channel: u8, frequency: u32, duty_percent: f64) -> ObnizResult<()> {
        self.channel(channel)?.set_duty_cycle(frequency, duty_percent).await
    }

    /// Generate square wave on specific channel
    pub async fn square_wave(&self, channel: u8, io_pin: u8, frequency: u32) -> ObnizResult<()> {
        self.channel(channel)?.square_wave(io_pin, frequency).await
    }

    /// Control servo on specific channel
    pub async fn servo(&self, channel: u8, io_pin: u8, angle: f64) -> ObnizResult<()> {
        self.channel(channel)?.servo(io_pin, angle).await
    }

    /// Deinitialize specific channel
    pub async fn deinit_channel(&self, channel: u8) -> ObnizResult<()> {
        self.channel(channel)?.deinit().await
    }

    /// Deinitialize all PWM channels
    pub async fn deinit_all(&self) -> ObnizResult<()> {
        for channel in 0..=5 {
            if let Err(e) = self.deinit_channel(channel).await {
                eprintln!("Failed to deinitialize PWM channel {}: {}", channel, e);
            }
        }
        Ok(())
    }

    /// Utility function to calculate pulse width from duty cycle
    pub fn duty_cycle_to_pulse_width(frequency: u32, duty_percent: f64) -> f64 {
        let period_ms = 1000.0 / frequency as f64;
        period_ms * duty_percent / 100.0
    }

    /// Utility function to calculate duty cycle from pulse width
    pub fn pulse_width_to_duty_cycle(frequency: u32, pulse_width_ms: f64) -> f64 {
        let period_ms = 1000.0 / frequency as f64;
        (pulse_width_ms / period_ms * 100.0).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwm_config_creation() {
        let config = PwmConfig {
            io_pin: 5,
            frequency: 1000,
            pulse_width_ms: 0.5,
        };
        
        assert_eq!(config.io_pin, 5);
        assert_eq!(config.frequency, 1000);
        assert_eq!(config.pulse_width_ms, 0.5);
    }

    #[test]
    fn test_modulation_config_creation() {
        let config = ModulationConfig {
            modulation_type: ModulationType::Am,
            symbol_length_ms: 100.0,
            data: vec![0, 1, 1, 0],
        };
        
        assert_eq!(config.modulation_type, ModulationType::Am);
        assert_eq!(config.symbol_length_ms, 100.0);
        assert_eq!(config.data, vec![0, 1, 1, 0]);
    }

    #[test]
    fn test_duty_cycle_calculations() {
        // Test duty cycle to pulse width conversion
        let pulse_width = PwmManager::duty_cycle_to_pulse_width(1000, 50.0);
        assert_eq!(pulse_width, 0.5); // 50% of 1ms period
        
        // Test pulse width to duty cycle conversion
        let duty_cycle = PwmManager::pulse_width_to_duty_cycle(1000, 0.25);
        assert_eq!(duty_cycle, 25.0); // 0.25ms of 1ms period
    }

    #[test]
    fn test_modulation_type_serialization() {
        use serde_json;
        
        let mod_type = ModulationType::Am;
        let serialized = serde_json::to_string(&mod_type).unwrap();
        assert_eq!(serialized, "\"am\"");
    }

    #[test]
    fn test_channel_key_generation() {
        assert_eq!(format!("pwm{}", 0), "pwm0");
        assert_eq!(format!("pwm{}", 5), "pwm5");
    }
}