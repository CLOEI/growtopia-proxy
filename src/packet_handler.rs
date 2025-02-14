use byteorder::{ByteOrder, LittleEndian};
use log::{error, info, warn};
use rusty_enet::Packet;
use crate::types::epacket_type::EPacketType;
use crate::types::etank_packet_type::ETankPacketType;
use crate::types::tank_packet::TankPacket;
use crate::{global, variant_handler};

pub fn handle(packet: &mut Packet, is_client: bool) {
    let data = packet.data();
    if data.len() < 4 {
        return;
    }
    let packet_id = LittleEndian::read_u32(&data[0..4]);
    let packet_type = EPacketType::from(packet_id);
    info!("Received {:?} packet", packet_type);

    match packet_type {
        EPacketType::NetMessageGamePacket => match bincode::deserialize::<TankPacket>(&data[4..]) {
            Ok(tank_packet) => {
                info!("Received tank: {:?}", tank_packet.r#type);
                match tank_packet.r#type {
                    ETankPacketType::NetGamePacketCallFunction => variant_handler::handle(&data[60..]),
                    ETankPacketType::NetGamePacketDisconnect => {
                        let mut client_host = global().client_enet_host.lock().unwrap();
                        let peer_id = global().client_peer_id.lock().unwrap();
                        if let Some(client_host) = &mut *client_host {
                            if let Some(peer_id) = &*peer_id {
                                let peer = client_host.peer_mut(*peer_id);
                                peer.disconnect_now(0);
                            }
                        }
                        return;
                    }
                    ETankPacketType::NetGamePacketAppIntegrityFail => {
                        warn!("App integrity fail packet blocked");
                        return;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("Failed to deserialize tank packet: {:?}", e);
            }
        },
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
                        error!("Failed sending packet: {}", err);
                    } else {
                        info!("Sent {:?} packet", packet_type);
                    }
                } else {
                    error!("Failed to send packet: Host is None");
                }
            } else {
                error!("Failed to send packet: Host lock failed");
            }
        } else {
            error!("Failed to send packet: Peer ID is None");
        }
    } else {
        error!("Failed to send packet: Peer ID lock failed");
    }
}