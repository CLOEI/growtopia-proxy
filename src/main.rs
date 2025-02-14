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
use crate::utils::text_parse::{map_to_string, parse_and_store_as_map};

struct GlobalData {
    server_data: Mutex<HashMap<String, String>>,
    server_enet_host: Mutex<Option<enet::Host<UdpSocket>>>,
    server_peer_id: Mutex<Option<enet::PeerID>>,
    client_enet_host: Mutex<Option<enet::Host<UdpSocket>>>,
    client_peer_id: Mutex<Option<enet::PeerID>>,
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

    global().server_enet_host.lock().unwrap().replace(host);

    loop {
        let event = {
            let mut host = global().server_enet_host.lock().unwrap();
            if let Some(host) = &mut *host {
                host.service().ok().flatten().map(|e| e.no_ref())
            } else {
                break;
            }
        };

        if let Some(event) = event  {
            match event {
                enet::EventNoRef::Connect { peer, .. } => {
                    info!("Peer {} connected", peer.0);
                    global().server_peer_id.lock().unwrap().replace(peer);
                }
                enet::EventNoRef::Disconnect { peer, .. } => {
                    info!("Peer {} disconnected", peer.0);
                    global().server_peer_id.lock().unwrap().take();
                }
                enet::EventNoRef::Receive {
                    peer,
                    packet,
                    ..
                } => {
                    global().server_peer_id.lock().unwrap().replace(peer);
                    info!("Received packet from peer Server");
                    if let Ok(peer_id) = global().client_peer_id.lock() {
                        if let Some(peer_id) = *peer_id {
                            if let Ok(mut host) = global().client_enet_host.lock() {
                                let mut host = host.as_mut().unwrap();
                                let peer = host.peer_mut(peer_id);
                                if let Err(err) = peer.send(0, &packet) {
                                    error!("Server failed sending packet to client: {}", err);
                                } else {
                                    info!("Server sent packet to peer client");
                                }
                            } else {
                                error!("Server failed to send packet to client: Client host is None");
                            }
                        } else {
                            error!("Server failed to send packet to client: Client peer ID is None");
                        }
                    } else {
                        error!("Server failed to send packet to client: Client peer ID lock failed");
                    }
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

    global().client_enet_host.lock().unwrap().replace(host);

    loop {
        {
            let server_peer_id = global().server_peer_id.lock().unwrap();
            if server_peer_id.is_none() {
                continue;
            }
        }

        let event = {
            let mut host = global().client_enet_host.lock().unwrap();
            if let Some(host) = &mut *host {
                host.service().ok().flatten().map(|e| e.no_ref())
            } else {
                break;
            }
        };

        if let Some(event) = event {
            match event {
                enet::EventNoRef::Connect { peer, .. } => {
                    println!("Peer {} connected", peer.0);
                    global().client_peer_id.lock().unwrap().replace(peer);
                }
                enet::EventNoRef::Disconnect { peer, .. } => {
                    println!("Peer {} disconnected", peer.0);
                    global().client_peer_id.lock().unwrap().take();
                }
                enet::EventNoRef::Receive {
                    peer,
                    packet,
                    ..
                } => {
                    global().client_peer_id.lock().unwrap().replace(peer);
                    info!("Peer Client received packet");
                    if let Ok(peer_id) = global().server_peer_id.lock() {
                        if let Some(peer_id) = *peer_id {
                            if let Ok(mut host) = global().server_enet_host.lock() {
                                let mut host = host.as_mut().unwrap();
                                let peer = host.peer_mut(peer_id);
                                if let Err(err) = peer.send(0, &packet) {
                                    error!("Client failed sending packet to server: {}", err);
                                } else {
                                    info!("Client sent packet to peer server");
                                }
                            } else {
                                error!("Client failed to send packet to server: Server host is None");
                            }
                        } else {
                            error!("Client failed to send packet to server: Server peer ID is None");
                        }
                    } else {
                        error!("Client failed to send packet to server: Server peer ID lock failed");
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}