use auth_service::{
    domain::{Email, LoginAttemptId, TwoFACode},
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
    ErrorResponse,
};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "loginAttemptId": LoginAttemptId::default(),
            "2FACode": "123456",
        }),
        serde_json::json!({
            "email": random_email,
            "2FACode": "123456",
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": LoginAttemptId::default(),
        }),
        serde_json::json!({
            "email": random_email,
        }),
        serde_json::json!({
            "loginAttemptId": LoginAttemptId::default(),
        }),
        serde_json::json!({
            "2FACode": "123456",
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let inputs = [
        serde_json::json!({
            "email":"example.com",
            "loginAttemptId": LoginAttemptId::default(),
            "2FACode": TwoFACode::default(),
        }),
        serde_json::json!({
            "email":random_email,
            "loginAttemptId": "invalid",
            "2FACode": TwoFACode::default(),
        }),
        serde_json::json!({
            "email":random_email,
            "loginAttemptId": LoginAttemptId::default(),
            "2FACode": "invalid",
        }),
    ];

    for input in inputs.iter() {
        let response = app.post_verify_2fa(input).await;

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

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let verify_2fa_body = serde_json::json!({
        "email":random_email,
        "loginAttemptId": LoginAttemptId::default().as_ref(),
        "2FACode": TwoFACode::default().as_ref(),
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

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

#[tokio::test]
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

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let verify_2fa_body = serde_json::json!({
        "email":random_email,
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": TwoFACode::default().as_ref(),
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

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

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let app = TestApp::new().await;

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

    let code_tuple = {
        let two_fa_code_store = app.two_fa_code_store.read().await;
        let email = Email::parse(random_email.clone()).unwrap();
        two_fa_code_store.get_code(&email).await.unwrap()
    };

    let verify_2fa_body = serde_json::json!({
        "email":random_email,
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": code_tuple.1.as_ref(),
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let app = TestApp::new().await;

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

    let code_tuple = {
        let two_fa_code_store = app.two_fa_code_store.read().await;
        let email = Email::parse(random_email.clone()).unwrap();
        two_fa_code_store.get_code(&email).await.unwrap()
    };

    let verify_2fa_body = serde_json::json!({
        "email":random_email,
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": code_tuple.1.as_ref(),
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let response = app.post_verify_2fa(&verify_2fa_body).await;

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
