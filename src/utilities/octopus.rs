use lazy_static::lazy_static;
use reqwest;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};

use std::env;

lazy_static! {
    pub static ref OCTOPUS_CONFIG: OctopusConfig = OctopusConfig {
        mpan: env::var("OCTOPUS_ENERGY_ELECTRICITY_METER_MPAN").unwrap(),
        serial_number: env::var("OCTOPUS_ENERGY_ELECTRICITY_METER_SERIAL_NUMBER").unwrap(),
        api_key: env::var("OCTOPUS_ENERGY_API_KEY").unwrap(),
    };
}

pub struct OctopusConfig {
    pub mpan: String,
    pub serial_number: String,
    pub api_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct OctopusData {
    count: u32,
    next: String,
    previous: Option<String>,
    results: Vec<ConsumptionDataPiont>,
}

#[derive(Serialize, Deserialize)]
struct ConsumptionDataPiont {
    consumption: f32,
    interval_start: String,
    interval_end: String,
}

fn make_auth_headers() -> HeaderMap {
    let mut map = HeaderMap::new();

    let mut token = String::from("Token ");
    token.push_str(OCTOPUS_CONFIG.api_key.clone().as_str());

    map.insert(
        AUTHORIZATION,
        HeaderValue::from_str(token.as_str()).unwrap(),
    );
    map
}

pub async fn get_last_24h_electricity_consumption() -> OctopusData {
    let client = reqwest::Client::new();
    let response = client.get(format!(
        "https://api.octopus.energy/v1/electricity-meter-points/{}/meters/{}/consumption/?page_size=48",
        OCTOPUS_CONFIG.mpan, OCTOPUS_CONFIG.serial_number
    ))
        .basic_auth(OCTOPUS_CONFIG.api_key.clone(), Some(""))
        .send()
        .await
        .unwrap();

    serde_json::from_str(dbg!(response.text().await.unwrap().as_str())).unwrap()
}
