use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use crate::AppState;
use crate::domain::{AuthAPIError, Email, Password, UserStoreError};
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

    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
