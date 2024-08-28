use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::domain::{AuthAPIError, Email, LoginAttemptId, Password, TwoFACode, UserStoreError};
use crate::utils::auth;

pub async fn login(State(state): State<AppState>, jar: CookieJar, Json(request): Json<LoginRequest>)
                   -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {

    let email = match Email::parse(&request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let password = match Password::parse(&request.password) {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let user_store = state.user_store.read().await;

    match user_store.validate_user(&email, &password).await {
        Ok(_) => (),
        Err(UserStoreError::InvalidCredentials) =>
            return (jar, Err(AuthAPIError::IncorrectCredentials)),
        Err(UserStoreError::UserNotFound) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    }

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(UserStoreError::UserNotFound) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError))
    };

    let auth_cookie = match auth::generate_auth_cookie(&user.email) {
        Ok(auth_cookie) => auth_cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError))
    };

    let updated_jar = jar.add(auth_cookie);

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, updated_jar).await,
        false => handle_no_2fa(updated_jar).await,
    }

    /*(updated_jar, Ok(StatusCode::OK.into_response()))*/
}

async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    // TODO: Return a TwoFactorAuthResponse. The message should be "2FA required".
    // The login attempt ID should be "123456". We will replace this hard-coded login attempt ID soon!

    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    match state.two_fa_code_store.write().await.add_code(
        email.clone(),
        login_attempt_id.clone(),
        two_fa_code.clone(),
    ).await {
        Ok(_) => (),
        _ => return (jar, Err(AuthAPIError::UnexpectedError)),
    }

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.to_string(),
    }));

    (jar, Ok((StatusCode::PARTIAL_CONTENT, response)))
}

async fn handle_no_2fa(
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {

    let response = (StatusCode::OK, Json(LoginResponse::RegularAuth));

    (jar, Ok(response))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
