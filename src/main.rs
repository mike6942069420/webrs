//mod body;
mod crypt;
mod db;
mod handler;
mod logging;
mod template;
mod ws;

use std::net::SocketAddr;

// hyper stuff
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

// loging
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting server...");
    db::add_message(db::Message {
        content: "hello world".to_string(),
    })
    .await;
    let _guard = logging::init_logging();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Listening on http://{}", addr);

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handler::handle_request))
                .with_upgrades()
                .await
            {
                eprintln!("Error serving connection: {err:?}");
            }
        });
    }
}
