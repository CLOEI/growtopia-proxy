use std::net::SocketAddr;
use byteorder::{ByteOrder, LittleEndian};
use log::{error, info, warn};
use rusty_enet::Packet;
use crate::types::epacket_type::EPacketType;
use crate::types::etank_packet_type::ETankPacketType;
use crate::types::tank_packet::TankPacket;
use crate::{global, variant_handler};

pub fn handle(packet: &mut Packet, is_client: bool) {
    let data = packet.data();
    let packet_id = LittleEndian::read_u32(&data[0..4]);
    let packet_type = EPacketType::from(packet_id);
    info!("{} Received {:?} packet", if is_client { "Client" } else { "Server" }, packet_type);

    match packet_type {
        EPacketType::NetMessageGamePacket => match bincode::deserialize::<TankPacket>(&data[4..]) {
            Ok(mut tank_packet) => {
                info!("{} Received tank: {:?}", if is_client { "Client" } else { "Server" }, tank_packet.r#type);
                match tank_packet.r#type {
                    ETankPacketType::NetGamePacketCallFunction => {
                        let packet = variant_handler::handle(&data[60..], &mut tank_packet, &data[0..4]);
                        if let Some(packet) = packet {
                            resend_packet(&packet, &packet_type, is_client);
                            return;
                        }
                    },
                    ETankPacketType::NetGamePacketDisconnect => {
                        // Disconnect client
                        let mut client_host = global().client_enet_host.lock().unwrap();
                        let peer_id = global().client_peer_id.lock().unwrap();
                        if let Some(client_host) = &mut *client_host {
                            if let Some(peer_id) = &*peer_id {
                                let peer = client_host.peer_mut(*peer_id);
                                peer.disconnect_now(0);
                            }
                            let (server, port) = {
                                let server_data = global().server_data.lock().unwrap();
                                (server_data.get("server").unwrap().clone(), server_data.get("port").unwrap().clone())
                            };
                            let address = SocketAddr::new(server.parse().unwrap(), port.parse().unwrap());
                            match client_host.connect(address, 2, 0) {
                                Ok(..) => {
                                    info!("Client Re-Connected to server");
                                }
                                Err(e) => {
                                    error!("Client Failed to connect to server: {:?}", e);
                                }
                            }
                            client_host.flush();
                        }

                        // Disconnect server
                        disconnect(false);
                        return;
                    }
                    ETankPacketType::NetGamePacketAppIntegrityFail => {
                        warn!("{} App integrity fail packet blocked", if is_client { "Client" } else { "Server" });
                        return;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("{} Failed to deserialize tank packet: {:?}", if is_client { "Client" } else { "Server" }, e);
            }
        },
        EPacketType::NetMessageGameMessage => {
            let message = String::from_utf8_lossy(&data);
            info!("{} Received message: {}", if is_client { "Client" } else { "Server" }, message);
            if message == "action|quit" {
                disconnect(false);
            }
        }
        EPacketType::NetMessageGenericText => {
            let message = String::from_utf8_lossy(&data);
            info!("{} Received generic text: {}", if is_client { "Client" } else { "Server" }, message);
        },
        EPacketType::NetMessageTrack => {
            let message = String::from_utf8_lossy(&data);
            info!("{} Received track: {}", if is_client { "Client" } else { "Server" }, message);
        }
        _ => {}
    }
    resend_packet(packet, &packet_type, is_client);
}

pub fn resend_packet(packet: &Packet, packet_type: &EPacketType, is_client: bool) {
    let (host_lock, peer_id_lock) = if is_client {
        (&global().server_enet_host, &global().server_peer_id)
    } else {
        (&global().client_enet_host, &global().client_peer_id)
    };

    if let Ok(peer_id) = peer_id_lock.lock() {
        if let Some(peer_id) = *peer_id {
            if let Ok(mut host) = host_lock.lock() {
                if let Some(host) = host.as_mut() {
                    let peer = host.peer_mut(peer_id);
                    if let Err(err) = peer.send(0, &packet) {
                        error!("{} Failed sending packet: {}",  if is_client { "Client" } else { "Server" }, err);
                    } else {
                        info!("{} Sent {:?} packet", if is_client { "Client" } else { "Server" }, packet_type);
                    }
                } else {
                    error!("{} Failed to send packet: Host is None", if is_client { "Client" } else { "Server" });
                }
            } else {
                error!("{} Failed to send packet: Host lock failed", if is_client { "Client" } else { "Server" });
            }
        } else {
            error!("{} Failed to send packet: Peer ID is None", if is_client { "Client" } else { "Server" });
        }
    } else {
        error!("{} Failed to send packet: Peer ID lock failed", if is_client { "Client" } else { "Server" });
    }
}

pub fn disconnect(is_client: bool) {
    let (host_lock, peer_id_lock) = if is_client {
        (&global().client_enet_host, &global().client_peer_id)
    } else {
        (&global().server_enet_host, &global().server_peer_id)
    };

    let mut host = host_lock.lock().unwrap();
    let peer_id = peer_id_lock.lock().unwrap();
    if let Some(host) = &mut *host {
        if let Some(peer_id) = &*peer_id {
            let peer = host.peer_mut(*peer_id);
            peer.disconnect_now(0);
        }
        host.flush();
    }
}