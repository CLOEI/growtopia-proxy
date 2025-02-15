use log::{error, info};
use serde::Deserialize;
use serde_json::Value;
use ureq::Error;
use crate::utils::text_parse::{map_to_string, parse_and_store_as_map};

pub fn resolve_ip(domain: &str) -> Option<String> {
    let doh_response = ureq::get("https://1.1.1.1/dns-query")
        .header("Accept", "application/dns-json")
        .query("name", "www.growtopia1.com")
        .query("type", "A")
        .call();

    match doh_response {
        Ok(mut resp) => {
            let body: Result<Value, Error> = resp.body_mut().read_json();
            match body {
                Ok(json) => {
                    let ip = json["Answer"][json["Answer"].as_array().unwrap().len() - 1]["data"].as_str().unwrap();
                    info!("Resolved IP: {}", ip);
                    Some(ip.to_string())
                }
                Err(e) => {
                    error!("Failed to parse JSON: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            error!("Failed to resolve IP: {}", e);
            None
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ServerDataInput {
    version: String,
    platform: String,
    protocol: String,
}

pub fn resolve_server_data(ip: &str, input: ServerDataInput) -> Option<String> {
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .tls_config(ureq::tls::TlsConfig::builder().disable_verification(true).build())
            .build()
    );

    let query = format!("https://{}/growtopia/server_data.php", ip);
    info!("Querying {}", query);
    let response = agent.post(&query)
        .header("Host", "www.growtopia1.com")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", "UbiServices_SDK_2022.Release.9_PC64_ansi_static")
        .send(format!("version={}&platform={}&protocol={}", input.version, input.platform, input.protocol));

    match response {
        Ok(mut resp) => {
            let body = resp.body_mut().read_to_string();
            match body {
                Ok(body) => {
                    info!("Server data: {}", body);
                    let mut modified = parse_and_store_as_map(&body);
                    modified.insert("type2".to_string(), "0".to_string());
                    let modified = map_to_string(&modified);
                    Some(format!("{}", modified))
                }
                Err(e) => {
                    error!("Failed to read response body: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            error!("Failed to query server data: {}", e);
            None
        }
    }
}