use crate::crypt;
use crate::db;
use crate::template;
use crate::ws;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use std::net::IpAddr;

use tracing::{error, info};

const F_FAVICON: &[u8] = include_bytes!("../templates/favicon.ico");
const F_SITEMAP: &[u8] = include_bytes!("../templates/sitemap.xml");
const F_ROBOTS: &[u8] = include_bytes!("../templates/robots.txt");
const F_STYLES: &[u8] = include_bytes!("../templates/styles.css");
const F_BG: &[u8] = include_bytes!("../templates/images/bg.webp");

macro_rules! empty {
    () => {
        Full::new(Bytes::new())
    };
}

macro_rules! full {
    ($chunk:expr) => {
        Full::new(Bytes::from($chunk))
    };
}

macro_rules! err {
    ($status:expr, $log:expr) => {{
        error!("{}", $log);
        let mut res = Response::new(empty!());
        *res.status_mut() = $status;
        Ok(res)
    }};
}

macro_rules! dump_headers {
    ($headers:expr) => {{
        let mut s = String::new();
        s.push_str("Headers: || ");
        for (_, (k, v)) in $headers.iter().enumerate() {
            s.push_str(k.as_str());
            s.push_str(": ");
            s.push_str(v.to_str().unwrap_or(""));
            s.push_str(" || ");
        }
        s
    }};
}

pub async fn handle_request(
    mut req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let headers: &hyper::HeaderMap = req.headers();

    let cf_ip_opt = headers
        .get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok());

    let cf_ip: IpAddr = if let Some(ip_str) = cf_ip_opt {
        match ip_str.parse() {
            Ok(ip) => ip,
            Err(_) => {
                // Invalid IP format in header
                return err!(
                    StatusCode::FORBIDDEN,
                    format!(
                        "Invalid CF-Connecting-IP header: '{}' |x| {}",
                        ip_str,
                        dump_headers!(headers)
                    )
                );
            }
        }
    } else {
        #[cfg(debug_assertions)]
        {
            // In debug, use localhost IP
            "127.0.0.1".parse().unwrap()
        }

        #[cfg(not(debug_assertions))]
        {
            return err!(
                StatusCode::FORBIDDEN,
                format!(
                    "Missing CF-Connecting-IP header |x| {}",
                    dump_headers!(headers)
                )
            );
        }
    };

    // log the request: ip, method, path UA
    info!(
        "[{}] {} {} {}",
        cf_ip,
        req.method(),
        req.uri().path(),
        headers
            .get("User-Agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("Unknown User-Agent")
    );

    let response_builder = Response::builder()
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

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let nonce = crypt::generate_nonce_base64(32);
            let messages: Vec<String> = db::get_messages()
                .await
                .into_iter()
                .map(|m| m.content)
                .collect();

            match template::render(template::Template {
                nbusers: ws::get_user_count().await,
                nonce: &nonce,
                messages,
            }) {
                Ok(body) => Ok(response_builder
                    .header("Content-Security-Policy", format!(
                        "default-src 'none'; script-src 'self' 'nonce-{nonce}'; style-src 'self'; img-src 'self'; connect-src 'self'"
                    ))
                    .header("Cache-Control", "no-cache, no-store, must-revalidate")
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(full!(body)).unwrap()),
                Err(e) => {
                    err!(StatusCode::INTERNAL_SERVER_ERROR, format!("[{cf_ip}] Internal Server Error |x| {e}"))
                }
            }
        }

        (&Method::GET, "/ws") => {
            if hyper_tungstenite::is_upgrade_request(&req) {
                let (response, websocket) = hyper_tungstenite::upgrade(&mut req, None).unwrap();
                tokio::spawn(async move {
                    if let Err(e) = ws::handle_websocket(websocket).await {
                        error!("[{}] WebSocket error: {}", cf_ip, e);
                    }
                });
                Ok(response)
            } else {
                err!(
                    StatusCode::BAD_REQUEST,
                    format!(
                        "[{}] Bad Request: Not a WebSocket upgrade request |x| {}",
                        cf_ip,
                        dump_headers!(headers)
                    )
                )
            }
        }

        (&Method::GET, "/favicon.ico") => Ok(response_builder
            .header("Cache-Control", "public, max-age=86400")
            .header("Content-Type", "image/x-icon")
            .body(full!(F_FAVICON))
            .unwrap()),

        (&Method::GET, "/sitemap.xml") => Ok(response_builder
            .header("Cache-Control", "public, max-age=86400")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(full!(F_SITEMAP))
            .unwrap()),

        (&Method::GET, "/robots.txt") => Ok(response_builder
            .header("Cache-Control", "public, max-age=86400")
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(full!(F_ROBOTS))
            .unwrap()),

        (&Method::GET, "/styles.css") => Ok(response_builder
            .header("Cache-Control", "no-cache, no-store, must-revalidate")
            .header("Content-Type", "text/css; charset=utf-8")
            .body(full!(F_STYLES))
            .unwrap()),

        (&Method::GET, "/images/9d878e595dc522b07a801eae0fc6974d.webp") => Ok(response_builder
            .header("Cache-Control", "public, max-age=31536000, immutable")
            .header("Content-Type", "image/webp")
            .body(full!(F_BG))
            .unwrap()),

        // Return 404 Not Found for other routes.
        _ => err!(
            StatusCode::NOT_FOUND,
            format!(
                "[{}] 404 Not Found |x| {} {}",
                cf_ip,
                req.method(),
                req.uri().path()
            )
        ),
    }
}
