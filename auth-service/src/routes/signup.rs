use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password, User},
};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email);
    if email.is_err() {
        return Err(AuthAPIError::InvalidCredentials);
    }
    let email = email.unwrap();

    let password = Password::parse(request.password);
    if password.is_err() {
        return Err(AuthAPIError::InvalidCredentials);
    }
    let password = password.unwrap();

    let user = User::new(email.clone(), password, request.requires_2fa);

    let mut user_store = state.user_store.write().await;

    if let Ok(_) = user_store.get_user(&email).await {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let result = user_store.add_user(user).await;
    if result.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SignupResponse {
    pub message: String,
}
