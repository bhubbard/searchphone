use crate::models::RedditPost;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct RedditApiResponse {
    data: Option<RedditListingData>,
}

#[derive(Debug, Deserialize)]
struct RedditListingData {
    #[serde(default)]
    children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: Option<RedditPostData>,
}

#[derive(Debug, Deserialize)]
struct RedditPostData {
    title: Option<String>,
    subreddit: Option<String>,
    permalink: Option<String>,
    score: Option<i64>,
    created_utc: Option<f64>,
}

pub async fn search_reddit(
    client: &Client,
    phone_number: &str,
) -> Result<Vec<RedditPost>, String> {
    let url = "https://www.reddit.com/r/all/search.json";
    let query = format!("\"{phone_number}\"");

    let resp = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; PhoneOSINT/1.0)")
        .query(&[("q", query.as_str()), ("limit", "20")])
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        let data: RedditApiResponse = resp.json().await.map_err(|e| e.to_string())?;
        let mut results = Vec::new();

        if let Some(listing) = data.data {
            for child in listing.children {
                if let Some(post) = child.data {
                    let created = if let Some(utc) = post.created_utc {
                        DateTime::<Utc>::from_timestamp(utc as i64, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };

                    let permalink = post.permalink.unwrap_or_default();
                    let full_url = if !permalink.is_empty() {
                        format!("https://reddit.com{permalink}")
                    } else {
                        String::new()
                    };

                    results.push(RedditPost {
                        title: post.title.unwrap_or_else(|| "No title".to_string()),
                        subreddit: post.subreddit.unwrap_or_default(),
                        url: full_url,
                        score: post.score.unwrap_or(0),
                        created,
                    });
                }
            }
        }

        return Ok(results);
    }

    Ok(vec![])
}
