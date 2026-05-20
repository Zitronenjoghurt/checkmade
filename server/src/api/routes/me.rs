use crate::api::error::{ApiError, ApiResult};
use crate::state::ServerState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use checkmade_core::data::store::Store;
use tower_sessions::Session;
use tower_sessions_sqlx_store::sqlx::types::Uuid;

async fn get_me(
    session: Session,
    State(state): State<ServerState>,
) -> ApiResult<impl IntoResponse> {
    let Some(user_id) = session.get::<Uuid>("user_id").await? else {
        return Err(ApiError::Unauthorized);
    };

    let Some(user) = state.data.user.find_by_id(user_id).await? else {
        return Err(ApiError::Unauthorized);
    };

    let info = state.service.user.private_info(&user);
    Ok(axum::Json(info).into_response())
}

pub fn router() -> Router<ServerState> {
    Router::<ServerState>::new().route("/", get(get_me))
}
