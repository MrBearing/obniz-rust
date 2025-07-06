use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::obniz::Obniz;
use crate::error::{ObnizError, ObnizResult};

/// Switch states for obniz board switch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwitchState {
    None,
    Push,
    Left,
    Right,
}

impl std::fmt::Display for SwitchState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwitchState::None => write!(f, "none"),
            SwitchState::Push => write!(f, "push"),
            SwitchState::Left => write!(f, "left"),
            SwitchState::Right => write!(f, "right"),
        }
    }
}

/// Switch action types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwitchAction {
    Get,
    Push,
    Release,
    Left,
    Right,
}

/// Switch response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchResponse {
    pub state: SwitchState,
    pub action: SwitchAction,
}

/// Switch manager for obniz board switch
#[derive(Debug, Clone)]
pub struct SwitchManager {
    obniz: Obniz,
}

impl SwitchManager {
    pub fn new(obniz: Obniz) -> Self {
        Self { obniz }
    }

    /// Get current switch state
    pub async fn get_state(&self) -> ObnizResult<SwitchState> {
        let request = json!([{"switch": "get"}]);
        let message = Message::from(request.to_string());
        
        let response = self.obniz.send_await_response(message, "switch".to_string()).await
            .map_err(|e| ObnizError::Connection(e.to_string()))?;
        
        // Parse the response to extract the switch state
        // Response format is typically [{"switch": {"state": "none", "action": "get"}}]
        if let Some(array) = response.as_array() {
            if let Some(first_item) = array.first() {
                if let Some(switch_data) = first_item.get("switch") {
                    if let Some(state_value) = switch_data.get("state") {
                        return serde_json::from_value(state_value.clone())
                            .map_err(|e| ObnizError::JsonParse(e.to_string()));
                    }
                }
            }
        }
        
        // Fallback: try direct object access
        if let Some(switch_data) = response.get("switch") {
            if let Some(state_value) = switch_data.get("state") {
                serde_json::from_value(state_value.clone())
                    .map_err(|e| ObnizError::JsonParse(e.to_string()))
            } else {
                Err(ObnizError::Generic("No state field in switch response".to_string()))
            }
        } else {
            Err(ObnizError::Generic("No switch data in response".to_string()))
        }
    }

    /// Check if switch is currently pressed (any direction)
    pub async fn is_pressed(&self) -> ObnizResult<bool> {
        let state = self.get_state().await?;
        Ok(state != SwitchState::None)
    }

    /// Check if switch is pressed in specific direction
    pub async fn is_pressed_direction(&self, direction: SwitchState) -> ObnizResult<bool> {
        let state = self.get_state().await?;
        Ok(state == direction)
    }

    /// Register callback for switch state changes
    pub async fn on_change<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn(SwitchState, SwitchAction) + Send + Sync + 'static,
    {
        self.obniz.register_callback("switch".to_string(), move |response| {
            if let Some(switch_data) = response.get("switch") {
                if let Ok(switch_response) = serde_json::from_value::<SwitchResponse>(switch_data.clone()) {
                    callback(switch_response.state, switch_response.action);
                }
            }
        }).map_err(|e| ObnizError::CallbackError(e.to_string()))?;
        
        Ok(())
    }

    /// Register callback for push events only
    pub async fn on_push<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_change(move |state, action| {
            if state == SwitchState::Push && action == SwitchAction::Push {
                callback();
            }
        }).await
    }

    /// Register callback for release events
    pub async fn on_release<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_change(move |state, action| {
            if state == SwitchState::None && action == SwitchAction::Release {
                callback();
            }
        }).await
    }

    /// Register callback for left direction events
    pub async fn on_left<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_change(move |state, action| {
            if state == SwitchState::Left && action == SwitchAction::Left {
                callback();
            }
        }).await
    }

    /// Register callback for right direction events
    pub async fn on_right<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_change(move |state, action| {
            if state == SwitchState::Right && action == SwitchAction::Right {
                callback();
            }
        }).await
    }

    /// Register callback for any press event (push, left, or right)
    pub async fn on_any_press<F>(&self, callback: F) -> ObnizResult<()>
    where
        F: Fn(SwitchState) + Send + Sync + 'static,
    {
        self.on_change(move |state, _action| {
            if state != SwitchState::None {
                callback(state.clone());
            }
        }).await
    }

    /// Remove switch callback
    pub fn remove_callback(&self) -> ObnizResult<()> {
        self.obniz.unregister_callback("switch".to_string())
            .map_err(|e| ObnizError::CallbackError(e.to_string()))
    }

    /// Wait for specific switch state (blocking until state is reached)
    pub async fn wait_for_state(&self, target_state: SwitchState, timeout_ms: Option<u64>) -> ObnizResult<()> {
        use tokio::time::{sleep, Duration, timeout};
        
        let check_interval = Duration::from_millis(50);
        let max_duration = timeout_ms.map(Duration::from_millis);
        
        let wait_future = async {
            loop {
                let current_state = self.get_state().await?;
                if current_state == target_state {
                    return Ok(());
                }
                sleep(check_interval).await;
            }
        };
        
        match max_duration {
            Some(duration) => {
                timeout(duration, wait_future).await
                    .map_err(|_| ObnizError::Timeout)?
            }
            None => wait_future.await,
        }
    }

    /// Wait for any press event
    pub async fn wait_for_press(&self, timeout_ms: Option<u64>) -> ObnizResult<SwitchState> {
        use tokio::time::{sleep, Duration, timeout};
        
        let check_interval = Duration::from_millis(50);
        let max_duration = timeout_ms.map(Duration::from_millis);
        
        let wait_future = async {
            loop {
                let current_state = self.get_state().await?;
                if current_state != SwitchState::None {
                    return Ok(current_state);
                }
                sleep(check_interval).await;
            }
        };
        
        match max_duration {
            Some(duration) => {
                timeout(duration, wait_future).await
                    .map_err(|_| ObnizError::Timeout)?
            }
            None => wait_future.await,
        }
    }

    /// Wait for release event
    pub async fn wait_for_release(&self, timeout_ms: Option<u64>) -> ObnizResult<()> {
        self.wait_for_state(SwitchState::None, timeout_ms).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_state_serialization() {
        use serde_json;
        
        assert_eq!(serde_json::to_string(&SwitchState::None).unwrap(), "\"none\"");
        assert_eq!(serde_json::to_string(&SwitchState::Push).unwrap(), "\"push\"");
        assert_eq!(serde_json::to_string(&SwitchState::Left).unwrap(), "\"left\"");
        assert_eq!(serde_json::to_string(&SwitchState::Right).unwrap(), "\"right\"");
    }

    #[test]
    fn test_switch_state_deserialization() {
        use serde_json;
        
        assert_eq!(serde_json::from_str::<SwitchState>("\"none\"").unwrap(), SwitchState::None);
        assert_eq!(serde_json::from_str::<SwitchState>("\"push\"").unwrap(), SwitchState::Push);
        assert_eq!(serde_json::from_str::<SwitchState>("\"left\"").unwrap(), SwitchState::Left);
        assert_eq!(serde_json::from_str::<SwitchState>("\"right\"").unwrap(), SwitchState::Right);
    }

    #[test]
    fn test_switch_action_serialization() {
        use serde_json;
        
        assert_eq!(serde_json::to_string(&SwitchAction::Get).unwrap(), "\"get\"");
        assert_eq!(serde_json::to_string(&SwitchAction::Push).unwrap(), "\"push\"");
        assert_eq!(serde_json::to_string(&SwitchAction::Release).unwrap(), "\"release\"");
        assert_eq!(serde_json::to_string(&SwitchAction::Left).unwrap(), "\"left\"");
        assert_eq!(serde_json::to_string(&SwitchAction::Right).unwrap(), "\"right\"");
    }

    #[test]
    fn test_switch_response_serialization() {
        use serde_json;
        
        let response = SwitchResponse {
            state: SwitchState::Push,
            action: SwitchAction::Push,
        };
        
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("\"state\":\"push\""));
        assert!(serialized.contains("\"action\":\"push\""));
    }

    #[test]
    fn test_switch_response_deserialization() {
        use serde_json;
        
        let json_str = r#"{"state": "left", "action": "left"}"#;
        let response: SwitchResponse = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(response.state, SwitchState::Left);
        assert_eq!(response.action, SwitchAction::Left);
    }

    #[test]
    fn test_switch_state_display() {
        assert_eq!(format!("{}", SwitchState::None), "none");
        assert_eq!(format!("{}", SwitchState::Push), "push");
        assert_eq!(format!("{}", SwitchState::Left), "left");
        assert_eq!(format!("{}", SwitchState::Right), "right");
    }

    #[test]
    fn test_switch_state_equality() {
        assert_eq!(SwitchState::None, SwitchState::None);
        assert_eq!(SwitchState::Push, SwitchState::Push);
        assert_ne!(SwitchState::None, SwitchState::Push);
        assert_ne!(SwitchState::Left, SwitchState::Right);
    }
}