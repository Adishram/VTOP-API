use axum::routing::{get, post};
use axum::Router;

use crate::handlers::{academic, auth};
use crate::state::AppState;

/// Build the full API router with all routes and shared state.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // ── Auth ──────────────────────────────────
        .route("/api/login", post(auth::login))
        .route("/api/login/verify", post(auth::login_verify))
        // ── Academic data ─────────────────────────
        .route("/api/attendance", get(academic::attendance))
        .route("/api/timetable", get(academic::timetable))
        .route("/api/marks", get(academic::marks))
        .route("/api/exams", get(academic::exams))
        .route("/api/grades", get(academic::grades))
        .route("/api/calendar", get(academic::calendar))
        .route("/api/courses", get(academic::courses))
        .route("/api/faculty", get(academic::faculty))
        // ── Shared state ──────────────────────────
        .with_state(state)
}
