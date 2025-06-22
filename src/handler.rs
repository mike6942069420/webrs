use crate::template;

use hyper::{Body, Request, Response};
use std::convert::Infallible;
use tracing::{error, info};

pub async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let headers = req.headers();

    // Extract Cloudflare IP
    let cf_ip = headers
        .get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("<no-ua>");
    let url = req.uri().to_string();
    let method = req.method().to_string();

    info!("[{}] \"{} {}\" UA:\"{}\"", cf_ip, method, url, ua);

    // Security check in release mode
    #[cfg(not(debug_assertions))]
    if cf_ip == "" {
        let x_real_ip = headers
            .get("X-Real-IP")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("<no-ip>");
        error!("[ {} ] - Not from cloudflare: {}", cf_ip, x_real_ip);
        return Ok(Response::builder()
            .status(403)
            .body(Body::from("Forbidden"))
            .unwrap());
    }

    let mut response_builder = Response::builder();
    response_builder = response_builder
        .header("X-Permitted-Cross-Domain-Policies", "none")
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "DENY")
        .header("Referrer-Policy", "no-referrer")
        .header(
            "Permissions-Policy",
            "geolocation=(), microphone=(), camera=()",
        )
        .header("Cross-Origin-Resource-Policy", "same-origin")
        .header("Cross-Origin-Opener-Policy", "same-origin")
        .header("Cross-Origin-Embedder-Policy", "require-corp");

    match req.method() {
        &hyper::Method::GET => {
            match req.uri().path() {
                "/" => {
                    match template::render(42) {
                        Ok(body) => Ok(response_builder
                            .header("Content-Security-Policy", "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; connect-src 'self'")
                            .header("Cache-Control", "no-cache, no-store, must-revalidate")
                            .header("Content-Type", "text/html; charset=utf-8")
                            .body(Body::from(body))
                            .unwrap()),
                        Err(e) => {
                            error!("[{}] Error rendering template: {}", cf_ip, e);
                            Ok(response_builder
                                .status(500)
                                .body(Body::from("Internal Server Error"))
                                .unwrap())
                        }
                    }
                },
                "/styles.css" => Ok(response_builder
                    .header("Content-Security-Policy", "default-src 'none'; img-src 'self'")
                    .header("Cache-Control", "no-cache, no-store, must-revalidate")
                    .header("Content-Type", "text/css; charset=utf-8")
                    .body(Body::from(include_str!("../templates/styles.css")))
                    .unwrap()),

                "/images/9d878e595dc522b07a801eae0fc6974d.webp" => Ok(response_builder
                    .header("Content-Security-Policy", "default-src 'none'; img-src 'self'")
                    .header("Cache-Control", "public, max-age=31536000, immutable")
                    .header("Content-Type", "templates/image/webp")
                    .body(Body::from(include_bytes!("../templates/images/bg.webp") as &'static [u8]))
                    .unwrap()),

                "/robots.txt" => Ok(response_builder
                    .header("Content-Security-Policy", "default-src 'none'; img-src 'self'")
                    .header("Cache-Control", "public, max-age=86400")
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .body(Body::from(include_str!("../templates/robots.txt")))
                    .unwrap()),

                "/sitemap.xml" => Ok(response_builder
                    .header("Content-Security-Policy", "default-src 'none'; img-src 'self'")
                    .header("Cache-Control", "public, max-age=86400")
                    .header("Content-Type", "application/xml; charset=utf-8")
                    .body(Body::from(include_str!("../templates/sitemap.xml")))
                    .unwrap()),

                "/favicon.ico" => Ok(response_builder
                    .header("Content-Security-Policy", "default-src 'none'; img-src 'self'")
                    .header("Cache-Control", "public, max-age=86400")
                    .header("Content-Type", "image/x-icon")
                    .body(Body::from(include_bytes!("../templates/favicon.ico") as &'static [u8]))
                    .unwrap()),

                _ => {
                    error!("[{}] Not Found: {}", cf_ip, req.uri().path());
                    Ok(response_builder
                        .status(404)
                        .body(Body::from("Not Found"))
                        .unwrap())
                }
            }
        },
        _ => {
            error!("[{}] Method Not Allowed: {}", cf_ip, method);
            Ok(response_builder
                .status(405)
                .body(Body::from("Method Not Allowed"))
                .unwrap())
        }
    }
}
