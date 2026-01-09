use std::io;
use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

//use crate::email_client::EmailClient;

use super::routes::{health_check, subscribe};

pub fn run(
    tcp_listener: TcpListener,
    db_connection_pool: PgPool,
    //email_client: EmailClient,
) -> io::Result<Server> {
    let db_connection_pool = Data::new(db_connection_pool);
    //let email_client = Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_connection_pool.clone())
            //.app_data(email_client.clone())
    })
    .listen(tcp_listener)?
    .run();

    Ok(server)
}
