use auth_service::{routes::SignupResponse, ErrorResponse};
use test_helpers::api_test;

use crate::helpers::{get_random_email, TestApp};

#[api_test]
async fn should_return_422_if_malformed_input() {
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email":random_email,
            "password": "password123",
        }),
        serde_json::json!({
            "email": random_email,
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
        }),
        serde_json::json!({
            "password": "password123",
        }),
        serde_json::json!({
            "requires2FA": false
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
async fn should_return_201_if_valid_input() {
    let test_case = serde_json::json!({
        "email":"test@example.com",
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&test_case).await;

    assert_eq!(response.status().as_u16(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_owned(),
    };

    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response
    );
}

#[api_test]
async fn should_return_400_if_invalid_input() {
    let inputs = [
        serde_json::json!({
            "email":"",
            "password": "password123",
            "requires2FA": false
        }),
        serde_json::json!({
            "email":"test",
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "test@example.com",
            "password": "pass123",
            "requires2FA": true
        }),
    ];

    for input in inputs.iter() {
        let response = app.post_signup(input).await;

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
async fn should_return_409_if_email_already_exists() {
    let test_case = serde_json::json!({
        "email":"test@example.com",
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&test_case).await;

    assert_eq!(response.status().as_u16(), 201);

    let response = app.post_signup(&test_case).await;

    assert_eq!(response.status().as_u16(), 409);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}
