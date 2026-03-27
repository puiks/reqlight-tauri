use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

/// Payload emitted to the frontend via Tauri events.
#[derive(Clone, Serialize)]
pub struct WsEvent {
    pub connection_id: String,
    pub event_type: WsEventType,
    pub data: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WsEventType {
    Message,
    Connected,
    Disconnected,
    Error,
}

type WsSender = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

struct Connection {
    sender: WsSender,
    cancel: tokio::sync::watch::Sender<bool>,
}

/// Manages active WebSocket connections.
pub struct WsManager {
    connections: Arc<Mutex<HashMap<String, Connection>>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Connect to a WebSocket URL. Spawns a background task to receive messages
    /// and emit them as Tauri events. Returns the connection ID.
    pub async fn connect<F>(
        &self,
        connection_id: String,
        url: &str,
        emit_fn: F,
    ) -> Result<(), String>
    where
        F: Fn(WsEvent) + Send + Sync + 'static,
    {
        // Validate URL before connecting
        let _ = url::Url::parse(url).map_err(|e| format!("Invalid WebSocket URL: {e}"))?;

        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {e}"))?;

        let (sender, mut receiver) = ws_stream.split();
        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);

        {
            let mut conns = self.connections.lock().await;
            conns.insert(
                connection_id.clone(),
                Connection {
                    sender,
                    cancel: cancel_tx,
                },
            );
        }

        let id = connection_id.clone();
        emit_fn(WsEvent {
            connection_id: id.clone(),
            event_type: WsEventType::Connected,
            data: None,
        });

        // Background task to receive messages
        let conns = self.connections.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = receiver.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                emit_fn(WsEvent {
                                    connection_id: id.clone(),
                                    event_type: WsEventType::Message,
                                    data: Some(text.to_string()),
                                });
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                break;
                            }
                            Some(Err(e)) => {
                                emit_fn(WsEvent {
                                    connection_id: id.clone(),
                                    event_type: WsEventType::Error,
                                    data: Some(e.to_string()),
                                });
                                break;
                            }
                            _ => {} // Ping/Pong/Binary ignored
                        }
                    }
                    _ = cancel_rx.changed() => {
                        break;
                    }
                }
            }

            // Cleanup
            conns.lock().await.remove(&id);
            emit_fn(WsEvent {
                connection_id: id,
                event_type: WsEventType::Disconnected,
                data: None,
            });
        });

        Ok(())
    }

    /// Send a text message on an active connection.
    pub async fn send(&self, connection_id: &str, message: &str) -> Result<(), String> {
        let mut conns = self.connections.lock().await;
        let conn = conns
            .get_mut(connection_id)
            .ok_or_else(|| format!("No active connection: {connection_id}"))?;

        conn.sender
            .send(Message::Text(message.to_string()))
            .await
            .map_err(|e| format!("Failed to send message: {e}"))
    }

    /// Disconnect an active connection.
    pub async fn disconnect(&self, connection_id: &str) -> Result<(), String> {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.remove(connection_id) {
            let _ = conn.cancel.send(true);
            Ok(())
        } else {
            Err(format!("No active connection: {connection_id}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::accept_async;

    /// Start a simple echo WebSocket server on a random port.
    async fn start_echo_server() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("ws://{addr}");

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    if let Ok(ws) = accept_async(stream).await {
                        let (mut tx, mut rx) = ws.split();
                        while let Some(Ok(msg)) = rx.next().await {
                            if msg.is_text() {
                                let _ = tx.send(msg).await;
                            }
                        }
                    }
                });
            }
        });

        // Give server time to bind
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        url
    }

    #[tokio::test]
    async fn connect_and_send_message() {
        let url = start_echo_server().await;
        let manager = WsManager::new();

        let received = Arc::new(Mutex::new(Vec::<WsEvent>::new()));
        let received_clone = received.clone();

        let emit = move |event: WsEvent| {
            let r = received_clone.clone();
            tokio::spawn(async move {
                r.lock().await.push(event);
            });
        };

        manager
            .connect("test-1".to_string(), &url, emit)
            .await
            .unwrap();

        // Send a message
        manager.send("test-1", "hello").await.unwrap();

        // Wait for echo
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let events = received.lock().await;
        assert!(events
            .iter()
            .any(|e| matches!(e.event_type, WsEventType::Connected)));
        assert!(events
            .iter()
            .any(|e| matches!(e.event_type, WsEventType::Message)
                && e.data.as_deref() == Some("hello")));
    }

    #[tokio::test]
    async fn disconnect_closes_connection() {
        let url = start_echo_server().await;
        let manager = WsManager::new();

        let emit = |_event: WsEvent| {};

        manager
            .connect("test-2".to_string(), &url, emit)
            .await
            .unwrap();

        manager.disconnect("test-2").await.unwrap();

        // Send should fail after disconnect
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let result = manager.send("test-2", "should fail").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn invalid_url_returns_error() {
        let manager = WsManager::new();
        let emit = |_event: WsEvent| {};
        let result = manager
            .connect("test-3".to_string(), "not-a-url", emit)
            .await;
        assert!(result.is_err());
    }
}
