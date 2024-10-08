use auth_service::domain::{Email, LoginAttemptId};
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use crate::helpers::{get_random_email, TestApp};

use test_helpers::api_test;

#[api_test]
async fn should_return_422_if_malformed_credentials() {

    let app = TestApp::new().await;

    let login_data = serde_json::json!({
        "email": get_random_email(),
    });

    let response = app.post_login(&login_data).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[api_test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.

    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "email": "invalid_email.com",
            "password": "password123"
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "invalid"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;

        assert_eq!(response.status().as_u16(), 400);
    }
}

#[api_test]
async fn should_return_401_if_incorrect_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.

    let app = TestApp::new().await;

    let login_data = serde_json::json!({
        "email": "invalid@example.com",
        "password": "password123"
    });

    let response = app.post_login(&login_data).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[api_test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {

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

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[api_test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {

    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let json_data = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(
        json_data.message,
        "2FA required".to_owned()
    );

    let login_attempt_id = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();

    assert_eq!(
        LoginAttemptId::parse(json_data.login_attempt_id).unwrap(),
        login_attempt_id.0
    );
}
