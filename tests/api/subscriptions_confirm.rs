use wiremock::{
    Mock, ResponseTemplate,
    matchers::{header, header_exists, method, path},
};

use crate::helper::*;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let test_app = spawn_app().await;

    let response = reqwest::get(&format!("{}/subscriptions/confirm", test_app.address))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn confirmations_without_invalid_token_are_rejected_with_a_401() {
    let test_app = spawn_app().await;

    let response = reqwest::get(&format!(
        "{}/subscriptions/confirm?subscription_token=a_common_string",
        test_app.address
    ))
    .await
    .unwrap();

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
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

    let response = reqwest::get(confirmation_links.html).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);

    let persisted_subscriber = sqlx::query!("SELECT id, email, name, status FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");
    assert_eq!(persisted_subscriber.name, "le guin");
    assert_eq!(persisted_subscriber.email, "le_guin@gmail.com");
    assert_eq!(persisted_subscriber.status, "confirmed");
}

#[tokio::test]
async fn confirmation_fails_if_there_is_a_fatal_database_error() {
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

    let _ = sqlx::query!("ALTER TABLE subscriptions DROP COLUMN status")
        .execute(&test_app.db_pool)
        .await
        .expect("Failed to alter table.");

    let email_request = test_app
        .mock_email_server
        .received_requests()
        .await
        .unwrap()
        .swap_remove(0);
    let confirmation_links = TestApp::get_confirmation_links(&email_request);

    let response = reqwest::get(confirmation_links.html).await.unwrap();
    assert_eq!(response.status().as_u16(), 500);
}
