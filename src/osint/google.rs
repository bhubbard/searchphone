use crate::models::SearchResult;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct SerpApiResponse {
    #[serde(default)]
    organic_results: Vec<SerpApiOrganicResult>,
}

#[derive(Debug, Deserialize)]
struct SerpApiOrganicResult {
    title: Option<String>,
    link: Option<String>,
    snippet: Option<String>,
}

pub async fn search_google(
    client: &Client,
    api_key: &str,
    phone_number: &str,
) -> Result<Vec<SearchResult>, String> {
    if api_key.trim().is_empty() {
        return Ok(vec![]);
    }

    let clean_number = phone_number.replace([' ', '+', '-'], "");
    let e164_number = if !phone_number.starts_with('+') {
        format!("+{clean_number}")
    } else {
        phone_number.to_string()
    };

    let query = format!(
        "\"{}\" OR \"{}\" OR \"{}\" OR \"{}\"",
        phone_number,
        e164_number,
        clean_number,
        phone_number.replace(' ', "")
    );

    let url = "https://serpapi.com/search";
    let resp = client
        .get(url)
        .query(&[
            ("q", query.as_str()),
            ("api_key", api_key),
            ("num", "20"),
            ("hl", "en"),
        ])
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("SerpAPI returned HTTP {}", resp.status()));
    }

    let data: SerpApiResponse = resp.json().await.map_err(|e| e.to_string())?;

    // Build format patterns to filter relevant results
    let mut formats = vec![
        phone_number.to_string(),
        e164_number.clone(),
        clean_number.clone(),
        phone_number.replace(' ', ""),
        format!("+{clean_number}"),
    ];
    if phone_number.contains('-') {
        formats.push(phone_number.replace('-', ""));
    }
    if phone_number.contains(' ') {
        let parts: Vec<&str> = phone_number.split_whitespace().collect();
        formats.push(parts.join(""));
        formats.push(parts.join(" "));
    }

    let mut results = Vec::new();
    for item in data.organic_results {
        let title = item.title.unwrap_or_default();
        let link = item.link.unwrap_or_default();
        let snippet = item.snippet.unwrap_or_default();

        let matched = formats
            .iter()
            .any(|fmt| title.contains(fmt) || link.contains(fmt) || snippet.contains(fmt));

        if matched {
            results.push(SearchResult {
                title,
                link,
                snippet,
            });
        }
    }

    Ok(results)
}
