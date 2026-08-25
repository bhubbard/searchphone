use crate::models::{DorkFinding, SearchResult};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DorkQuery {
    pub category: &'static str,
    pub template: &'static str,
}

pub const DORK_TEMPLATES: &[DorkQuery] = &[
    DorkQuery {
        category: "Paste Sites & Dumps",
        template: "site:pastebin.com OR site:justpaste.it OR site:rentry.co OR site:ghostbin.com",
    },
    DorkQuery {
        category: "Leaked Documents & Spreadsheets",
        template: "filetype:pdf OR filetype:xlsx OR filetype:csv OR filetype:docx",
    },
    DorkQuery {
        category: "Social Profiles & Directories",
        template: "site:linkedin.com/in OR site:facebook.com OR site:instagram.com OR site:twitter.com",
    },
    DorkQuery {
        category: "Classifieds & Marketplaces",
        template: "site:craigslist.org OR site:offerup.com OR site:ebay.com",
    },
];

pub fn generate_dorks(phone: &str) -> Vec<(String, String)> {
    let clean = phone.replace([' ', '-'], "");
    DORK_TEMPLATES
        .iter()
        .map(|d| {
            let query = format!("{} \"{}\"", d.template, clean);
            (d.category.to_string(), query)
        })
        .collect()
}

pub async fn run_dorks(client: &Client, phone: &str) -> Vec<DorkFinding> {
    let dork_list = generate_dorks(phone);
    let mut findings = Vec::new();

    // Query DuckDuckGo API or simulated endpoint for each dork
    for (category, query) in dork_list {
        let results = search_dork_ddg(client, &query).await.unwrap_or_default();
        if !results.is_empty() {
            findings.push(DorkFinding {
                category,
                query,
                results,
            });
        }
    }

    findings
}

async fn search_dork_ddg(client: &Client, query: &str) -> Result<Vec<SearchResult>, String> {
    let url = "https://api.duckduckgo.com/";
    let resp = client
        .get(url)
        .query(&[("q", query), ("format", "json"), ("no_html", "1")])
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            let mut results = Vec::new();
            if let Some(abs) = v.get("AbstractText").and_then(|a| a.as_str()) {
                if !abs.trim().is_empty() {
                    results.push(SearchResult {
                        title: v.get("Abstract").and_then(|a| a.as_str()).unwrap_or("Abstract").to_string(),
                        link: v.get("AbstractURL").and_then(|u| u.as_str()).unwrap_or_default().to_string(),
                        snippet: abs.chars().take(200).collect(),
                    });
                }
            }
            return Ok(results);
        }
    }

    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_dorks() {
        let dorks = generate_dorks("+16502530000");
        assert_eq!(dorks.len(), 4);
        assert!(dorks[0].1.contains("+16502530000"));
    }
}
