use crate::models::NumLookupResult;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct NumLookupApiResponse {
    valid: Option<bool>,
    country_code: Option<String>,
    country_name: Option<String>,
    location: Option<String>,
    carrier: Option<String>,
    line_type: Option<String>,
}

pub async fn check_numlookup(
    client: &Client,
    api_key: &str,
    phone_number: &str,
) -> Result<Option<NumLookupResult>, String> {
    let clean_number: String = phone_number.chars().filter(|c| c.is_ascii_digit()).collect();
    let web_url = format!("https://www.numlookup.com/{}", clean_number);

    if clean_number.is_empty() {
        return Ok(None);
    }

    // 1. If API key is available, query the official NumLookup API
    if !api_key.trim().is_empty() {
        let url = format!(
            "https://api.numlookupapi.com/v1/validate/{}?apikey={}",
            clean_number,
            api_key.trim()
        );

        let resp = client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            if let Ok(data) = resp.json::<NumLookupApiResponse>().await {
                return Ok(Some(NumLookupResult {
                    valid: data.valid.unwrap_or(true),
                    country_code: data.country_code,
                    country_name: data.country_name,
                    location: data.location,
                    carrier: data.carrier,
                    line_type: data.line_type,
                    owner_name: None,
                    web_url,
                }));
            }
        }
    }

    // 2. Fallback / direct portal metadata
    Ok(Some(NumLookupResult {
        valid: true,
        country_code: None,
        country_name: None,
        location: None,
        carrier: None,
        line_type: None,
        owner_name: None,
        web_url,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_numlookup_portal_generation() {
        let client = Client::new();
        let res = check_numlookup(&client, "", "+16502530000").await.unwrap();
        assert!(res.is_some());
        let val = res.unwrap();
        assert_eq!(val.web_url, "https://www.numlookup.com/16502530000");
    }
}
