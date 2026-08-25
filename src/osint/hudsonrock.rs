use crate::models::{HudsonRockResult, StealerInfo};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct HudsonRockApiResponse {
    message: Option<String>,
    #[serde(default)]
    stealers: Vec<HudsonRockStealer>,
    #[serde(default)]
    total_corporate_services: Option<u64>,
    #[serde(default)]
    total_user_services: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct HudsonRockStealer {
    stealer_family: Option<String>,
    date_compromised: Option<String>,
    computer_name: Option<String>,
    operating_system: Option<String>,
    #[serde(default)]
    top_logins: Vec<String>,
}

pub async fn check_hudsonrock(
    client: &Client,
    phone_number: &str,
) -> Result<Option<HudsonRockResult>, String> {
    let clean_number = phone_number.replace([' ', '+'], "");
    let formatted_number = if !phone_number.starts_with('+') {
        format!("+{clean_number}")
    } else {
        phone_number.to_string()
    };

    let url = "https://cavalier.hudsonrock.com/api/json/v2/osint-tools/search-by-username";
    let resp = client
        .get(url)
        .query(&[("username", &formatted_number)])
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().as_u16() == 404 {
        return Ok(Some(HudsonRockResult {
            found: false,
            message: "Phone number not found in database".to_string(),
            stealers: vec![],
            total_corporate_services: 0,
            total_user_services: 0,
        }));
    }

    if resp.status().is_success() {
        let data: HudsonRockApiResponse = resp.json().await.map_err(|e| e.to_string())?;
        let has_stealers = !data.stealers.is_empty();

        let stealers = data
            .stealers
            .into_iter()
            .map(|s| StealerInfo {
                stealer_family: s.stealer_family,
                date_compromised: s.date_compromised,
                computer_name: s.computer_name,
                operating_system: s.operating_system,
                top_logins: s.top_logins,
            })
            .collect();

        if has_stealers {
            return Ok(Some(HudsonRockResult {
                found: true,
                message: data.message.unwrap_or_default(),
                stealers,
                total_corporate_services: data.total_corporate_services.unwrap_or(0),
                total_user_services: data.total_user_services.unwrap_or(0),
            }));
        } else {
            return Ok(Some(HudsonRockResult {
                found: false,
                message: "No infostealer records found".to_string(),
                stealers: vec![],
                total_corporate_services: 0,
                total_user_services: 0,
            }));
        }
    }

    Err(format!("HTTP status {}", resp.status()))
}
