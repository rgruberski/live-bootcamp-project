use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use serde::Deserialize;
use crate::AppState;
use crate::domain::{AuthAPIError};
use crate::utils::auth;

pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>
) -> Result<impl IntoResponse, AuthAPIError> {

    match auth::validate_token(&request.token, state.banned_token_store.clone()).await {
        Ok(_) => (),
        Err(_) => return Err(AuthAPIError::InvalidToken),
    }

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String
}
