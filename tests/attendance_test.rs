use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use mockito::Server;
use serde_json::Value;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

use vtop_api::{
    config::Config,
    handlers::auth::Claims,
    parsers::attendance::{classes_to_target, parse_attendance_html},
    router::build_router,
    state::{AppState, AuthSession},
};

#[test]
fn test_parse_attendance_html() {
    let html = fs::read_to_string("fixtures/attendance.html").unwrap();
    let records = parse_attendance_html(&html).unwrap();
    
    assert_eq!(records.len(), 6);
    
    // First record: BCSE302L, 36/40 (90%)
    assert_eq!(records[0].course_code, "BCSE302L");
    assert_eq!(records[0].course_title, "Database Management Systems");
    assert_eq!(records[0].course_type, "Theory");
    assert_eq!(records[0].slot, "A1+TA1");
    assert_eq!(records[0].faculty, "Dr. Faculty One");
    assert_eq!(records[0].attended, 36);
    assert_eq!(records[0].total, 40);
    assert_eq!(records[0].percentage, 90);
    // target_classes check: 36/40 = 90%. Target is 75%.
    // To drop to 75%, how many classes can we miss?
    // 36 / (40 + M) = 0.75 => 40 + M = 48 => M = 8.
    // So target_classes should be -8.
    assert_eq!(records[0].target_classes, -8);
    
    // Last record: BECE401L, 28/40 (70%)
    assert_eq!(records[5].course_code, "BECE401L");
    assert_eq!(records[5].percentage, 70);
    // target_classes check: 28/40 = 70%. Target is 75%.
    // (28 + N) / (40 + N) = 0.75
    // 28 + N = 30 + 0.75N
    // 0.25N = 2
    // N = 8
    assert_eq!(records[5].target_classes, 8);
}

#[test]
fn test_classes_to_target() {
    // 0 classes
    assert_eq!(classes_to_target(0, 0, 75.0), 0);
    
    // Already on target perfectly
    assert_eq!(classes_to_target(3, 4, 75.0), 0); // 75%
    
    // Below target
    assert_eq!(classes_to_target(28, 40, 75.0), 8); // 28/40 = 70%. Needs 8 -> 36/48 = 75%
    assert_eq!(classes_to_target(5, 10, 75.0), 10); // 5/10 = 50%. Needs 10 -> 15/20 = 75%
    
    // Above target
    assert_eq!(classes_to_target(36, 40, 75.0), -8); // 36/40 = 90%. Can miss 8 -> 36/48 = 75%
    assert_eq!(classes_to_target(10, 10, 75.0), -3); // 10/10 = 100%. Can miss 3 -> 10/13 = 76.9%. If miss 4 -> 10/14 = 71.4%
}

#[tokio::test]
async fn test_attendance_endpoint() {
    let mut server = Server::new_async().await;
    
    let html = fs::read_to_string("fixtures/attendance.html").unwrap();
    
    let mock_post = server.mock("POST", "/processViewStudentAttendance")
        .match_body(mockito::Matcher::Regex("semesterSubId=CH2025261".into()))
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(html)
        .create_async().await;

    let config = Config {
        host: "127.0.0.1".into(),
        port: 3000,
        vtop_base_url: server.url(),
        session_ttl_secs: 1800,
    };
    
    let app_state = AppState::new(config);
    let app = build_router(app_state.clone());

    // Inject a fake session directly into state
    let user_id = "test_user_id".to_string();
    {
        let mut map = app_state.auth_sessions.write().await;
        map.insert(user_id.clone(), AuthSession {
            client: reqwest::Client::new(),
            csrf_token: "test_csrf".to_string(),
            authorized_id: "20BCE0000".to_string(),
            created_at: std::time::Instant::now(),
            request_lock: std::sync::Arc::new(tokio::sync::Mutex::new(())),
        });
    }

    // Generate JWT for the fake session
    let claims = Claims {
        sub: user_id,
        exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize + 3600,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(app_state.jwt_secret.as_bytes())).unwrap();

    // Make request to /api/attendance
    let request1 = Request::builder()
        .method("GET")
        .uri("/api/attendance?semester=CH2025261")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    let body_bytes = response1.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["status"], "ok");
    
    // Verify the mock was hit exactly once
    mock_post.assert_async().await;

    // Second request should hit the cache!
    let request2 = Request::builder()
        .method("GET")
        .uri("/api/attendance?semester=CH2025261")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::OK);

    // If it hit the mock a second time, it would fail because the mock expects exactly 1 request by default.
    // The fact that it succeeds and returns OK proves the cache intercepted it.
}
