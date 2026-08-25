use crate::models::SearchResult;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    web: Option<BraveWebResults>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    #[serde(default)]
    results: Vec<BraveSearchResultItem>,
}

#[derive(Debug, Deserialize)]
struct BraveSearchResultItem {
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
}

pub async fn search_brave(
    client: &Client,
    api_key: &str,
    phone_number: &str,
) -> Result<Vec<SearchResult>, String> {
    if api_key.trim().is_empty() {
        return Ok(vec![]);
    }

    let url = "https://api.search.brave.com/res/v1/web/search";
    let query = format!("\"{phone_number}\"");

    let resp = client
        .get(url)
        .header("X-Subscription-Token", api_key)
        .header("Accept", "application/json")
        .query(&[("q", query.as_str()), ("count", "10")])
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        let data: BraveSearchResponse = resp.json().await.map_err(|e| e.to_string())?;
        let mut results = Vec::new();

        if let Some(web) = data.web {
            for item in web.results {
                results.push(SearchResult {
                    title: item.title.unwrap_or_else(|| "Brave Result".to_string()),
                    link: item.url.unwrap_or_default(),
                    snippet: item.description.unwrap_or_default(),
                });
            }
        }
        return Ok(results);
    }

    Err(format!("Brave API returned HTTP {}", resp.status()))
}
