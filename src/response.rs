use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

// ──────────────────────────────────────────────
// Consistent JSON response envelope
// { "status": "ok" | "error", "data": ..., "error": null | string }
// ──────────────────────────────────────────────

/// The standard API envelope that wraps every response.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub status: &'static str,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Wrap a successful result.
    pub fn ok(data: T) -> Self {
        Self {
            status: "ok",
            data: Some(data),
            error: None,
        }
    }
}

impl ApiResponse<()> {
    /// Wrap an error message (no data payload).
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            status: "error",
            data: None,
            error: Some(msg.into()),
        }
    }
}

// ──────────────────────────────────────────────
// ApiError — converts into the envelope shape
// ──────────────────────────────────────────────

/// Structured API error that produces the envelope on serialisation.
#[derive(Debug)]
pub struct ApiError {
    pub code: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(code: StatusCode, msg: impl Into<String>) -> Self {
        Self {
            code,
            message: msg.into(),
        }
    }

    /// Convenience: 501 Not Implemented (used by stubs).
    pub fn not_implemented() -> Self {
        Self::new(StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
    }

    /// Convenience: 401 Unauthorized.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, msg)
    }

    /// Convenience: 500 Internal Server Error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "status": "error",
            "data": null,
            "error": self.message,
        });
        (self.code, axum::Json(body)).into_response()
    }
}
