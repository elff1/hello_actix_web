use std::io;
use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;

use hello_actix_web::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    init_subscriber(get_subscriber("hello_actix_web", "info", std::io::stdout));

    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address)?;

    let db_connection_pool =
        PgPool::connect(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect Postgres.");

    println!("Listening on: {address}");

    run(listener, db_connection_pool)?.await
}
