use std::io;
use std::net::TcpListener;
use hello::startup::run;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr()?.port();
    println!("Listening on: {port}");

    run(listener)?.await
}
