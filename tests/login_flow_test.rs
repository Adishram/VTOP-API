use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mockito::Server;
use serde_json::{json, Value};
use std::fs;
use tower::ServiceExt;

use vtop_api::{config::Config, router::build_router, state::AppState};

#[tokio::test]
async fn test_full_login_flow() {
    // 1. Setup Mockito server
    let mut server = Server::new_async().await;

    // Load fixtures
    let login_page_html = fs::read_to_string("fixtures/login_page.html").expect("Failed to read fixture");
    let captcha_form_html = fs::read_to_string("fixtures/captcha_login_form.html").expect("Failed to read fixture");
    let dashboard_html = fs::read_to_string("fixtures/dashboard.html").expect("Failed to read fixture");

    // Mock GET /login
    let mock_get_login = server.mock("GET", "/login")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(login_page_html)
        .create_async().await;

    // Mock POST /prelogin/setup
    let mock_post_setup = server.mock("POST", "/prelogin/setup")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(captcha_form_html)
        .create_async().await;

    // Mock POST /login (the actual login verification)
    let mock_post_login = server.mock("POST", "/login")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(dashboard_html)
        .create_async().await;

    // 2. Setup API Router
    let config = Config {
        host: "127.0.0.1".into(),
        port: 3000,
        vtop_base_url: server.url(),
        session_ttl_secs: 1800,
    };
    let app_state = AppState::new(config);
    let app = build_router(app_state.clone());

    // 3. Step 1: POST /api/login
    let login_req_body = json!({
        "username": "21BCE1234"
    });
    
    let request1 = Request::builder()
        .method("POST")
        .uri("/api/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_req_body).unwrap()))
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    let body_bytes = response1.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(body_json["status"], "ok");
    
    let login_session_id = body_json["data"]["login_session_id"].as_str().unwrap().to_string();
    let captcha_image = body_json["data"]["captcha_image"].as_str().unwrap().to_string();
    
    assert!(!login_session_id.is_empty());
    assert!(captcha_image.starts_with("data:image/png;base64"));
    
    // Check that mockito endpoints were hit
    mock_get_login.assert_async().await;
    mock_post_setup.assert_async().await;

    // Verify state map
    {
        let map = app_state.login_sessions.read().await;
        assert!(map.contains_key(&login_session_id));
    }

    // 4. Step 2: POST /api/login/verify
    let verify_req_body = json!({
        "login_session_id": login_session_id,
        "username": "21BCE1234",
        "password": "mypassword",
        "captcha": "aBcD"
    });

    let request2 = Request::builder()
        .method("POST")
        .uri("/api/login/verify")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&verify_req_body).unwrap()))
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::OK);

    let body_bytes = response2.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(body_json["status"], "ok");
    
    let token = body_json["data"]["token"].as_str().unwrap().to_string();
    assert!(!token.is_empty());

    // Check that the login verification mock was hit
    mock_post_login.assert_async().await;

    // Verify that the login session was removed
    {
        let map = app_state.login_sessions.read().await;
        assert!(!map.contains_key(&login_session_id));
    }

    // Verify that the auth session was added
    {
        let map = app_state.auth_sessions.read().await;
        assert_eq!(map.len(), 1);
        let session = map.values().next().unwrap();
        assert_eq!(session.authorized_id, "REDACTED_REG_NO"); // Based on fixture
    }
}
