use std::io;
use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};

use super::routes::{health_check, subscribe};

pub fn run(tcp_listener: TcpListener) -> io::Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(tcp_listener)?
    .run();

    Ok(server)
}
