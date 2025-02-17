use std::path::Path;
use crate::types;

pub fn init() {
    let config_path = Path::new("config.json");
    if !config_path.exists() {
        let config = types::config::Config {
            web_server_port: 443,
            enet_server_port: 17111,
        };
        let config_json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write("config.json", config_json).expect("Failed to write config file");
    }
}

fn get_config() -> types::config::Config {
    let config_json = std::fs::read_to_string("config.json").expect("Failed to read config file");
    serde_json::from_str(&config_json).expect("Failed to parse config file")
}

pub fn get_web_server_port() -> u16 {
    get_config().web_server_port
}

pub fn get_enet_server_port() -> u16 {
    get_config().enet_server_port
}

pub fn set_web_server_port(port: u16) {
    let mut config = get_config();
    config.web_server_port = port;
    let config_json = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write("config.json", config_json).expect("Failed to write config file");
}

pub fn set_enet_server_port(port: u16) {
    let mut config = get_config();
    config.enet_server_port = port;
    let config_json = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write("config.json", config_json).expect("Failed to write config file");
}