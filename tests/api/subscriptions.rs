use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, header, header_exists, method, path},
};

use hello_actix_web::constants::SUBSCRIPTION_TOKEN_LENGTH;

use crate::helper::*;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.mock_email_server)
        .await;

    let form_data = "name=le%20guin&email=le_guin%40gmail.com";
    let response = test_app.post_subscriptions(form_data.into()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_for_missing_field() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing email"),
        ("email=le_guin%40gmail.com", "missing name"),
        ("", "missiing name and email"),
    ];

    for (invalid_data, error_msg) in test_cases {
        let response = test_app.post_subscriptions(invalid_data.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "failed of case {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_for_empty_filed() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin&email=", "empty email"),
        ("name=&email=le_guin%40gmail.com", "empty name"),
        ("name=&email=", "empty name and email"),
    ];

    for (invalid_data, error_msg) in test_cases {
        let response = test_app.post_subscriptions(invalid_data.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "failed of case {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_email() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin&email=invalid_email", "invalid email"),
        ("name=le%20guin&email=%40invalid_email", "invalid email"),
    ];

    for (invalid_data, error_msg) in test_cases {
        let response = test_app.post_subscriptions(invalid_data.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "failed of case {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber_and_token() {
    let test_app = spawn_app().await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.mock_email_server)
        .await;

    let form_data = "name=le%20guin&email=le_guin%40gmail.com";
    let _response = test_app.post_subscriptions(form_data.into()).await;

    let persisted_subscriber = sqlx::query!("SELECT id, email, name, status FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    let persisted_token =
        sqlx::query!("SELECT subscription_token, subscriber_id FROM subscription_tokens")
            .fetch_one(&test_app.db_pool)
            .await
            .expect("Failed to fetch saved subscription tokens.");

    assert_eq!(persisted_subscriber.name, "le guin");
    assert_eq!(persisted_subscriber.email, "le_guin@gmail.com");
    assert_eq!(persisted_subscriber.status, "pending_confirmation");
    assert_eq!(persisted_subscriber.id, persisted_token.subscriber_id);
    assert_eq!(
        persisted_token.subscription_token.len(),
        SUBSCRIPTION_TOKEN_LENGTH
    );
    assert!(
        persisted_token
            .subscription_token
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
    )
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let test_app = spawn_app().await;

    Mock::given(header_exists("X-Postmark-Server-Token"))
        .and(header("Content-Type", "application/json"))
        .and(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.mock_email_server)
        .await;

    let form_data = "name=le%20guin&email=le_guin%40gmail.com";
    let _response = test_app.post_subscriptions(form_data.into()).await;

    let email_request = test_app
        .mock_email_server
        .received_requests()
        .await
        .unwrap()
        .swap_remove(0);
    let confirmation_links = TestApp::get_confirmation_links(&email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    let test_app = spawn_app().await;
    let form_data = "name=le%20guin&email=le_guin%40gmail.com";

    //let _ = sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token")
    let _ = sqlx::query!("ALTER TABLE subscriptions DROP COLUMN email")
        .execute(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    let response = test_app.post_subscriptions(form_data.into()).await;

    assert_eq!(response.status().as_u16(), 500);
}
