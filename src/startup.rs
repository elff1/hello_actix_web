use std::io;
use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use super::routes::{health_check, subscribe};

pub fn run(tcp_listener: TcpListener, db_connection_pool: PgPool) -> io::Result<Server> {
    let db_connection_pool = web::Data::new(db_connection_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_connection_pool.clone())
    })
    .listen(tcp_listener)?
    .run();

    Ok(server)
}
