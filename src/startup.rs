use std::io;
use std::net::TcpListener;

use actix_web::{App, HttpServer, dev::Server as ActixServer, web, web::Data};
use secrecy::ExposeSecret;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::Settings,
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

pub struct Server {
    pub actix_server: ActixServer,
    pub listen_address: String,
}

impl Server {
    pub fn build(configuration: Settings) -> io::Result<Self> {
        // application
        let mut address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        if configuration.application.port == 0 {
            address = format!(
                "{}:{}",
                configuration.application.host,
                listener.local_addr().unwrap().port()
            );
        }

        // database
        let db_connection_pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy(configuration.database.connection_string().expose_secret())
            .expect("Failed to connect Postgres.");

        // email
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        println!("Listening on: {address}");

        Ok(Self {
            actix_server: build_actix_server(listener, db_connection_pool, email_client)?,
            listen_address: address,
        })
    }

    pub async fn run(self) -> io::Result<()> {
        self.actix_server.await
    }
}

fn build_actix_server(
    tcp_listener: TcpListener,
    db_connection_pool: PgPool,
    email_client: EmailClient,
) -> io::Result<ActixServer> {
    let db_connection_pool = Data::new(db_connection_pool);
    let email_client = Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_connection_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(tcp_listener)?
    .run();

    Ok(server)
}
