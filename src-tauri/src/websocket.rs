use tauri::{AppHandle, Manager};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketMessage {
    LlmStatus {
        llm_id: String,
        status: String,
        current_task: Option<String>,
    },
    LlmResponse {
        llm_id: String,
        content: String,
        is_final: bool,
    },
    DocumentIndexed {
        document_id: String,
        chunk_count: usize,
    },
    AuditLogEntry {
        action: String,
        approved: bool,
    },
    LockdownTriggered {
        reason: String,
    },
}

pub async fn start_server(app: AppHandle) -> anyhow::Result<()> {
    let addr = "127.0.0.1:3030";
    let listener = TcpListener::bind(addr).await?;

    info!("🌐 WebSocket server listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let app_handle = app.clone();

        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    debug!("✅ New WebSocket connection");
                    handle_connection(ws_stream, app_handle).await;
                }
                Err(e) => error!("❌ WebSocket error: {}", e),
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    app: AppHandle,
) {
    let (mut write, mut read) = ws_stream.split();

    // Send initial connection message
    let msg = WebSocketMessage::LlmStatus {
        llm_id: "system".to_string(),
        status: "connected".to_string(),
        current_task: None,
    };

    if let Ok(json) = serde_json::to_string(&msg) {
        let _ = write.send(tokio_tungstenite::tungstenite::Message::Text(json)).await;
    }

    // Listen for messages from client
    while let Some(msg) = read.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                debug!("📨 Received: {}", text);
                // TODO: Handle incoming messages
            }
            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                debug!("👋 WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("❌ WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Broadcast a message to all connected WebSocket clients
pub async fn broadcast_message(_app: &AppHandle, _message: WebSocketMessage) {
    // TODO: Implement broadcast to all connected clients
    // Will need to maintain a list of active connections
}
