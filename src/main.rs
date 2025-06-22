mod handler;
mod logging;
mod template;

use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    println!("Starting web server...");

    let _log_guard = logging::init_logging();
    info!("Starting web server...");

    let addr = ([127, 0, 0, 1], 8080).into();
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handler::handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);
    info!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    } else {
        info!("Server stopped gracefully.");
    }
}
