use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, header, header_exists, method, path},
};

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
async fn subscribe_persists_the_new_subscriber() {
    let test_app = spawn_app().await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.mock_email_server)
        .await;

    let form_data = "name=le%20guin&email=le_guin%40gmail.com";
    let _response = test_app.post_subscriptions(form_data.into()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "le_guin@gmail.com");
    assert_eq!(saved.status, "pending_confirmation");
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
    let email_json: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let html_link = get_url_links(email_json["HtmlBody"].as_str().unwrap())[0];
    let text_link = get_url_links(email_json["TextBody"].as_str().unwrap())[0];

    assert_eq!(html_link, text_link);
}
