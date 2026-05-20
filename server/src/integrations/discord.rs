use crate::error::{ServerError, ServerResult};
use oauth2::basic::BasicTokenResponse;
use oauth2::reqwest::Url;
use oauth2::{
    reqwest, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
};
use std::fmt::Display;

pub struct DiscordIntegration {
    api_client: reqwest::Client,
    oauth_client: DiscordOauthClient,
}

impl DiscordIntegration {
    pub fn new(config: &DiscordConfig) -> ServerResult<Self> {
        let oauth_client = Client::new(ClientId::new(config.client_id.clone()))
            .set_client_secret(ClientSecret::new(config.client_secret.clone()))
            .set_auth_uri(AuthUrl::new(config.auth_url.clone())?)
            .set_token_uri(TokenUrl::new(config.token_url.clone())?)
            .set_redirect_uri(RedirectUrl::new(config.callback_url.clone())?);
        Ok(Self {
            api_client: reqwest::Client::new(),
            oauth_client,
        })
    }

    pub fn oauth_authorize(&self, pkce_challenge: PkceCodeChallenge) -> (Url, CsrfToken) {
        self.oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("identify".into()))
            .set_pkce_challenge(pkce_challenge)
            .url()
    }

    pub async fn oauth_token(
        &self,
        code: AuthorizationCode,
        pkce_verifier: PkceCodeVerifier,
    ) -> ServerResult<BasicTokenResponse> {
        self.oauth_client
            .exchange_code(code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(&self.api_client)
            .await
            .map_err(|e| ServerError::TokenRequest(e.to_string()))
    }

    pub async fn get_user(&self, token: impl Display) -> ServerResult<Option<DiscordUser>> {
        let response = self
            .api_client
            .get("https://discord.com/api/users/@me")
            .bearer_auth(token)
            .send()
            .await?;

        if response.status() == 404 {
            return Ok(None);
        }

        let bytes = response.bytes().await?;
        let user: DiscordUser = serde_json::from_slice(&bytes)?;
        Ok(Some(user))
    }
}

type DiscordOauthClient = Client<
    oauth2::basic::BasicErrorResponse,
    oauth2::basic::BasicTokenResponse,
    oauth2::basic::BasicTokenIntrospectionResponse,
    oauth2::StandardRevocableToken,
    oauth2::basic::BasicRevocationErrorResponse,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
>;

#[derive(serde::Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
}

pub struct DiscordConfig {
    client_id: String,
    client_secret: String,
    auth_url: String,
    token_url: String,
    callback_url: String,
}

impl DiscordConfig {
    pub fn from_env() -> ServerResult<Self> {
        Ok(Self {
            client_id: std::env::var("DISCORD_CLIENT_ID")?,
            client_secret: std::env::var("DISCORD_CLIENT_SECRET")?,
            auth_url: std::env::var("DISCORD_AUTH_URL")?,
            token_url: std::env::var("DISCORD_TOKEN_URL")?,
            callback_url: std::env::var("DISCORD_CALLBACK_URL")?,
        })
    }
}
