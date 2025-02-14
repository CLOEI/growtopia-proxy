use std::net::SocketAddr;
use log::info;
use crate::global;
use crate::utils::text_parse;
use crate::utils::variant::VariantList;

pub fn handle(data: &[u8]) {
    let variant = VariantList::deserialize(&data).unwrap();
    let function_call: String = variant.get(0).unwrap().as_string();
    info!("Received function call: {}", function_call);

    match function_call.as_str() {
        "OnSendToServer" => {
            let port = variant.get(1).unwrap().as_int32();
            let token = variant.get(2).unwrap().as_int32();
            let user_id = variant.get(3).unwrap().as_int32();
            let server_data = variant.get(4).unwrap().as_string();
            let parsed_server_data = text_parse::parse_and_store_as_vec(&server_data);

            let ip = parsed_server_data.get(0).unwrap().to_string();
            let mut global_server_data = global().server_data.lock().unwrap();
            global_server_data.insert("ip".to_string(), ip);
        }
        _ => {}
    }
}