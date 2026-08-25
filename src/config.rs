use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub numverify_key: String,
    pub numlookup_key: String,
    pub serpapi_key: String,
    pub brave_api_key: String,
    pub github_token: String,
    pub report_dir: String,
    pub proxy_url: Option<String>,
    pub redact: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        let _ = dotenvy::dotenv();

        Self {
            numverify_key: env::var("NUMVERIFY_KEY").unwrap_or_default(),
            numlookup_key: env::var("NUMLOOKUP_KEY")
                .or_else(|_| env::var("NUMLOOKUP_API_KEY"))
                .unwrap_or_default(),
            serpapi_key: env::var("SERPAPI_KEY").unwrap_or_default(),
            brave_api_key: env::var("BRAVE_API_KEY").unwrap_or_default(),
            github_token: env::var("GITHUB_TOKEN").unwrap_or_default(),
            report_dir: env::var("SEARCHPHONE_REPORT_DIR").unwrap_or_else(|_| "reports".to_string()),
            proxy_url: env::var("SEARCHPHONE_PROXY").ok().filter(|s| !s.trim().is_empty()),
            redact: env::var("SEARCHPHONE_REDACT").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false),
        }
    }

    pub fn build_client(&self) -> reqwest::Client {
        let mut builder = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36");

        if let Some(ref proxy) = self.proxy_url {
            if let Ok(p) = reqwest::Proxy::all(proxy) {
                builder = builder.proxy(p);
            }
        }

        builder.build().unwrap_or_else(|_| reqwest::Client::new())
    }
}
