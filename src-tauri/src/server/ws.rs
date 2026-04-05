use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use tracing::{debug, error, instrument};

use crate::state::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

#[instrument(level = "debug", skip_all)]
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.hr_broadcast.subscribe();

    debug!("websocket client connected");

    if let Some(current) = state.current_data.read().await.as_ref() {
        if let Ok(json) = serde_json::to_string(current) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
    }

    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(json) => {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    error!(error = %e, "failed to serialize hr event");
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    debug!("websocket client disconnected");
}
