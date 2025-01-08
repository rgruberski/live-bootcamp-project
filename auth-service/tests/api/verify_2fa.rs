use secrecy::{ExposeSecret, Secret};
use test_helpers::api_test;
use auth_service::domain::{Email, LoginAttemptId, TwoFACode};
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use crate::helpers::{get_random_email, TestApp};

#[api_test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let login_attempt_id = LoginAttemptId::default().as_ref().to_owned();

    let body = serde_json::json!({
        "email": get_random_email(),
        "loginAttemptId": login_attempt_id.expose_secret(),
    });

    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[api_test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let login_attempt_id = LoginAttemptId::default().as_ref().to_owned();

    let body = serde_json::json!({
        "email": get_random_email(),
        "loginAttemptId": login_attempt_id.expose_secret(),
        "2FACode": "1234",
    });

    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[api_test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(response_body.message, "2FA required".to_owned());
    assert!(!response_body.login_attempt_id.is_empty());

    let incorrect_email = get_random_email();
    let incorrect_login_attempt_id = LoginAttemptId::default().as_ref().to_owned();
    let incorrect_two_fa_code = TwoFACode::default().as_ref().to_owned();
    
    let body = serde_json::json!({
        "email": incorrect_email,
        "loginAttemptId": incorrect_login_attempt_id.expose_secret(),
        "2FACode": incorrect_two_fa_code.expose_secret(),
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

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

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
    assert!(!json_body.login_attempt_id.is_empty());
    
    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(Secret::new(random_email.to_owned())).unwrap())
        .await
        .unwrap();

    assert_eq!(
        LoginAttemptId::parse(Secret::new(json_body.login_attempt_id)).unwrap(),
        login_attempt_id
    );

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let two_fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id.as_ref().expose_secret(),
        "2FACode": two_fa_code.as_ref().expose_secret(),
    });

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
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(response_body.message, "2FA required".to_owned());
    assert!(!response_body.login_attempt_id.is_empty());

    let login_attempt_id = response_body.login_attempt_id;

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(Secret::new(random_email.clone())).unwrap())
        .await
        .unwrap();

    let code = code_tuple.1.as_ref().expose_secret();

    let request_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": code
    });

    let response = app.post_verify_2fa(&request_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}


#[api_test]
async fn should_return_401_if_same_code_twice() {
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(response_body.message, "2FA required".to_owned());
    assert!(!response_body.login_attempt_id.is_empty());

    let login_attempt_id = response_body.login_attempt_id;

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(Secret::new(random_email.clone())).unwrap())
        .await
        .unwrap();

    let code = code_tuple.1.as_ref();

    let request_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": code.expose_secret()
    });

    let response = app.post_verify_2fa(&request_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let response = app.post_verify_2fa(&request_body).await;

    assert_eq!(response.status().as_u16(), 401);
}