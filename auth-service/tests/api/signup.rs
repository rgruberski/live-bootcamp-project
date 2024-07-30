use crate::helpers::TestApp;

#[tokio::test]
async fn signup_returns_200() {
    let app = TestApp::new().await;

    let dummy_body = serde_json::json!({
        "email": "user@example.com",
        "password": "password",
    });

    let response = app.post_signup(&dummy_body).await;

    assert_eq!(response.status().as_u16(), 200);
}
