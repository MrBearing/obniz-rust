use std::collections::HashMap;
use std::sync::Arc;

use anyhow::*;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt,
};
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio_tungstenite::{
    connect_async as ws_connect_async, tungstenite::protocol::Message, MaybeTlsStream,
    WebSocketStream,
};

use serde_json::Value;

use crate::ad::AdManager;
use crate::display::DisplayManager;
use crate::io::IoManager;
use crate::pwm::PwmManager;
use crate::switch::SwitchManager;
use crate::system::SystemManager;
use crate::uart::UartManager;

const OBNIZE_WEBSOKET_HOST: &str = "wss://obniz.io";
pub type ObnizWSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub type CallbackFn = Box<dyn Fn(Value) + Send + Sync>;
pub type ResponseSender = oneshot::Sender<Value>;

pub enum CallbackType {
    OneShot(ResponseSender),
    Persistent(CallbackFn),
}

impl std::fmt::Debug for CallbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallbackType::OneShot(_) => write!(f, "CallbackType::OneShot(_)"),
            CallbackType::Persistent(_) => write!(f, "CallbackType::Persistent(_)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Obniz {
    id: String,
    sender: mpsc::UnboundedSender<ObnizCommand>,
    #[allow(dead_code)] // Used in WebSocket handler for callback routing
    callbacks: Arc<RwLock<HashMap<String, CallbackType>>>,
}

#[derive(Debug)]
pub enum ObnizCommand {
    Send {
        message: Message,
        response_key: Option<String>,
    },
    RegisterCallback {
        key: String,
        callback: CallbackType,
    },
    UnregisterCallback {
        key: String,
    },
}

impl Obniz {
    async fn new(id: &str, api_url: url::Url) -> anyhow::Result<Obniz> {
        let (socket, _response) = ws_connect_async(api_url.as_str())
            .await
            .context(format!("Failed to connect to {api_url}"))?;

        let (write, read) = socket.split();
        let (cmd_sender, cmd_receiver) = mpsc::unbounded_channel();
        let callbacks = Arc::new(RwLock::new(HashMap::new()));

        let callbacks_clone = callbacks.clone();

        // Spawn WebSocket handler task
        tokio::spawn(async move {
            Self::websocket_handler(write, read, cmd_receiver, callbacks_clone).await;
        });

        Ok(Obniz {
            id: id.to_string(),
            sender: cmd_sender,
            callbacks,
        })
    }

    async fn websocket_handler(
        mut write: SplitSink<ObnizWSocket, Message>,
        mut read: SplitStream<ObnizWSocket>,
        mut cmd_receiver: mpsc::UnboundedReceiver<ObnizCommand>,
        callbacks: Arc<RwLock<HashMap<String, CallbackType>>>,
    ) {
        loop {
            tokio::select! {
                cmd = cmd_receiver.recv() => {
                    match cmd {
                        Some(ObnizCommand::Send { message, response_key: _ }) => {
                            if let Err(e) = write.send(message).await {
                                eprintln!("Failed to send message: {e}");
                            }
                        }
                        Some(ObnizCommand::RegisterCallback { key, callback }) => {
                            callbacks.write().await.insert(key, callback);
                        }
                        Some(ObnizCommand::UnregisterCallback { key }) => {
                            callbacks.write().await.remove(&key);
                        }
                        None => break,
                    }
                }
                message = read.next() => {
                    match message {
                        Some(result) => {
                            match result {
                                std::result::Result::Ok(msg) => {
                                    if let Err(e) = Self::handle_incoming_message(msg, &callbacks).await {
                                        eprintln!("Failed to handle message: {e}");
                                    }
                                }
                                std::result::Result::Err(e) => {
                                    eprintln!("WebSocket error: {e}");
                                }
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    }

    async fn handle_incoming_message(
        message: Message,
        callbacks: &Arc<RwLock<HashMap<String, CallbackType>>>,
    ) -> anyhow::Result<()> {
        let text = message
            .to_text()
            .context("Failed to parse message as text")?;
        let value: Value = serde_json::from_str(text).context("Failed to parse JSON")?;

        let mut keys_to_remove = Vec::new();

        // Route message to appropriate callback
        {
            let callbacks_guard = callbacks.read().await;

            // Check if it's an array response (typical obniz format)
            if let Some(array) = value.as_array() {
                for item in array {
                    let mut remove_keys =
                        Self::route_message_to_callback(item, &callbacks_guard).await?;
                    keys_to_remove.append(&mut remove_keys);
                }
            } else {
                let mut remove_keys =
                    Self::route_message_to_callback(&value, &callbacks_guard).await?;
                keys_to_remove.append(&mut remove_keys);
            }
        }

        // Handle OneShot callbacks - send response and remove from map
        if !keys_to_remove.is_empty() {
            let mut callbacks_guard = callbacks.write().await;
            for key in keys_to_remove {
                if let Some(CallbackType::OneShot(sender)) = callbacks_guard.remove(&key) {
                    // Send the response through the channel
                    if sender.send(value.clone()).is_err() {
                        eprintln!("Failed to send response through oneshot channel for key: {key}");
                    }
                }
            }
        }

        Ok(())
    }

    async fn route_message_to_callback(
        message: &Value,
        callbacks: &HashMap<String, CallbackType>,
    ) -> anyhow::Result<Vec<String>> {
        let mut keys_to_remove = Vec::new();

        // Extract callback key from message structure
        let callback_key = Self::extract_callback_key(message);

        if let Some(key) = callback_key {
            if let Some(callback) = callbacks.get(&key) {
                match callback {
                    CallbackType::OneShot(_sender) => {
                        // For OneShot callbacks, we need to signal that this key should be removed
                        // The actual sending will be handled in the websocket_handler
                        keys_to_remove.push(key.clone());
                    }
                    CallbackType::Persistent(callback_fn) => {
                        callback_fn(message.clone());
                    }
                }
            }
        }

        Ok(keys_to_remove)
    }

    fn extract_callback_key(message: &Value) -> Option<String> {
        // Handle array responses first (most common format from obniz)
        if let Some(array) = message.as_array() {
            if let Some(first_item) = array.first() {
                if let Some(obj) = first_item.as_object() {
                    for (key, _) in obj {
                        // Check for various obniz response patterns
                        if key.starts_with("io")
                            || key.starts_with("ad")
                            || key.starts_with("pwm")
                            || key.starts_with("uart")
                            || key == "display"
                            || key == "switch"
                            || key == "system"
                        {
                            return Some(key.clone());
                        }
                    }
                }
            }
        }

        // Check for direct object responses (fallback)
        if let Some(obj) = message.as_object() {
            for (key, _) in obj {
                if key.starts_with("io")
                    || key.starts_with("ad")
                    || key.starts_with("pwm")
                    || key.starts_with("uart")
                    || key == "display"
                    || key == "switch"
                    || key == "system"
                {
                    return Some(key.clone());
                }
            }
        }

        None
    }

    pub fn send_message(&self, msg: Message) -> anyhow::Result<()> {
        self.sender
            .send(ObnizCommand::Send {
                message: msg,
                response_key: None,
            })
            .context("Failed to send command")
    }

    pub async fn send_await_response(
        &self,
        msg: Message,
        response_key: String,
    ) -> anyhow::Result<Value> {
        let (tx, rx) = oneshot::channel::<Value>();

        // Register callback for response
        self.sender
            .send(ObnizCommand::RegisterCallback {
                key: response_key.clone(),
                callback: CallbackType::OneShot(tx),
            })
            .context("Failed to register callback")?;

        // Send message
        self.sender
            .send(ObnizCommand::Send {
                message: msg,
                response_key: Some(response_key.clone()),
            })
            .context("Failed to send message")?;

        // Wait for response (the callback will be automatically removed after receiving)
        let result = rx.await.context("Failed to receive response")?;

        Ok(result)
    }

    pub fn register_callback<F>(&self, key: String, callback: F) -> anyhow::Result<()>
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        self.sender
            .send(ObnizCommand::RegisterCallback {
                key,
                callback: CallbackType::Persistent(Box::new(callback)),
            })
            .context("Failed to register callback")
    }

    pub fn unregister_callback(&self, key: String) -> anyhow::Result<()> {
        self.sender
            .send(ObnizCommand::UnregisterCallback { key })
            .context("Failed to unregister callback")
    }

    /// Get the IO manager for this Obniz device
    pub fn io(&self) -> IoManager {
        IoManager::new(self.clone())
    }

    /// Get the display manager for this Obniz device
    pub fn display(&self) -> DisplayManager {
        DisplayManager::new(self.clone())
    }

    /// Get the system manager for this Obniz device
    pub fn system(&self) -> SystemManager {
        SystemManager::new(self.clone())
    }

    /// Get the AD manager for this Obniz device
    pub fn ad(&self) -> AdManager {
        AdManager::new(self.clone())
    }

    /// Get the PWM manager for this Obniz device
    pub fn pwm(&self) -> PwmManager {
        PwmManager::new(self.clone())
    }

    /// Get the UART manager for this Obniz device
    pub fn uart(&self) -> UartManager {
        UartManager::new(self.clone())
    }

    /// Get the switch manager for this Obniz device
    pub fn switch(&self) -> SwitchManager {
        SwitchManager::new(self.clone())
    }

    /// Get the device ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

pub async fn connect_async(obniz_id: &str) -> anyhow::Result<Obniz> {
    let redirect_host = get_redirect_host(obniz_id).context("failed to get redirect host name")?;
    let api_url = endpoint_url(&redirect_host, obniz_id)?;
    Obniz::new(obniz_id, api_url)
        .await
        .context("failed to create Obniz object")
}

// Synchronous connect function is deprecated - use connect_async instead

fn endpoint_url(host: &str, obniz_id: &str) -> anyhow::Result<url::Url> {
    if !host.starts_with("wss://") {
        return Err(anyhow!("Illegal url, host needs to start with 'wss://'"));
    }

    let endpoint = format!("{host}/obniz/{obniz_id}/ws/1");
    url::Url::parse(&endpoint).context("Failed to parse endpoint url")
}

fn get_redirect_host(obniz_id: &str) -> anyhow::Result<String> {
    let url = endpoint_url(OBNIZE_WEBSOKET_HOST, obniz_id)?;
    //Websokcet接続
    let (mut ws_stream, _response) = tungstenite::connect(url.as_str()).context("Failed to connect")?;

    let message = ws_stream.read().context("Fail to read message")?;
    //　接続するとリダイレクトアドレスが入ったjsonが返るのでパースする
    let message = message.to_text().context("fail to parse text")?;

    let res: Value = serde_json::from_str(message).context("Failed to parse json")?;
    let json_redirect_host = &res[0]["ws"]["redirect"];
    let redirect_host = match json_redirect_host.as_str() {
        // ダブルクォートが入るので除去するためにstrに一旦する
        Some(host) => host.to_string(),
        None => return Err(anyhow!("Failed to get redirect host name")),
    };
    println!("redirect_host : {redirect_host}");
    if redirect_host.is_empty() {
        return Err(anyhow!("Redirect host name is empty"));
    }
    if !redirect_host.starts_with("wss://") {
        return Err(anyhow!("Redirect host name is bad format"));
    }

    Ok(redirect_host)
}

// Legacy enums moved to display module - kept here for backward compatibility
pub use crate::display::{DisplayRawColorDepth, ObnizDisplay, QrCorrectionType};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// The following modules are now implemented in separate files:
// - IO: src/io.rs
// - AD: src/ad.rs
// - PWM: src/pwm.rs
// - UART: src/uart.rs
// - Switch: src/switch.rs
// - Display: src/display.rs
// - System: src/system.rs

// Future features (not yet implemented):
// - IoAnimation: Complex IO state animations
// - TCP: Network communication
// - SPI: Serial Peripheral Interface
// - I2C: Inter-Integrated Circuit
// - BLE HCI: Bluetooth Low Energy
// - WiFi: WiFi management
// - Logic Analyzer: Digital signal analysis
// - Measurement: Advanced measurement tools
