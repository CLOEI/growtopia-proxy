use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub web_server_port: u16,
    pub enet_server_port: u16,
}