use auth_service::{
    domain::{data_stores::LoginAttemptId, Email},
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
    ErrorResponse,
};
use secrecy::Secret;
use test_helpers::api_test;

use crate::helpers::{get_random_email, TestApp};

#[api_test]
async fn should_return_422_if_malformed_credentials() {
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
        }),
        serde_json::json!({
            "email":random_email,
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[api_test]
async fn should_return_400_if_invalid_input() {
    let inputs = [
        serde_json::json!({
            "email":"",
            "password": "password123",
        }),
        serde_json::json!({
            "email":"test",
            "password": "password123",
        }),
        serde_json::json!({
            "email": "test@example.com",
            "password": "pass123",
        }),
    ];

    for input in inputs.iter() {
        let response = app.post_login(input).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            input
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
}

#[api_test]
async fn should_return_401_if_incorrect_credentials() {
    let test_case = serde_json::json!({
        "email":"test@example.com",
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&test_case).await;

    assert_eq!(response.status().as_u16(), 201);

    let test_case = serde_json::json!({
        "email":"test@example.com",
        "password": "password456"
    });

    let response = app.post_login(&test_case).await;

    assert_eq!(response.status().as_u16(), 401);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Incorrect credentials".to_owned()
    );
}

#[api_test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[api_test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
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
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    let two_fa_code_store = app.two_fa_code_store.read().await;
    let email = Email::parse(Secret::new(random_email)).unwrap();
    let stored_code = two_fa_code_store.get_code(&email).await;
    assert!(stored_code.is_ok());
    let (stored_login_attempt_id, _) = stored_code.unwrap();
    let expected_login_attempt_id =
        LoginAttemptId::parse(Secret::new(json_body.login_attempt_id)).unwrap();
    assert_eq!(stored_login_attempt_id, expected_login_attempt_id);
}
