use crate::api::error::{ApiError, ApiResult};
use crate::state::ServerState;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;
use oauth2::{AuthorizationCode, PkceCodeChallenge, PkceCodeVerifier, TokenResponse};
use petname::petname;
use tower_sessions::Session;

#[derive(serde::Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}

async fn get_login(session: Session, State(state): State<ServerState>) -> ApiResult<Redirect> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf) = state.integrations.discord.oauth_authorize(pkce_challenge);

    session.insert("oauth_discord_csrf", csrf).await?;
    session
        .insert("oauth_discord_pkce_verifier", pkce_verifier)
        .await?;
    session.save().await?;

    Ok(Redirect::to(auth_url.as_str()))
}

async fn get_callback(
    session: Session,
    State(state): State<ServerState>,
    Query(query): Query<AuthRequest>,
) -> ApiResult<Redirect> {
    let csrf = session
        .remove::<String>("oauth_discord_csrf")
        .await?
        .ok_or(ApiError::MissingCsrfToken)?;
    let pkce_verifier = session
        .remove::<String>("oauth_discord_pkce_verifier")
        .await?
        .ok_or(ApiError::MissingPkceVerifier)?;

    if query.state != csrf {
        return Err(ApiError::InvalidCsrfToken);
    };

    let code = AuthorizationCode::new(query.code);
    let pkce_verifier = PkceCodeVerifier::new(pkce_verifier);
    let token = state
        .integrations
        .discord
        .oauth_token(code, pkce_verifier)
        .await
        .map_err(|_| ApiError::OauthTokenExchange)?;

    let Some(discord_user) = state
        .integrations
        .discord
        .get_user(token.access_token().secret())
        .await
        .map_err(|e| ApiError::DiscordUnreachable(e.to_string()))?
    else {
        return Err(ApiError::DiscordUserInvalid);
    };

    let user = if let Some(user) = state.data.user.find_by_discord_id(&discord_user.id).await? {
        user
    } else {
        let name = petname(3, "-").unwrap();
        state
            .data
            .user
            .create_from_discord(&discord_user.id, name)
            .await?
    };

    session.insert("user_id", user.id).await?;
    session.save().await?;

    Ok(Redirect::to("/"))
}

pub fn router() -> Router<ServerState> {
    Router::<ServerState>::new()
        .route("/login", get(get_login))
        .route("/callback", get(get_callback))
}
