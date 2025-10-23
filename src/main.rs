use std::io;
use std::net::TcpListener;

use hello::configuration::get_configuration;
use hello::startup::run;

#[tokio::main]
async fn main() -> io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address)?;

    println!("Listening on: {address}");

    run(listener)?.await
}
