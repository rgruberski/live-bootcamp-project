use auth_service::utils::constants::JWT_COOKIE_NAME;
use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {

    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "email": get_random_email(),
            "loginAttemptId": "example string"
        }),
        serde_json::json!({
            "email": get_random_email(),
            "2FACode": "example string"
        }),
        serde_json::json!({
            "loginAttemptId": "example string",
            "2FACode": "example string"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_token(&test_case).await;

        assert_eq!(response.status().as_u16(), 422);
    }
}

#[tokio::test]
async fn should_return_200_valid_token() {

    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("Auth cookie not found");

    let verify_token_body = serde_json::json!({
        "token": cookie.value(),
    });

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {

    let app = TestApp::new().await;

    let verify_token_body = serde_json::json!({
        "token": "invalid token",
    });

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn should_return_401_if_banned_token() {

    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("Auth cookie not found");

    let verify_token_body = serde_json::json!({
        "token": cookie.value(),
    });

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status(), 200);

    app.post_logout().await;

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status(), 401);
}
