use crate::models::GitHubItem;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    #[serde(default)]
    items: Vec<GitHubItemResponse>,
}

#[derive(Debug, Deserialize)]
struct GitHubItemResponse {
    path: Option<String>,
    html_url: Option<String>,
    repository: Option<GitHubRepositoryResponse>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepositoryResponse {
    full_name: Option<String>,
    language: Option<String>,
}

pub async fn search_github(
    client: &Client,
    token: &str,
    phone_number: &str,
) -> Result<Vec<GitHubItem>, String> {
    if token.trim().is_empty() {
        return Ok(vec![]);
    }

    let url = "https://api.github.com/search/code";
    let query = format!("\"{phone_number}\"");

    let resp = client
        .get(url)
        .header("Authorization", format!("token {token}"))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "SearchPhone-OSINT/1.0")
        .query(&[("q", query.as_str())])
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        let data: GitHubSearchResponse = resp.json().await.map_err(|e| e.to_string())?;
        let mut results = Vec::new();

        for item in data.items.into_iter().take(10) {
            let (repo_name, language) = if let Some(repo) = item.repository {
                (
                    repo.full_name.unwrap_or_else(|| "Unknown".to_string()),
                    repo.language.unwrap_or_default(),
                )
            } else {
                ("Unknown".to_string(), String::new())
            };

            results.push(GitHubItem {
                repository: repo_name,
                path: item.path.unwrap_or_default(),
                url: item.html_url.unwrap_or_default(),
                language,
            });
        }

        return Ok(results);
    }

    Err(format!("GitHub API returned HTTP {}", resp.status()))
}
