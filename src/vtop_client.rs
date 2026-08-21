use std::time::Duration;
use tokio::time::sleep;

use crate::extractors::AuthContext;
use crate::response::ApiError;

/// Centralized helper to fetch HTML from VTOP with resilience built-in.
/// 
/// Features:
/// 1. Strict per-session throttling (500ms spacing).
/// 2. 10-second timeout per request.
/// 3. One automatic retry on network failure or timeout.
/// 4. Explicit detection of VTOP session expiry.
pub async fn fetch_html(
    auth: &AuthContext,
    url: &str,
    form_data: &[(&str, &str)],
) -> Result<String, ApiError> {
    // 1. Acquire the per-session lock to serialize requests
    let _guard = auth.request_lock.lock().await;

    // Retry loop (max 2 attempts)
    let mut attempts = 0;
    let mut last_error = String::new();

    while attempts < 2 {
        attempts += 1;

        // Perform the POST request with a 10s timeout overrides (client has 30s default)
        let res_result = auth.client.post(url)
            .form(form_data)
            .timeout(Duration::from_secs(10))
            .send()
            .await;

        match res_result {
            Ok(res) => {
                if res.status().is_success() {
                    let html = res.text().await.unwrap_or_default();

                    // VTOP Expiry Detection
                    if html.contains("Session timed out") || html.contains("Not Authorized") || html.contains("var errMsg = 'You are logged out'") {
                        // Release lock before returning
                        drop(_guard);
                        return Err(ApiError::unauthorized("VTOP_SESSION_EXPIRED"));
                    }

                    // Throttle success by sleeping *while holding the lock* for 500ms
                    // This guarantees at least 500ms between requests from the same session
                    sleep(Duration::from_millis(500)).await;
                    
                    return Ok(html);
                } else {
                    last_error = format!("HTTP {}", res.status());
                }
            }
            Err(e) => {
                last_error = e.to_string();
            }
        }

        // Wait before retry
        if attempts < 2 {
            sleep(Duration::from_millis(500)).await;
        }
    }

    // Release lock happens automatically on function exit
    Err(ApiError::internal(format!("VTOP request failed after retries: {}", last_error)))
}
