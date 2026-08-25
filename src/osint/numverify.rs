use crate::models::NumverifyResult;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct NumverifyApiResponse {
    valid: Option<bool>,
    country_name: Option<String>,
    location: Option<String>,
    carrier: Option<String>,
    line_type: Option<String>,
}

pub async fn check_numverify(
    client: &Client,
    api_key: &str,
    phone_number: &str,
    region: &str,
) -> Result<Option<NumverifyResult>, String> {
    if api_key.trim().is_empty() {
        return Ok(None);
    }

    let url = "https://apilayer.net/api/validate";
    let resp = client
        .get(url)
        .query(&[
            ("access_key", api_key),
            ("number", phone_number),
            ("country_code", &region.to_uppercase()),
        ])
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        let data: NumverifyApiResponse = resp.json().await.map_err(|e| e.to_string())?;
        if data.valid.unwrap_or(false) {
            return Ok(Some(NumverifyResult {
                country: data.country_name,
                location: data.location,
                carrier: data.carrier,
                line_type: data.line_type,
            }));
        }
    }

    Ok(None)
}
