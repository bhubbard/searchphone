use crate::models::SearchResult;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct DuckDuckGoApiResponse {
    #[serde(rename = "Abstract")]
    r#abstract: Option<String>,
    #[serde(rename = "AbstractURL")]
    abstract_url: Option<String>,
    #[serde(rename = "AbstractText")]
    abstract_text: Option<String>,
}

pub async fn search_duckduckgo(
    client: &Client,
    phone_number: &str,
) -> Result<Vec<SearchResult>, String> {
    let url = "https://api.duckduckgo.com/";
    let query = format!("\"{phone_number}\"");

    let resp = client
        .get(url)
        .query(&[("q", query.as_str()), ("format", "json"), ("no_html", "1")])
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        let data: DuckDuckGoApiResponse = resp.json().await.map_err(|e| e.to_string())?;
        if let Some(text) = data.abstract_text {
            if !text.trim().is_empty() {
                let snippet = if text.len() > 200 {
                    text[..200].to_string()
                } else {
                    text
                };

                return Ok(vec![SearchResult {
                    title: data.r#abstract.unwrap_or_else(|| "Abstract".to_string()),
                    link: data.abstract_url.unwrap_or_default(),
                    snippet,
                }]);
            }
        }
    }

    Ok(vec![])
}
