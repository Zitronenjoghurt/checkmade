use crate::api::error::ApiResult;
use crate::state::ServerState;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;
use tower_sessions::Session;

mod discord;

async fn post_logout(session: Session) -> ApiResult<impl IntoResponse> {
    session.delete().await?;
    Ok(())
}

pub fn router() -> Router<ServerState> {
    Router::new()
        .nest("/discord", discord::router())
        .route("/logout", post(post_logout))
}
