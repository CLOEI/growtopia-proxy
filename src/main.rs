mod resolver;
mod utils;
mod enet;
mod types;
mod packet_handler;
mod variant_handler;

use std::{env, thread};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::path::PathBuf;
use axum::http::{HeaderMap, Response, StatusCode, Uri};
use axum::response::{Html, IntoResponse};
use axum::{Form, Router};
use axum::routing::post;
use axum_server::tls_rustls::RustlsConfig;
use log::{error, info};
use serde_json::Value;
use serde::Deserialize;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use rusty_enet;
use std::str::FromStr;
use crate::utils::text_parse::{map_to_string, parse_and_store_as_map};

struct GlobalData {
    server_data: Mutex<HashMap<String, String>>,
    server_enet_host: Mutex<Option<rusty_enet::Host<UdpSocket>>>,
    server_peer_id: Mutex<Option<rusty_enet::PeerID>>,
    client_enet_host: Mutex<Option<rusty_enet::Host<UdpSocket>>>,
    client_peer_id: Mutex<Option<rusty_enet::PeerID>>,
}

fn global() -> &'static GlobalData {
    static GLOBAL: OnceLock<GlobalData> = OnceLock::new();
    GLOBAL.get_or_init(|| {
        GlobalData {
            server_data: Mutex::new(HashMap::new()),
            server_enet_host: Mutex::new(None),
            server_peer_id: Mutex::new(None),
            client_enet_host: Mutex::new(None),
            client_peer_id: Mutex::new(None),
        }
    })
}

fn main() {
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or_else(|_| "growtopia_proxy".to_string()));

    env_logger::init();
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    info!("Growtopia Proxy started");

    thread::spawn(|| {
        enet::server::setup();
    });

    thread::spawn(|| {
        enet::client::setup();
    });

    thread::spawn(|| {
        setup_webserver();
    });

    loop {}
}

#[tokio::main]
async fn setup_webserver() {
    info!("Running webserver");
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

async fn server_data(header_map: HeaderMap, Form(input): Form<resolver::ServerDataInput>) -> Html<String> {
    info!("Received server_data request: {:?}", &input);
    info!("Headers: {:?}", header_map);
    let ip = resolver::resolve_ip("www.growtopia1.com").unwrap();
    let server_data = resolver::resolve_server_data(&ip, input).unwrap();
    let mut data = global().server_data.lock().unwrap();
    let mut parsed = parse_and_store_as_map(&server_data);
    let address = format!("{}:{}", parsed["server"], parsed["port"]);
    info!("ENet client connecting to server: {}", address);
    if let Some(mut host) = global().client_enet_host.lock().unwrap().as_mut() {
        let address = SocketAddr::from_str(&address).unwrap();
        match host.connect(address, 1, 0) {
            Ok(..) => {
                info!("ENet client connected to server");
            }
            Err(err) => {
                error!("ENet client unable to connect to server: {}", err);
            }
        }
    }
    *data = parsed.clone();
    parsed.insert("server".to_string(), "127.0.0.1".to_string());
    parsed.insert("port".to_string(), "17176".to_string());
    Html(map_to_string(&parsed))
}