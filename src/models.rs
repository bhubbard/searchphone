use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhoneInfo {
    pub international: String,
    pub national: String,
    pub e164: String,
    pub country: String,
    pub carrier: String,
    pub timezone: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NumverifyResult {
    pub country: Option<String>,
    pub location: Option<String>,
    pub carrier: Option<String>,
    pub line_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NumLookupResult {
    pub valid: bool,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub location: Option<String>,
    pub carrier: Option<String>,
    pub line_type: Option<String>,
    pub owner_name: Option<String>,
    pub web_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StealerInfo {
    pub stealer_family: Option<String>,
    pub date_compromised: Option<String>,
    pub computer_name: Option<String>,
    pub operating_system: Option<String>,
    #[serde(default)]
    pub top_logins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HudsonRockResult {
    pub found: bool,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub stealers: Vec<StealerInfo>,
    #[serde(default)]
    pub total_corporate_services: u64,
    #[serde(default)]
    pub total_user_services: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RedditPost {
    pub title: String,
    pub subreddit: String,
    pub url: String,
    pub score: i64,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubItem {
    pub repository: String,
    pub path: String,
    pub url: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DorkFinding {
    pub category: String, // "Paste Site", "Leaked Document", "Social Profile", "Classified Ad"
    pub query: String,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreatReputation {
    pub risk_score: u8,      // 0 (Clean) to 100 (Critical)
    pub risk_level: String,  // "CLEAN", "LOW", "MEDIUM", "HIGH", "CRITICAL"
    pub spam_reports_count: u32,
    pub is_voip: bool,
    pub is_disposable: bool,
    pub breach_hits: Vec<String>,
    pub threat_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OsintResults {
    pub phone_info: Option<PhoneInfo>,
    pub numverify: Option<NumverifyResult>,
    pub numlookup: Option<NumLookupResult>,
    pub hudsonrock: Option<HudsonRockResult>,
    pub threat_reputation: Option<ThreatReputation>,
    #[serde(default)]
    pub dorks: Vec<DorkFinding>,
    #[serde(default)]
    pub google: Vec<SearchResult>,
    #[serde(default)]
    pub duckduckgo: Vec<SearchResult>,
    #[serde(default)]
    pub brave: Vec<SearchResult>,
    #[serde(default)]
    pub reddit: Vec<RedditPost>,
    #[serde(default)]
    pub github: Vec<GitHubItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub phone: String,
    pub region: String,
    pub timestamp: String,
    pub tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsintReport {
    pub metadata: ReportMetadata,
    pub results: OsintResults,
}

// Visualizer D3 Graph Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub group: String, // "phone", "machine", "stealer", "account", "repo", "dork", "social"
    pub val: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct D3GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}
