use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::thread;
use std::time::Duration;
use byteorder::LittleEndian;
use log::{error, info};
use rusty_enet as enet;
use crate::{global, packet_handler};
use byteorder::ByteOrder;
use crate::types::epacket_type::EPacketType;

pub fn setup() {
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
                    info!("Client Peer {} connected", peer.0);
                    global().client_peer_id.lock().unwrap().replace(peer);
                }
                enet::EventNoRef::Disconnect { peer, .. } => {
                    info!("Client Peer {} disconnected", peer.0);
                    global().client_peer_id.lock().unwrap().take();
                }
                enet::EventNoRef::Receive {
                    peer,
                    mut packet,
                    ..
                } => {
                    global().client_peer_id.lock().unwrap().replace(peer);
                    packet_handler::handle(&mut packet, true);
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}