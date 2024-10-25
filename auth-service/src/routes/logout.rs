use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{domain::AuthAPIError, utils::auth::validate_token};

pub async fn logout(jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = match jar.get("jwt") {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = cookie.value().to_owned();

    match validate_token(&token).await {
        Ok(_) => (),
        Err(_) => return (jar, Err(AuthAPIError::InvalidToken)),
    }

    let jar = jar.remove("jwt");

    (jar, Ok(StatusCode::OK))
}
