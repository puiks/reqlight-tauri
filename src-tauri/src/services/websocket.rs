use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

use crate::error::AppError;

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

    /// Connect to a WebSocket URL with optional custom headers.
    /// Spawns a background task to receive messages and emit them as Tauri events.
    pub async fn connect<F>(
        &self,
        connection_id: String,
        url: &str,
        headers: &[(String, String)],
        emit_fn: F,
    ) -> Result<(), AppError>
    where
        F: Fn(WsEvent) + Send + Sync + 'static,
    {
        tracing::info!(%url, %connection_id, "Opening WebSocket connection");

        // Validate URL before connecting
        let _ = url::Url::parse(url)
            .map_err(|e| AppError::Validation(format!("Invalid WebSocket URL: {e}")))?;

        // Build request with custom headers
        let mut request = url
            .into_client_request()
            .map_err(|e| AppError::Validation(format!("Invalid WebSocket URL: {e}")))?;
        for (key, value) in headers {
            if let (Ok(name), Ok(val)) = (
                key.parse::<tokio_tungstenite::tungstenite::http::HeaderName>(),
                value.parse::<tokio_tungstenite::tungstenite::http::HeaderValue>(),
            ) {
                request.headers_mut().insert(name, val);
            }
        }

        let (ws_stream, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| AppError::WebSocket(format!("Connection failed: {e}")))?;

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
    pub async fn send(&self, connection_id: &str, message: &str) -> Result<(), AppError> {
        let mut conns = self.connections.lock().await;
        let conn = conns
            .get_mut(connection_id)
            .ok_or_else(|| AppError::WebSocket(format!("No active connection: {connection_id}")))?;

        conn.sender
            .send(Message::Text(message.to_string()))
            .await
            .map_err(|e| AppError::WebSocket(format!("Failed to send message: {e}")))
    }

    /// Disconnect an active connection.
    pub async fn disconnect(&self, connection_id: &str) -> Result<(), AppError> {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.remove(connection_id) {
            let _ = conn.cancel.send(true);
            Ok(())
        } else {
            Err(AppError::WebSocket(format!(
                "No active connection: {connection_id}"
            )))
        }
    }
}

#[cfg(test)]
#[path = "websocket_tests.rs"]
mod tests;
