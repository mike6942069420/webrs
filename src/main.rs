mod constants;
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
    let _guard = logging::init_logging();

    #[cfg(not(debug_assertions))]
    if let Err(e) = db::init_messages().await {
        eprintln!("Failed to initialize DB: {}", e);
        return Err("Failed to initialize DB".into());
    }

    let addr = SocketAddr::from(constants::MAIN_HOST);
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on http://{}", addr);
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .keep_alive(true)
                .serve_connection(io, service_fn(handler::handle_request))
                .with_upgrades()
                .await
            {
                eprintln!("Error serving connection: {err:?}");
            }
        });
    }
}
