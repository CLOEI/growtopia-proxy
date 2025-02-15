use crate::{global, packet_handler};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use log::{error, info};
use rusty_enet as enet;
use std::str::FromStr;
use byteorder::LittleEndian;
use crate::types::epacket_type::EPacketType;
use byteorder::ByteOrder;

pub fn setup() {
    info!("Running ENet server");
    let socket = UdpSocket::bind(SocketAddr::from_str("127.0.0.1:17176").unwrap()).expect("Failed to bind UDP socket");
    let mut host = enet::Host::new(
        socket,
        enet::HostSettings {
            peer_limit: 1,
            channel_limit: 2,
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
                    info!("Server Peer {} connected", peer.0);
                    global().server_peer_id.lock().unwrap().replace(peer);
                    let mut server_host = global().server_enet_host.lock().unwrap();
                    let peer_id = global().server_peer_id.lock().unwrap();
                    if let Some(server_host) = &mut *server_host {
                        if let Some(peer_id) = &*peer_id {
                            let peer = server_host.peer_mut(*peer_id);
                            peer.set_timeout(0, 12000, 0);
                        }
                    }
                }
                enet::EventNoRef::Disconnect { peer, .. } => {
                    info!("Server Peer {} disconnected", peer.0);
                    global().server_peer_id.lock().unwrap().take();
                    packet_handler::disconnect(true);
                }
                enet::EventNoRef::Receive {
                    peer,
                    mut packet,
                    ..
                } => {
                    global().server_peer_id.lock().unwrap().replace(peer);
                    packet_handler::handle(&mut packet, false);
                }
            }
        }
        thread::sleep(Duration::from_millis(16));
    }
}