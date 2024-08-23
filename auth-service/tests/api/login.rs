use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {

    let app = TestApp::new().await;

    let login_data = serde_json::json!({
        "email": get_random_email(),
    });

    let response = app.post_login(&login_data).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.

    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "email": "invalid_email.com",
            "password": "password123"
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "invalid"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;

        assert_eq!(response.status().as_u16(), 400);
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.

    let app = TestApp::new().await;

    let login_data = serde_json::json!({
        "email": "invalid@example.com",
        "password": "password123"
    });

    let response = app.post_login(&login_data).await;

    assert_eq!(response.status().as_u16(), 401);
}
