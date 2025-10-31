use std::io;
use std::net::TcpListener;

use hello::configuration::get_configuration;
use hello::startup::run;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address)?;

    let db_connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect Postgres.");

    println!("Listening on: {address}");

    run(listener, db_connection_pool)?.await
}
