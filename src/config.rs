use std::env;

/// Application configuration loaded from environment variables.
/// No secrets are stored here — only public configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub vtop_base_url: String,
    pub session_ttl_secs: u64,
}

impl Config {
    /// Load configuration from environment variables.
    /// Falls back to sane defaults if variables are missing.
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            vtop_base_url: env::var("VTOP_BASE_URL")
                .unwrap_or_else(|_| "https://vtopcc.vit.ac.in/vtop".into()),
            session_ttl_secs: env::var("SESSION_TTL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1800),
        }
    }

    /// Returns the socket address string for binding.
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
