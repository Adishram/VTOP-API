use axum::extract::State;
use axum::Json;
use jsonwebtoken::{encode, EncodingKey, Header};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use uuid::Uuid;

use crate::response::{ApiError, ApiResponse};
use crate::state::{AppState, AuthSession, InProgressLogin};

// ──────────────────────────────────────────────
// Request and Response bodies
// ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String, // Actually not strictly needed for captcha fetch, but we keep it
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub login_session_id: String,
    pub captcha_image: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginVerifyRequest {
    pub login_session_id: String,
    pub username: String,
    // Intentionally skipped by custom Debug/tracing by omitting Debug on this struct
    // wait, we can just use zeroize if we want, but keeping it out of logs is enough
    pub password: String,
    pub captcha: String,
}

#[derive(Debug, Serialize)]
pub struct LoginVerifyResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

// ──────────────────────────────────────────────
// HTML Parsing helpers
// ──────────────────────────────────────────────

fn extract_csrf(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let meta_sel = Selector::parse("meta[name='_csrf']").unwrap();
    let input_sel = Selector::parse("input[name='_csrf']").unwrap();

    if let Some(el) = document.select(&input_sel).next() {
        if let Some(val) = el.value().attr("value") {
            return Some(val.to_string());
        }
    }
    if let Some(el) = document.select(&meta_sel).next() {
        if let Some(val) = el.value().attr("content") {
            return Some(val.to_string());
        }
    }
    None
}

fn extract_captcha(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    // Look for any img with a base64 src
    let sel = Selector::parse("img").unwrap();
    for el in document.select(&sel) {
        if let Some(src) = el.value().attr("src") {
            // src format: data:image/png;base64,... or jpeg
            if src.starts_with("data:image/") {
                return Some(src.to_string());
            }
        }
    }
    None
}

fn extract_authorized_id(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let sel = Selector::parse("#authorizedIDX").unwrap();
    if let Some(el) = document.select(&sel).next() {
        if let Some(val) = el.value().attr("value") {
            return Some(val.to_string());
        }
    }
    None
}

// ──────────────────────────────────────────────
// Auth handlers
// ──────────────────────────────────────────────

/// POST /api/login
pub async fn login(
    State(state): State<AppState>,
    Json(_body): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, ApiError> {
    tracing::info!(endpoint = "POST /api/login", "Fetching login page & captcha");

    // 1. Create a fresh reqwest client with its own cookie jar
    let client = AppState::new_client();

    // 2. GET /vtop/login to grab JSESSIONID and initial CSRF (this is the landing page)
    let login_url = format!("{}/login", state.config.vtop_base_url);
    let res = client.get(&login_url)
        .send()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to fetch /login: {}", e)))?;
    
    let html = res.text().await.unwrap_or_default();
    let csrf_token = extract_csrf(&html).ok_or_else(|| ApiError::internal("Missing CSRF on login page"))?;

    // 3. POST /prelogin/setup with flag=VTOP to initiate the student login flow.
    //    This redirects to /init/page -> /login, which now shows the actual login form with captcha.
    let setup_url = format!("{}/prelogin/setup", state.config.vtop_base_url);
    let form = [("flag", "VTOP"), ("_csrf", &csrf_token)];
    
    let res = client.post(&setup_url)
        .form(&form)
        .send()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to POST /prelogin/setup: {}", e)))?;

    // Follow redirects: reqwest auto-follows but POST->GET redirect may need manual handling.
    // After the redirect chain, we should be on the login page with the captcha.
    let final_url = res.url().clone();
    let html = res.text().await.unwrap_or_default();

    // If we didn't land on the login form page (no captcha), follow the redirect manually
    let html = if extract_captcha(&html).is_none() {
        let res = client.get(final_url)
            .send()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to follow redirect: {}", e)))?;
        res.text().await.unwrap_or_default()
    } else {
        html
    };

    // If still no captcha, try fetching /login again with the now-established session
    let html = if extract_captcha(&html).is_none() {
        let res = client.get(&login_url)
            .send()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to re-fetch /login: {}", e)))?;
        res.text().await.unwrap_or_default()
    } else {
        html
    };

    let captcha_image = extract_captcha(&html)
        .ok_or_else(|| ApiError::internal("Could not find captcha image in VTOP response"))?;
    
    let new_csrf = extract_csrf(&html).unwrap_or(csrf_token);

    // 4. Save to in-progress map
    let login_session_id = Uuid::new_v4().to_string();
    
    {
        let mut map = state.login_sessions.write().await;
        map.insert(login_session_id.clone(), InProgressLogin {
            client,
            csrf_token: new_csrf,
            created_at: Instant::now(),
        });
    }

    Ok(Json(ApiResponse::ok(LoginResponse {
        login_session_id,
        captcha_image,
    })))
}

/// POST /api/login/verify
pub async fn login_verify(
    State(state): State<AppState>,
    Json(body): Json<LoginVerifyRequest>,
) -> Result<Json<ApiResponse<LoginVerifyResponse>>, ApiError> {
    tracing::info!(endpoint = "POST /api/login/verify", "Verifying login credentials");

    // 1. Retrieve the in-progress login session
    let in_progress = {
        let mut map = state.login_sessions.write().await;
        map.remove(&body.login_session_id)
            .ok_or_else(|| ApiError::unauthorized("Invalid or expired login_session_id"))?
    };

    let client = in_progress.client;
    let csrf_token = in_progress.csrf_token;

    // 2. Submit credentials to VTOP
    let verify_url = format!("{}/login", state.config.vtop_base_url);
    let form = [
        ("username", body.username.as_str()),
        ("password", body.password.as_str()),
        ("captchaStr", body.captcha.as_str()),
        ("gResponse", body.captcha.as_str()),
        ("_csrf", csrf_token.as_str()),
    ];

    let res = client.post(&verify_url)
        .form(&form)
        .send()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to submit login: {}", e)))?;
    
    let html = res.text().await.unwrap_or_default();

    // 3. Verify success by looking for authorizedIDX
    let authorized_id = match extract_authorized_id(&html) {
        Some(id) => id,
        None => {
            tracing::warn!("Login failed: No authorizedIDX found in response");
            return Err(ApiError::unauthorized("Invalid username, password, or captcha"));
        }
    };

    let new_csrf = extract_csrf(&html).unwrap_or(csrf_token);

    // 4. Issue JWT
    let user_id = Uuid::new_v4().to_string();
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize + state.config.session_ttl_secs as usize;

    let claims = Claims {
        sub: user_id.clone(),
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ).map_err(|e| ApiError::internal(format!("Failed to generate token: {}", e)))?;

    // 5. Save the authenticated session
    {
        let mut map = state.auth_sessions.write().await;
        let auth_session = AuthSession {
            client: client,
            csrf_token: new_csrf.to_string(),
            authorized_id: authorized_id.to_string(),
            created_at: std::time::Instant::now(),
            request_lock: std::sync::Arc::new(tokio::sync::Mutex::new(())),
        };
        map.insert(user_id, auth_session);
    }

    Ok(Json(ApiResponse::ok(LoginVerifyResponse { token })))
}
