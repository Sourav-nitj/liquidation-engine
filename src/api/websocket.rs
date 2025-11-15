use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

use crate::engine::EngineState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<EngineState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<EngineState>) {
    let mut rx = state.event_tx.subscribe();

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        let _ = socket.send(Message::Text(format!("echo: {}", t))).await;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => break,
                }
            }

            evt = rx.recv() => {
                if let Ok(event) = evt {
                    if let Ok(text) = serde_json::to_string(&event) {
                        let _ = socket.send(Message::Text(text)).await;
                    }
                }
            }
        }
    }
}
