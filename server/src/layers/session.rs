use crate::state::ServerState;
use tower_sessions::cookie::time::Duration;
use tower_sessions::cookie::{Key, SameSite};
use tower_sessions::service::SignedCookie;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

pub async fn build_session_layer(
    state: &ServerState,
) -> SessionManagerLayer<PostgresStore, SignedCookie> {
    let session_store = PostgresStore::new(state.data.pool().clone());
    session_store.migrate().await.unwrap();

    let key = Key::from(state.config.session_secret.as_bytes());

    if state.config.dev_mode {
        SessionManagerLayer::new(session_store)
            .with_expiry(Expiry::OnInactivity(Duration::days(14)))
            .with_same_site(SameSite::Lax)
            .with_signed(key)
            .with_secure(false)
    } else {
        SessionManagerLayer::new(session_store)
            .with_expiry(Expiry::OnInactivity(Duration::days(14)))
            .with_same_site(SameSite::Lax)
            .with_signed(key)
            .with_secure(true)
            .with_domain(state.config.domain.clone())
    }
}
