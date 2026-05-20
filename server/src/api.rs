use crate::state::ServerState;
use axum::Router;

pub mod error;
mod routes;

pub fn build() -> Router<ServerState> {
    Router::new().merge(routes::build_routes())
}
