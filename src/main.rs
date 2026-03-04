use std::io;

use hello_actix_web::{
    configuration::get_configuration,
    startup::Server,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    init_subscriber(get_subscriber("hello_actix_web", "info", std::io::stdout));

    let configuration = get_configuration().expect("Failed to read configuration.");
    Server::build(configuration)?.run().await
}
