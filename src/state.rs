use moka::future::Cache;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::config::Config;

/// State for a login that is in-progress (waiting for captcha solve).
pub struct InProgressLogin {
    pub client: Client,
    pub csrf_token: String,
    pub created_at: std::time::Instant,
}

/// State for a fully authenticated session.
pub struct AuthSession {
    pub client: Client,
    pub csrf_token: String,
    pub authorized_id: String,
    pub created_at: std::time::Instant,
    /// Mutex to ensure requests to VTOP for this session are strictly serialized and throttled.
    pub request_lock: Arc<Mutex<()>>,
}

/// Shared application state, injected into every handler via Axum's State extractor.
///
/// IMPORTANT: No credentials are ever stored in AppState.
/// The `client` inside each session carries a cookie jar for per-user session management.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration (host, port, base URL, etc.).
    pub config: Arc<Config>,

    /// Secret key for JWT issuing and verification.
    pub jwt_secret: Arc<String>,

    /// Logins currently waiting for the user to solve the captcha.
    /// Keyed by a UUID (`login_session_id`).
    pub login_sessions: Arc<RwLock<HashMap<String, InProgressLogin>>>,

    /// Authenticated sessions.
    /// Keyed by the JWT's `sub` claim (user UUID).
    pub auth_sessions: Arc<RwLock<HashMap<String, AuthSession>>>,

    /// Per-endpoint cache to prevent aggressive repetitive scraping.
    pub cache: Cache<String, serde_json::Value>,
}

impl AppState {
    /// Build shared state from the loaded config.
    pub fn new(config: Config) -> Self {
        // For security, generate a random JWT secret on every startup.
        // This ensures all sessions are invalidated if the API restarts.
        use rand::RngExt;
        let jwt_secret: String = rand::rng().sample_iter(&rand::distr::Alphanumeric).take(32).map(char::from).collect();

        let cache = Cache::builder()
            .time_to_live(std::time::Duration::from_secs(180)) // 3 minutes TTL
            .build();

        Self {
            config: Arc::new(config),
            jwt_secret: Arc::new(jwt_secret),
            login_sessions: Arc::new(RwLock::new(HashMap::new())),
            auth_sessions: Arc::new(RwLock::new(HashMap::new())),
            cache,
        }
    }

    /// Helper to create a new reqwest client configured for VTOP.
    pub fn new_client() -> Client {
        Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                 AppleWebKit/537.36 (KHTML, like Gecko) \
                 Chrome/126.0.0.0 Safari/537.36",
            )
            .build()
            .expect("Failed to build HTTP client")
    }
}
