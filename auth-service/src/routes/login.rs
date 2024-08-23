use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use serde::Deserialize;
use crate::AppState;
use crate::domain::{AuthAPIError, Email, Password, UserStoreError};

pub async fn login(State(state): State<AppState>, Json(request): Json<LoginRequest>)
    -> Result<impl IntoResponse, AuthAPIError> {

    let email = match Email::parse(&request.email) {
        Ok(email) => email,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    let password = match Password::parse(&request.password) {
        Ok(password) => password,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };

    let user_store = state.user_store.read().await;

    match user_store.validate_user(&email, &password).await {
        Ok(_) => (),
        Err(UserStoreError::InvalidCredentials) => return Err(AuthAPIError::IncorrectCredentials),
        Err(UserStoreError::UserNotFound) => return Err(AuthAPIError::IncorrectCredentials),
        Err(_) => return Err(AuthAPIError::UnexpectedError),
    }

    let user = match user_store.get_user(&email).await {
        Ok(_) => (),
        Err(UserStoreError::UserNotFound) => return Err(AuthAPIError::IncorrectCredentials),
        Err(_) => return Err(AuthAPIError::UnexpectedError)
    };

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
