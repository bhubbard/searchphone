pub mod brave;
pub mod duckduckgo;
pub mod github;
pub mod google;
pub mod hudsonrock;
pub mod numlookup;
pub mod numverify;
pub mod reddit;

use crate::config::AppConfig;
use crate::dorks;
use crate::models::{HudsonRockResult, NumLookupResult, NumverifyResult, OsintResults};
use colored::Colorize;

pub async fn run_osint_scans(
    config: &AppConfig,
    phone_number: &str,
    region: &str,
) -> OsintResults {
    let client = config.build_client();

    println!("{}", "╭── Reconnaissance Engine Scanning ──────────────────────────────────────────╮".bright_cyan());

    let (
        numverify_res,
        numlookup_res,
        hudsonrock_res,
        google_res,
        duckduckgo_res,
        brave_res,
        reddit_res,
        github_res,
        dorks_res,
    ) = tokio::join!(
        numverify::check_numverify(&client, &config.numverify_key, phone_number, region),
        numlookup::check_numlookup(&client, &config.numlookup_key, phone_number),
        hudsonrock::check_hudsonrock(&client, phone_number),
        google::search_google(&client, &config.serpapi_key, phone_number),
        duckduckgo::search_duckduckgo(&client, phone_number),
        brave::search_brave(&client, &config.brave_api_key, phone_number),
        reddit::search_reddit(&client, phone_number),
        github::search_github(&client, &config.github_token, phone_number),
        dorks::run_dorks(&client, phone_number),
    );

    let numverify: Option<NumverifyResult> = match numverify_res {
        Ok(Some(res)) => {
            println!(
                "│  {} {:<32} {}",
                "[API]".bright_cyan().bold(),
                "Numverify Telecom Lookup",
                "[✓ DATA RETRIEVED]".green().bold()
            );
            Some(res)
        }
        Ok(None) => {
            println!(
                "│  {} {:<32} {}",
                "[API]".bright_cyan().bold(),
                "Numverify Telecom Lookup",
                "[— NO DATA / NO KEY]".bright_black()
            );
            None
        }
        Err(e) => {
            println!(
                "│  {} {:<32} {}",
                "[API]".bright_cyan().bold(),
                "Numverify Telecom Lookup",
                format!("[✗ ERROR: {}]", e).red()
            );
            None
        }
    };

    let numlookup: Option<NumLookupResult> = match numlookup_res {
        Ok(Some(res)) => {
            if res.carrier.is_some() || res.location.is_some() {
                println!(
                    "│  {} {:<32} {}",
                    "[API]".bright_cyan().bold(),
                    "NumLookup Reverse Directory",
                    "[✓ DATA RETRIEVED]".green().bold()
                );
            } else {
                println!(
                    "│  {} {:<32} {}",
                    "[API]".bright_cyan().bold(),
                    "NumLookup Reverse Directory",
                    "[✓ PORTAL LINK GENERATED]".green()
                );
            }
            Some(res)
        }
        Ok(None) => {
            println!(
                "│  {} {:<32} {}",
                "[API]".bright_cyan().bold(),
                "NumLookup Reverse Directory",
                "[— NO DATA]".bright_black()
            );
            None
        }
        Err(e) => {
            println!(
                "│  {} {:<32} {}",
                "[API]".bright_cyan().bold(),
                "NumLookup Reverse Directory",
                format!("[✗ ERROR: {}]", e).red()
            );
            None
        }
    };

    let hudsonrock: Option<HudsonRockResult> = match hudsonrock_res {
        Ok(Some(res)) => {
            if res.found {
                println!(
                    "│  {} {:<32} {}",
                    "[THREAT-INTEL]".bright_red().bold(),
                    "Hudson Rock Infostealer DB",
                    format!("[⚠️ {} INFECTION(S) FOUND]", res.stealers.len()).red().bold()
                );
            } else {
                println!(
                    "│  {} {:<32} {}",
                    "[THREAT-INTEL]".bright_red().bold(),
                    "Hudson Rock Infostealer DB",
                    "[✓ CLEAN - NO RECORDS]".green().bold()
                );
            }
            Some(res)
        }
        Ok(None) => {
            println!(
                "│  {} {:<32} {}",
                "[THREAT-INTEL]".bright_red().bold(),
                "Hudson Rock Infostealer DB",
                "[— NO DATA]".bright_black()
            );
            None
        }
        Err(e) => {
            println!(
                "│  {} {:<32} {}",
                "[THREAT-INTEL]".bright_red().bold(),
                "Hudson Rock Infostealer DB",
                format!("[✗ ERROR: {}]", e).red()
            );
            None
        }
    };

    let google = match google_res {
        Ok(results) => {
            if !results.is_empty() {
                println!(
                    "│  {} {:<32} {}",
                    "[SEARCH]".bright_yellow().bold(),
                    "Google Search (SerpAPI)",
                    format!("[✓ {} HIT(S)]", results.len()).green().bold()
                );
            } else {
                println!(
                    "│  {} {:<32} {}",
                    "[SEARCH]".bright_yellow().bold(),
                    "Google Search (SerpAPI)",
                    "[— 0 HITS / NO KEY]".bright_black()
                );
            }
            results
        }
        Err(e) => {
            println!(
                "│  {} {:<32} {}",
                "[SEARCH]".bright_yellow().bold(),
                "Google Search (SerpAPI)",
                format!("[✗ ERROR: {}]", e).red()
            );
            vec![]
        }
    };

    let duckduckgo = match duckduckgo_res {
        Ok(results) => {
            if !results.is_empty() {
                println!(
                    "│  {} {:<32} {}",
                    "[SEARCH]".bright_yellow().bold(),
                    "DuckDuckGo Instant Answers",
                    format!("[✓ {} HIT(S)]", results.len()).green().bold()
                );
            } else {
                println!(
                    "│  {} {:<32} {}",
                    "[SEARCH]".bright_yellow().bold(),
                    "DuckDuckGo Instant Answers",
                    "[— 0 HITS]".bright_black()
                );
            }
            results
        }
        Err(e) => {
            println!(
                "│  {} {:<32} {}",
                "[SEARCH]".bright_yellow().bold(),
                "DuckDuckGo Instant Answers",
                format!("[✗ ERROR: {}]", e).red()
            );
            vec![]
        }
    };

    let brave = match brave_res {
        Ok(results) => {
            if !results.is_empty() {
                println!(
                    "│  {} {:<32} {}",
                    "[SEARCH]".bright_yellow().bold(),
                    "Brave Search API",
                    format!("[✓ {} HIT(S)]", results.len()).green().bold()
                );
            } else {
                println!(
                    "│  {} {:<32} {}",
                    "[SEARCH]".bright_yellow().bold(),
                    "Brave Search API",
                    "[— 0 HITS / NO KEY]".bright_black()
                );
            }
            results
        }
        Err(e) => {
            println!(
                "│  {} {:<32} {}",
                "[SEARCH]".bright_yellow().bold(),
                "Brave Search API",
                format!("[✗ ERROR: {}]", e).red()
            );
            vec![]
        }
    };

    let reddit = match reddit_res {
        Ok(results) => {
            if !results.is_empty() {
                println!(
                    "│  {} {:<32} {}",
                    "[SOCIAL]".bright_magenta().bold(),
                    "Reddit Discussion Search",
                    format!("[✓ {} HIT(S)]", results.len()).green().bold()
                );
            } else {
                println!(
                    "│  {} {:<32} {}",
                    "[SOCIAL]".bright_magenta().bold(),
                    "Reddit Discussion Search",
                    "[— 0 HITS]".bright_black()
                );
            }
            results
        }
        Err(e) => {
            println!(
                "│  {} {:<32} {}",
                "[SOCIAL]".bright_magenta().bold(),
                "Reddit Discussion Search",
                format!("[✗ ERROR: {}]", e).red()
            );
            vec![]
        }
    };

    let github = match github_res {
        Ok(results) => {
            if !results.is_empty() {
                println!(
                    "│  {} {:<32} {}",
                    "[CODE]".bright_green().bold(),
                    "GitHub Code Repository Search",
                    format!("[✓ {} LEAK(S)]", results.len()).green().bold()
                );
            } else {
                println!(
                    "│  {} {:<32} {}",
                    "[CODE]".bright_green().bold(),
                    "GitHub Code Repository Search",
                    "[— 0 LEAKS / NO KEY]".bright_black()
                );
            }
            results
        }
        Err(e) => {
            println!(
                "│  {} {:<32} {}",
                "[CODE]".bright_green().bold(),
                "GitHub Code Repository Search",
                format!("[✗ ERROR: {}]", e).red()
            );
            vec![]
        }
    };

    if !dorks_res.is_empty() {
        println!(
            "│  {} {:<32} {}",
            "[DORKS]".bright_purple().bold(),
            "Paste & Document Dorking",
            format!("[✓ {} FINDING(S)]", dorks_res.len()).green().bold()
        );
    } else {
        println!(
            "│  {} {:<32} {}",
            "[DORKS]".bright_purple().bold(),
            "Paste & Document Dorking",
            "[— 0 DORK MATCHES]".bright_black()
        );
    }

    println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────╯".bright_cyan());

    OsintResults {
        phone_info: None,
        numverify,
        numlookup,
        hudsonrock,
        threat_reputation: None,
        dorks: dorks_res,
        google,
        duckduckgo,
        brave,
        reddit,
        github,
    }
}

pub async fn run_osint_scans_quiet(
    config: &AppConfig,
    phone_number: &str,
    region: &str,
) -> OsintResults {
    let client = config.build_client();

    let (
        numverify_res,
        numlookup_res,
        hudsonrock_res,
        google_res,
        duckduckgo_res,
        brave_res,
        reddit_res,
        github_res,
        dorks_res,
    ) = tokio::join!(
        numverify::check_numverify(&client, &config.numverify_key, phone_number, region),
        numlookup::check_numlookup(&client, &config.numlookup_key, phone_number),
        hudsonrock::check_hudsonrock(&client, phone_number),
        google::search_google(&client, &config.serpapi_key, phone_number),
        duckduckgo::search_duckduckgo(&client, phone_number),
        brave::search_brave(&client, &config.brave_api_key, phone_number),
        reddit::search_reddit(&client, phone_number),
        github::search_github(&client, &config.github_token, phone_number),
        dorks::run_dorks(&client, phone_number),
    );

    OsintResults {
        phone_info: None,
        numverify: numverify_res.ok().flatten(),
        numlookup: numlookup_res.ok().flatten(),
        hudsonrock: hudsonrock_res.ok().flatten(),
        threat_reputation: None,
        dorks: dorks_res,
        google: google_res.unwrap_or_default(),
        duckduckgo: duckduckgo_res.unwrap_or_default(),
        brave: brave_res.unwrap_or_default(),
        reddit: reddit_res.unwrap_or_default(),
        github: github_res.unwrap_or_default(),
    }
}
