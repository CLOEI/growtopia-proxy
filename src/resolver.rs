use log::{error, info};
use serde_json::Value;
use ureq::Error;

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

pub fn resolve_server_data(ip: &str) -> Option<String> {
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .tls_config(ureq::tls::TlsConfig::builder().disable_verification(true).build())
            .build()
    );

    let query = format!("https://{}/growtopia/server_data.php", ip);
    info!("Querying {}", query);
    let response = agent.post(&query)
        .header("Host", "www.growtopia1.com")
        .header("User-Agent", "UbiServices_SDK_2022.Release.9_PC64_ansi_static")
        .send_empty();

    match response {
        Ok(mut resp) => {
            let body = resp.body_mut().read_to_string();
            match body {
                Ok(body) => {
                    info!("Server data: {}", body);
                    Some(body)
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