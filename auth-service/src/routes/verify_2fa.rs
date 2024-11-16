use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
    utils::auth::generate_auth_cookie,
};

#[tracing::instrument(name = "Verify 2FA", skip_all)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = Email::parse(request.email);
    if email.is_err() {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }
    let email = email.unwrap();

    let login_attempt_id = LoginAttemptId::parse(request.login_attempt_id);
    if login_attempt_id.is_err() {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }
    let login_attempt_id = login_attempt_id.unwrap();

    let two_fa_code = TwoFACode::parse(request.two_fa_code);
    if two_fa_code.is_err() {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }
    let two_fa_code = two_fa_code.unwrap();

    let mut two_fa_code_store = state.two_fa_code_store.write().await;

    let code_tuple = match two_fa_code_store.get_code(&email).await {
        Ok(tuple) => tuple,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    if code_tuple.0 != login_attempt_id || code_tuple.1 != two_fa_code {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    if let Err(e) = two_fa_code_store.remove_code(&email).await {
        return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
    }

    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))),
    };

    let updated_jar = jar.add(auth_cookie);

    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}
