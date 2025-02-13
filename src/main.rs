mod resolver;

use std::env;
use log::info;
use serde_json::Value;
use ureq::Agent;

fn main() {
    #[cfg(debug_assertions)]
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or_else(|_| "growtopia_proxy".to_string()));

    env_logger::init();
    info!("Growtopia Proxy started");
    let ip = resolver::resolve_ip("www.growtopia1.com").unwrap();
    let server_data = resolver::resolve_server_data(&ip).unwrap();
}