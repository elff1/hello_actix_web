use std::net::TcpListener;

use reqwest::Client;
use sqlx::{Connection, PgConnection};

use hello::{configuration::get_configuration, startup::run};

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = Client::new();

    let response = client
        .get(format!("{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration.");

    let db_connection_string = configuration.database.connection_string();
    let db_connection = PgConnection::connect(&db_connection_string)
        .await
        .expect("Failed to connect Postgres.");

    let client = Client::new();

    let form_data = "name=le%20guin&email=le_guin%40gmail.com";
    let response = client
        .post(format!("{app_address}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_data)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_form_data() {
    let address = spawn_app();
    let client = Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing email"),
        ("email=le_guin%40gmail.com", "missing name"),
        ("", "missiing name and email"),
    ];

    for (invalid_data, error_msg) in test_cases {
        let response = client
            .post(format!("{address}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_data)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "failed of case {}",
            error_msg
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");

    tokio::spawn(server);

    format!("http://127.0.0.1:{port}")
}
