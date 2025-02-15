use std::net::SocketAddr;
use log::info;
use rusty_enet::Packet;
use crate::global;
use crate::packet_handler::resend_packet;
use crate::types::epacket_type::EPacketType;
use crate::types::tank_packet::TankPacket;
use crate::utils::text_parse;
use crate::utils::variant::{Variant, VariantList};

pub fn handle(data: &[u8], tank_packet: &mut TankPacket, packet_id: &[u8]) -> Option<Packet> {
    let mut variant = VariantList::deserialize(&data).unwrap();
    let function_call: String = variant.get(0).unwrap().as_string();
    info!("Received function call: {}", function_call);

    match function_call.as_str() {
        "OnSendToServer" => {
            let port = variant.get(1).unwrap().as_int32();
            let server_data = variant.get(4).unwrap().as_string();
            let mut parsed_server_data = text_parse::parse_and_store_as_vec(&server_data);
            let ip = parsed_server_data.get(0).unwrap().to_string();
            let mut global_server_data = global().server_data.lock().unwrap();
            global_server_data.insert("server".to_string(), ip);
            global_server_data.insert("port".to_string(), port.to_string());

            parsed_server_data[0] = "127.0.0.1".to_string();
            variant.set(1, Variant::Signed(17176));
            variant.set(4, Variant::String(text_parse::vec_to_string(&parsed_server_data)));

            let serialized = variant.serialize();
            let packet = recreate_variant(&serialized, tank_packet, packet_id);
            Some(packet)
        }
        "OnConsoleMessage" => {
            let message = variant.get(1).unwrap().as_string();
            info!("Received console message: {}", message);
            None
        },
        "OnDialogRequest" => {
            let message = variant.get(1).unwrap().as_string();
            info!("Received dialog request: {}", message);
            None
        },
        "OnSpawn" => {
            let message = variant.get(1).unwrap().as_string();
            let mut parsed_message = text_parse::parse_and_store_as_map(&message);

            if let Some(val) = parsed_message.get("type") {
                if val == "local" {
                    parsed_message.insert("mstate".to_string(), "1".to_string());
                    let new_message = text_parse::map_to_string(&parsed_message);
                    variant.set(1, Variant::String(new_message));

                    let serialized = variant.serialize();
                    let packet = recreate_variant(&serialized, tank_packet, packet_id);
                    Some(packet)
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => {
            None
        }
    }
}

pub fn recreate_variant(serialized: &Vec<u8>, tank_packet: &mut TankPacket, packet_id: &[u8]) -> Packet {
    tank_packet.extended_data_length = serialized.len() as u32;
    let serialized_tank_packet = bincode::serialize(&tank_packet).unwrap();
    let mut data = vec![0; 4 + serialized_tank_packet.len() + serialized.len()];
    data[0..4].copy_from_slice(&packet_id);
    data[4..4 + serialized_tank_packet.len()].copy_from_slice(&serialized_tank_packet);
    data[4 + serialized_tank_packet.len()..].copy_from_slice(&serialized);
    *tank_packet = bincode::deserialize(&data[4..]).unwrap();

    Packet::new(&*data, rusty_enet::PacketKind::Reliable)
}