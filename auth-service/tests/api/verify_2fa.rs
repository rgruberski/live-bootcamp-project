use macros::api_test;
use auth_service::domain::{Email, LoginAttemptId, TwoFACode};
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use crate::helpers::{get_random_email, TestApp};

#[api_test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let login_attempt_id = LoginAttemptId::default();

    let body = serde_json::json!({
        "email": get_random_email(),
        "loginAttemptId": login_attempt_id,
    });

    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[api_test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let login_attempt_id = LoginAttemptId::default();

    let body = serde_json::json!({
        "email": get_random_email(),
        "loginAttemptId": login_attempt_id,
        "2FACode": "1234",
    });

    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[api_test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code
    });

    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[api_test]
async fn should_return_401_if_old_code() {

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
        "password": "password123"
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Can't deserialize to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();

    assert_eq!(
        LoginAttemptId::parse(json_body.login_attempt_id).unwrap(),
        login_attempt_id
    );

    let two_fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code
    });

    let response = app.post_verify_2fa(&two_fa_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let cookie = response
         .cookies()
         .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
         .expect("Auth cookie not found");

     assert!(!cookie.value().is_empty());

     let response = app.post_verify_2fa(&two_fa_body).await;

     assert_eq!(response.status().as_u16(), 401);
}

#[api_test]
async fn should_return_200_if_correct_code() {

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
        "password": "password123"
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Can't deserialize to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();

    assert_eq!(
        LoginAttemptId::parse(json_body.login_attempt_id).unwrap(),
        login_attempt_id
    );

    let two_fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code
    });

    let response = app.post_verify_2fa(&two_fa_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("Auth cookie not found");

    assert!(!cookie.value().is_empty());
}