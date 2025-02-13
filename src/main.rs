mod resolver;
mod utils;

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
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use rusty_enet as enet;
use crate::utils::text_parse::parse_and_store_as_map;

fn http_data() -> &'static Mutex<HashMap<String, String>> {
    static HASHMAP: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let map = HashMap::new();
        Mutex::new(map)
    })
}

fn main() {
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or_else(|_| "growtopia_proxy".to_string()));

    env_logger::init();
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    info!("Growtopia Proxy started");

    thread::spawn(|| {
        setup_webserver();
    });

    thread::spawn(|| {
        setup_enet_server();
    });

    thread::spawn(|| {
        setup_enet_client();
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
    let mut data = http_data().lock().unwrap();
    *data = parse_and_store_as_map(&server_data);
    Html(server_data)
}

fn setup_enet_server() {
    info!("Running ENet server");
    let socket = UdpSocket::bind(SocketAddr::from_str("127.0.0.1:17176").unwrap()).expect("Failed to bind UDP socket");
    let mut host = enet::Host::new(
        socket,
        enet::HostSettings {
            peer_limit: 1,
            channel_limit: 1,
            compressor: Some(Box::new(enet::RangeCoder::new())),
            checksum: Some(Box::new(enet::crc32)),
            using_new_packet_server: true,
            ..Default::default()
        },
    ).expect("Failed to create ENet Server Host");

    'server_loop: loop {
        while let Some(event) = match host.service() {
            Ok(event) => event,
            Err(e) => {
                error!("Error: {}", e);
                continue 'server_loop;
            }
        } {
            match event {
                enet::Event::Connect { peer, .. } => {
                    println!("Peer {} connected", peer.id().0);
                }
                enet::Event::Disconnect { peer, .. } => {
                    println!("Peer {} disconnected", peer.id().0);
                }
                enet::Event::Receive {
                    peer,
                    packet,
                    ..
                } => {
                    println!("Received packet from peer");
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn setup_enet_client() {
    info!("Running ENet client");
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).expect("Failed to bind UDP socket");
    let host = enet::Host::<UdpSocket>::new(
        socket,
        enet::HostSettings {
            peer_limit: 1,
            channel_limit: 1,
            compressor: Some(Box::new(enet::RangeCoder::new())),
            checksum: Some(Box::new(enet::crc32)),
            using_new_packet: true,
            ..Default::default()
        },
    ).expect("Failed to create ENet Client Host");
}