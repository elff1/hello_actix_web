use std::io;
use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;

use hello_actix_web::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    init_subscriber(get_subscriber("hello_actix_web", "info", std::io::stdout));

    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(&address)?;

    let db_connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Failed to connect Postgres.");

    println!("Listening on: {address}");

    run(listener, db_connection_pool)?.await
}
