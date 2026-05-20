use crate::state::ServerState;
use axum::Router;

mod auth;
mod me;

pub fn build_routes() -> Router<ServerState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/me", me::router())
}
