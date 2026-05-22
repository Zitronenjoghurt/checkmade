use crate::api::error::{ApiError, ApiResult};
use crate::state::ServerState;
use crate::websocket::connection::{ConnectionId, WebsocketConnection};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use checkmade_core::data::store::Store;
use checkmade_core::data::{chrono_now, IntoActiveModel, Set};
use checkmade_core::messages::server::ServerMessage;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use metrics::{counter, histogram};
use tokio::sync::mpsc;
use tower_sessions::Session;
use tracing::error;
use uuid::Uuid;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    session: Session,
    State(state): State<ServerState>,
) -> ApiResult<impl IntoResponse> {
    let Some(user_id) = session.get::<Uuid>("user_id").await? else {
        counter!("ws.upgrade_rejected", "reason" => "unauthorized").increment(1);
        return Err(ApiError::Unauthorized);
    };

    let Some(user) = state.data.user.find_by_id(user_id).await? else {
        counter!("ws.upgrade_rejected", "reason" => "unauthorized").increment(1);
        return Err(ApiError::Unauthorized);
    };

    let connection_count = state.ws.user_connection_count(user.id);
    if connection_count >= state.config.max_user_connection_count {
        counter!("ws.upgrade_rejected", "reason" => "too_many_connections").increment(1);
        return Err(ApiError::TooManyConnections);
    }

    if user.rate_limit_infractions >= 3 {
        counter!("ws.upgrade_rejected", "reason" => "banned").increment(1);
        return Err(ApiError::BannedRateLimitAbuse);
    }

    counter!("ws.upgrade_accepted").increment(1);

    let last_login = user.last_login;

    let mut active_user = user.into_active_model();
    active_user.last_login = Set(chrono_now());
    let user = state.data.user.update(active_user).await?;

    let since_last_login = chrono_now()
        .signed_duration_since(last_login)
        .num_milliseconds();
    if since_last_login < 3000 {
        tokio::time::sleep(std::time::Duration::from_millis(
            3000 - since_last_login as u64,
        ))
        .await;
    }

    let message_size = state.config.max_ws_message_size_kb * 1024;
    let write_buffer_size = state.config.max_ws_outbound_buffer_size_kb * 1024;
    Ok(ws
        .max_message_size(message_size)
        .max_frame_size(message_size)
        .write_buffer_size(write_buffer_size)
        .on_upgrade(move |socket| handle_socket(socket, state.clone(), user.id)))
}

async fn handle_socket(socket: WebSocket, state: ServerState, user_id: Uuid) {
    let start = std::time::Instant::now();

    let (ws_send, ws_receive) = socket.split();
    let (connection_id, rx) = state.ws.register(user_id);

    let state_clone = state.clone();
    let send_fut = handle_send(user_id, ws_send, rx);
    let recv_fut = WebsocketConnection::new(&state.config, connection_id, user_id, state_clone)
        .handle_receive(ws_receive);

    tokio::select! {
        _ = send_fut => {}
        _ = recv_fut => {}
    }

    state.ws.unregister(connection_id);

    histogram!("ws.connection_duration_secs").record(start.elapsed().as_secs_f64());
}

async fn handle_send(
    connection_id: ConnectionId,
    mut ws_send: SplitSink<WebSocket, Message>,
    mut rx: mpsc::Receiver<ServerMessage>,
) {
    while let Some(message) = rx.recv().await {
        let msg_type = message.name();
        let encoded = message.as_bytes();
        let msg_size = encoded.len();
        if let Err(err) = ws_send.send(Message::binary(encoded)).await {
            error!("[{connection_id}] Failed to send message: {err}");
            break;
        } else {
            counter!("ws.outbound_total", "type" => msg_type).increment(1);
            histogram!("ws.outbound_size_bytes").record(msg_size as f64);
        }
    }
}
