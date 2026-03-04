use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use hello_actix_web::{
    configuration::{DatabaseSettings, get_configuration},
    startup::Server,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACIING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber(get_subscriber("test", "debug", std::io::stdout));
    } else {
        init_subscriber(get_subscriber("test", "debug", std::io::sink));
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("http://{}/subscriptions", self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACIING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.application.port = 0;
        c.database.database_name = Uuid::new_v4().to_string();
        c
    };

    let db_connection_pool = configure_database(&configuration.database).await;

    let Server {
        actix_server,
        listen_address,
    } = Server::build(configuration).expect("Failed to build actix server");

    // run server in the background to do not block the test cases
    tokio::spawn(actix_server);

    TestApp {
        address: listen_address,
        db_pool: db_connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}
