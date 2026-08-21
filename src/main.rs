use vtop_api::{config, router, state};

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // ── Load .env (silently skip if missing) ──────────
    dotenvy::dotenv().ok();

    // ── Structured logging via tracing ────────────────
    // RUST_LOG controls verbosity, e.g. RUST_LOG=vtop_api=debug,tower_http=debug
    //
    // SECURITY: The tracing layer logs structured fields only.
    //           Request bodies (which may contain passwords) are NEVER logged.
    //           Only endpoint names and metadata appear in logs.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("vtop_api=info")),
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // ── Config ────────────────────────────────────────
    let config = config::Config::from_env();
    let addr = config.addr();

    tracing::info!(
        host = %config.host,
        port = %config.port,
        vtop_base_url = %config.vtop_base_url,
        session_ttl_secs = %config.session_ttl_secs,
        "Configuration loaded"
    );

    // ── Shared state ──────────────────────────────────
    let state = state::AppState::new(config);

    // ── Router ────────────────────────────────────────
    let app = router::build_router(state);

    // ── Bind & serve ──────────────────────────────────
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!(addr = %addr, "🚀 vtop-api listening");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
