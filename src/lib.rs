use std::io;
use std::net::TcpListener;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web::dev::Server;

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn health_check(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    //req.
    HttpResponse::Ok().finish()
}

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
