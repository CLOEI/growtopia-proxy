mod resolver;

use std::{env, thread};
use std::net::SocketAddr;
use std::path::PathBuf;
use axum::http::StatusCode;
use axum::Router;
use axum::routing::post;
use axum_server::tls_rustls::RustlsConfig;
use log::info;
use serde_json::Value;

fn main() {
    #[cfg(debug_assertions)]
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or_else(|_| "growtopia_proxy".to_string()));

    env_logger::init();
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    info!("Growtopia Proxy started");

    thread::spawn(|| {
        setup_webserver();
    });

    loop {}
}

#[tokio::main]
async fn setup_webserver() {
    info!("Setting up webserver");
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("cert.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("key.pem"),
    ).await.unwrap();
    let app = Router::new()
        .route("/growtopia/server_data.php", post(server_data));
    let addr = SocketAddr::from(([127, 0, 0, 1], 443));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn server_data() -> (StatusCode, String) {
    info!("Received request for server data");
    let ip = resolver::resolve_ip("www.growtopia1.com").unwrap();
    let server_data = resolver::resolve_server_data(&ip).unwrap();
    (StatusCode::OK, server_data)
}