use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use crate::AppState;
use crate::domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode, UserStoreError};
use crate::utils::auth::generate_auth_cookie;

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {

    let email = match Email::parse(&request.email) {
        Ok(email) => email,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    let login_attempt_id = match LoginAttemptId::parse(request.login_attempt_id) {
        Ok(login_attempt_id) => login_attempt_id,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    let two_fa_code = match TwoFACode::parse(request.two_fa_code) {
        Ok(two_fa_code) => two_fa_code,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    let mut two_fa_code_store =
        state.two_fa_code_store.write().await;

    let user = match state.user_store.read().await.get_user(&email).await {
        Ok(user) => user,
        Err(UserStoreError::UserNotFound ) => return Err(AuthAPIError::IncorrectCredentials),
        Err(_) => return Err(AuthAPIError::UnexpectedError),
    };

    let code_tuple = two_fa_code_store.get_code(&email).await;

    match code_tuple {
        Ok(code_tuple) if code_tuple == (login_attempt_id, two_fa_code) => {

            two_fa_code_store.remove_code(&email).await.unwrap();

            let cookie = match generate_auth_cookie(&user.email) {
                Ok(cookie) => cookie,
                Err(_) => return Err(AuthAPIError::IncorrectCredentials),
            };

            Ok((jar.add(cookie), StatusCode::OK.into_response()))
        },
        _ => Err(AuthAPIError::IncorrectCredentials),
    }
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
