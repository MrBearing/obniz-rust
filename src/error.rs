use std::fmt;

/// Custom error types for the obniz library
#[derive(Debug)]
pub enum ObnizError {
    /// Connection errors
    Connection(String),

    /// WebSocket communication errors
    WebSocket(String),

    /// IO operation errors
    IoOperation(String),

    /// Invalid pin number (valid range: 0-11)
    InvalidPin(u8),

    /// JSON parsing errors
    JsonParse(String),

    /// Response timeout
    Timeout,

    /// Callback registration errors
    CallbackError(String),

    /// Device not found or invalid device ID
    DeviceNotFound(String),

    /// Permission denied
    PermissionDenied,

    /// Generic error with message
    Generic(String),
}

impl fmt::Display for ObnizError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObnizError::Connection(msg) => write!(f, "Connection error: {}", msg),
            ObnizError::WebSocket(msg) => write!(f, "WebSocket error: {}", msg),
            ObnizError::IoOperation(msg) => write!(f, "IO operation error: {}", msg),
            ObnizError::InvalidPin(pin) => {
                write!(f, "Invalid pin number: {}. Valid range is 0-11", pin)
            }
            ObnizError::JsonParse(msg) => write!(f, "JSON parse error: {}", msg),
            ObnizError::Timeout => write!(f, "Operation timed out"),
            ObnizError::CallbackError(msg) => write!(f, "Callback error: {}", msg),
            ObnizError::DeviceNotFound(id) => write!(f, "Device not found: {}", id),
            ObnizError::PermissionDenied => write!(f, "Permission denied"),
            ObnizError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ObnizError {}

impl From<anyhow::Error> for ObnizError {
    fn from(err: anyhow::Error) -> Self {
        ObnizError::Generic(err.to_string())
    }
}

impl From<serde_json::Error> for ObnizError {
    fn from(err: serde_json::Error) -> Self {
        ObnizError::JsonParse(err.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for ObnizError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        ObnizError::WebSocket(err.to_string())
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for ObnizError {
    fn from(_: tokio::sync::oneshot::error::RecvError) -> Self {
        ObnizError::Timeout
    }
}

impl From<tokio::sync::mpsc::error::SendError<crate::obniz::ObnizCommand>> for ObnizError {
    fn from(err: tokio::sync::mpsc::error::SendError<crate::obniz::ObnizCommand>) -> Self {
        ObnizError::Connection(format!("Failed to send command: {}", err))
    }
}

/// Result type alias for obniz operations
pub type ObnizResult<T> = Result<T, ObnizError>;

/// Validation helpers
pub fn validate_pin(pin: u8) -> ObnizResult<()> {
    if pin > 11 {
        Err(ObnizError::InvalidPin(pin))
    } else {
        Ok(())
    }
}

/// Timeout helper for operations
pub async fn with_timeout<F, T>(future: F, timeout_duration: std::time::Duration) -> ObnizResult<T>
where
    F: std::future::Future<Output = ObnizResult<T>>,
{
    match tokio::time::timeout(timeout_duration, future).await {
        Ok(result) => result,
        Err(_) => Err(ObnizError::Timeout),
    }
}
