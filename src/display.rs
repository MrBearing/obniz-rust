use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::error::{ObnizError, ObnizResult};
use crate::obniz::Obniz;

/// QR code error correction levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QrCorrectionType {
    #[serde(rename = "L")]
    Low,
    #[serde(rename = "M")]
    Medium,
    #[serde(rename = "Q")]
    Quality,
    #[serde(rename = "H")]
    High,
}

/// Color depth for raw display data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisplayRawColorDepth {
    #[serde(rename = "1")]
    OneBit,
    #[serde(rename = "4")]
    FourBit,
    #[serde(rename = "16")]
    SixteenBit,
}

/// Display configuration for raw data
#[derive(Debug, Clone)]
pub struct RawDisplayConfig {
    pub width: u16,
    pub height: u16,
    pub color_depth: DisplayRawColorDepth,
    pub data: Vec<u16>,
}

/// Pin assignment configuration for display modules
#[derive(Debug, Clone)]
pub struct PinAssignment {
    pub pin: u8,
    pub module_name: String,
    pub pin_name: String,
}

/// Display manager for obniz device
#[derive(Debug, Clone)]
pub struct DisplayManager {
    obniz: Obniz,
}

impl DisplayManager {
    pub fn new(obniz: Obniz) -> Self {
        Self { obniz }
    }

    /// Display text on the obniz screen
    pub async fn text(&self, text: &str) -> ObnizResult<()> {
        if text.is_empty() {
            return Err(ObnizError::Generic("Text cannot be empty".to_string()));
        }

        let request = json!([{"display": {"text": text}}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Clear the display
    pub async fn clear(&self) -> ObnizResult<()> {
        let request = json!([{"display": {"clear": true}}]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Display a QR code with specified error correction level
    pub async fn qr(&self, text: &str, correction_type: QrCorrectionType) -> ObnizResult<()> {
        if text.is_empty() {
            return Err(ObnizError::Generic("QR text cannot be empty".to_string()));
        }

        let request = json!([{
            "display": {
                "qr": {
                    "text": text,
                    "correction": correction_type
                }
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Display raw pixel data
    pub async fn raw(&self, config: RawDisplayConfig) -> ObnizResult<()> {
        if config.data.is_empty() {
            return Err(ObnizError::Generic("Raw data cannot be empty".to_string()));
        }

        if config.width == 0 || config.height == 0 {
            return Err(ObnizError::Generic(
                "Width and height must be greater than 0".to_string(),
            ));
        }

        let expected_length = match config.color_depth {
            DisplayRawColorDepth::OneBit => (config.width * config.height).div_ceil(8),
            DisplayRawColorDepth::FourBit => (config.width * config.height).div_ceil(2),
            DisplayRawColorDepth::SixteenBit => config.width * config.height,
        };

        if config.data.len() != expected_length as usize {
            return Err(ObnizError::Generic(format!(
                "Data length mismatch. Expected {expected_length} but got {}",
                config.data.len()
            )));
        }

        let request = json!([{
            "display": {
                "raw": {
                    "width": config.width,
                    "height": config.height,
                    "color_depth": config.color_depth,
                    "data": config.data
                }
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Set display brightness (0-100)
    pub async fn brightness(&self, level: u8) -> ObnizResult<()> {
        if level > 100 {
            return Err(ObnizError::Generic(
                "Brightness level must be between 0-100".to_string(),
            ));
        }

        let request = json!([{
            "display": {
                "brightness": level
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Set display contrast (0-100)
    pub async fn contrast(&self, level: u8) -> ObnizResult<()> {
        if level > 100 {
            return Err(ObnizError::Generic(
                "Contrast level must be between 0-100".to_string(),
            ));
        }

        let request = json!([{
            "display": {
                "contrast": level
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Assign pins for display modules
    pub async fn pin_assign(&self, assignments: Vec<PinAssignment>) -> ObnizResult<()> {
        if assignments.is_empty() {
            return Err(ObnizError::Generic(
                "Pin assignments cannot be empty".to_string(),
            ));
        }

        for assignment in &assignments {
            if assignment.pin > 11 {
                return Err(ObnizError::InvalidPin(assignment.pin));
            }
            if assignment.module_name.is_empty() || assignment.pin_name.is_empty() {
                return Err(ObnizError::Generic(
                    "Module name and pin name cannot be empty".to_string(),
                ));
            }
        }

        let mut pin_config = json!({});
        for assignment in assignments {
            pin_config[format!("io{}", assignment.pin)] = json!({
                assignment.module_name: assignment.pin_name
            });
        }

        let request = json!([pin_config]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Draw a pixel at specified coordinates
    pub async fn pixel(&self, x: u16, y: u16, color: bool) -> ObnizResult<()> {
        let request = json!([{
            "display": {
                "pixel": {
                    "x": x,
                    "y": y,
                    "color": color
                }
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Draw a line from (x1, y1) to (x2, y2)
    pub async fn line(&self, x1: u16, y1: u16, x2: u16, y2: u16, color: bool) -> ObnizResult<()> {
        let request = json!([{
            "display": {
                "line": {
                    "x1": x1,
                    "y1": y1,
                    "x2": x2,
                    "y2": y2,
                    "color": color
                }
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Draw a rectangle
    pub async fn rect(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        filled: bool,
        color: bool,
    ) -> ObnizResult<()> {
        if width == 0 || height == 0 {
            return Err(ObnizError::Generic(
                "Width and height must be greater than 0".to_string(),
            ));
        }

        let request = json!([{
            "display": {
                "rect": {
                    "x": x,
                    "y": y,
                    "width": width,
                    "height": height,
                    "filled": filled,
                    "color": color
                }
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Draw a circle
    pub async fn circle(
        &self,
        x: u16,
        y: u16,
        radius: u16,
        filled: bool,
        color: bool,
    ) -> ObnizResult<()> {
        if radius == 0 {
            return Err(ObnizError::Generic(
                "Radius must be greater than 0".to_string(),
            ));
        }

        let request = json!([{
            "display": {
                "circle": {
                    "x": x,
                    "y": y,
                    "radius": radius,
                    "filled": filled,
                    "color": color
                }
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Set text size
    pub async fn text_size(&self, size: u8) -> ObnizResult<()> {
        if size == 0 {
            return Err(ObnizError::Generic(
                "Text size must be greater than 0".to_string(),
            ));
        }

        let request = json!([{
            "display": {
                "text_size": size
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }

    /// Set text position
    pub async fn text_pos(&self, x: u16, y: u16) -> ObnizResult<()> {
        let request = json!([{
            "display": {
                "text_pos": {
                    "x": x,
                    "y": y
                }
            }
        }]);
        let message = Message::from(request.to_string());

        self.obniz
            .send_message(message)
            .map_err(|e| ObnizError::Connection(e.to_string()))
    }
}

/// Legacy trait for backward compatibility
pub trait ObnizDisplay {
    fn display_text(&self, text: &str) -> ObnizResult<()>;
    fn display_clear(&self) -> ObnizResult<()>;
    fn display_qr(&self, text: &str, correction_type: QrCorrectionType) -> ObnizResult<()>;
    fn display_raw(&self, config: RawDisplayConfig) -> ObnizResult<()>;
    fn display_pin_assign(&self, assignments: Vec<PinAssignment>) -> ObnizResult<()>;
}

impl ObnizDisplay for Obniz {
    fn display_text(&self, text: &str) -> ObnizResult<()> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| ObnizError::Generic("No tokio runtime available".to_string()))?;

        rt.block_on(async { self.display().text(text).await })
    }

    fn display_clear(&self) -> ObnizResult<()> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| ObnizError::Generic("No tokio runtime available".to_string()))?;

        rt.block_on(async { self.display().clear().await })
    }

    fn display_qr(&self, text: &str, correction_type: QrCorrectionType) -> ObnizResult<()> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| ObnizError::Generic("No tokio runtime available".to_string()))?;

        rt.block_on(async { self.display().qr(text, correction_type).await })
    }

    fn display_raw(&self, config: RawDisplayConfig) -> ObnizResult<()> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| ObnizError::Generic("No tokio runtime available".to_string()))?;

        rt.block_on(async { self.display().raw(config).await })
    }

    fn display_pin_assign(&self, assignments: Vec<PinAssignment>) -> ObnizResult<()> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| ObnizError::Generic("No tokio runtime available".to_string()))?;

        rt.block_on(async { self.display().pin_assign(assignments).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_correction_type_serialization() {
        use serde_json;

        let correction = QrCorrectionType::Low;
        let serialized = serde_json::to_string(&correction).unwrap();
        assert_eq!(serialized, "\"L\"");

        let correction = QrCorrectionType::High;
        let serialized = serde_json::to_string(&correction).unwrap();
        assert_eq!(serialized, "\"H\"");
    }

    #[test]
    fn test_color_depth_serialization() {
        use serde_json;

        let depth = DisplayRawColorDepth::OneBit;
        let serialized = serde_json::to_string(&depth).unwrap();
        assert_eq!(serialized, "\"1\"");

        let depth = DisplayRawColorDepth::SixteenBit;
        let serialized = serde_json::to_string(&depth).unwrap();
        assert_eq!(serialized, "\"16\"");
    }

    #[test]
    fn test_raw_display_config_validation() {
        // Test valid config
        let config = RawDisplayConfig {
            width: 128,
            height: 64,
            color_depth: DisplayRawColorDepth::OneBit,
            data: vec![0; 1024], // 128 * 64 / 8 = 1024 bytes for 1-bit
        };
        assert_eq!(config.width, 128);
        assert_eq!(config.height, 64);
    }

    #[test]
    fn test_pin_assignment_creation() {
        let assignment = PinAssignment {
            pin: 5,
            module_name: "spi".to_string(),
            pin_name: "mosi".to_string(),
        };

        assert_eq!(assignment.pin, 5);
        assert_eq!(assignment.module_name, "spi");
        assert_eq!(assignment.pin_name, "mosi");
    }
}
