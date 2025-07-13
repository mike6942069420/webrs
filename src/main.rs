mod constants;
mod crypt;
mod db;
mod handler;
mod log;
mod ws;

use std::net::SocketAddr;

// hyper stuff
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};

// loging
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting server...");
    let _guard = log::init_logging();
    db::initialize().await;

    let addr = SocketAddr::from(constants::MAIN_HOST);
    let listener = TcpListener::bind(addr).await?;

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    info!(
        "======================================================== Listening on http://{} ========================================================",
        addr
    );
    println!("Listening on http://{addr}");

    loop {
        tokio::select! {
            conn = listener.accept() => {
                let (stream, _) = conn?;
                let io = TokioIo::new(stream);

                tokio::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .keep_alive(true)
                        .serve_connection(io, service_fn(handler::handle_request))
                        .with_upgrades()
                        .await
                    {
                        error!("XXX Error serving connection: {:?}", err);
                        eprintln!("Error serving connection: {err:?}");
                    }
                });
            },
            _ = sigint.recv() => {
                info!("XXX SIGINT");
                println!("Shutdown signal received: SIGINT");
                break;
            },
            _ = sigterm.recv() => {
                info!("XXX SIGTERM");
                println!("Shutdown signal received: SIGTERM");
                break;
            }
        }
    }

    Ok(())
}
