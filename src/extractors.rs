use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use reqwest::Client;

use crate::{
    handlers::auth::Claims,
    response::ApiError,
    state::AppState,
};

/// The authenticated context extracted from a valid JWT.
/// Contains the cloned HTTP client (which holds the session cookie jar),
/// the current CSRF token, and the student's authorized ID.
pub struct AuthContext {
    pub client: Client,
    pub csrf_token: String,
    pub authorized_id: String,
    pub request_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
}

impl FromRequestParts<AppState> for AuthContext {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // 1. Extract the Authorization header
        let auth_header = parts.headers.get("Authorization")
            .and_then(|h| h.to_str().ok())
            .filter(|s| s.starts_with("Bearer "))
            .map(|s| s[7..].to_string())
            .ok_or_else(|| ApiError::unauthorized("Missing or invalid Authorization header"))?;

        // 2. Decode the JWT to get the `sub` claim
        let token_data = decode::<Claims>(
            &auth_header,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        ).map_err(|_| ApiError::unauthorized("Invalid or expired token"))?;

        let user_id = token_data.claims.sub;

        // 3. Look up the session in the AppState
        let map = state.auth_sessions.read().await;
        let session = map.get(&user_id)
            .ok_or_else(|| ApiError::unauthorized("Session expired or invalid"))?;

        Ok(AuthContext {
            client: session.client.clone(),
            csrf_token: session.csrf_token.clone(),
            authorized_id: session.authorized_id.clone(),
            request_lock: session.request_lock.clone(),
        })
    }
}
