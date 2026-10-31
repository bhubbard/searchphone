mod batch;
mod config;
mod dorks;
mod legal_ooh;
mod models;
mod osint;
mod reporter;
mod threat;
mod validator;
mod web;

use chrono::Local;
use clap::Parser;
use colored::Colorize;
use config::AppConfig;
use models::{OsintReport, ReportMetadata};
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(
    name = "searchphone",
    version = "1.0.0",
    about = "High-performance OSINT telephone number intelligence gathering and reporting in Rust"
)]
struct Cli {
    /// Phone number to search (e.g. +16502530000 or +447911123456)
    #[arg(short, long)]
    phone: Option<String>,

    /// Country region code (ISO 3166-1 alpha-2, e.g. 'us', 'gb', 'pe')
    #[arg(short, long)]
    region: Option<String>,

    /// Positional phone number
    #[arg(index = 1)]
    pos_phone: Option<String>,

    /// Positional region
    #[arg(index = 2)]
    pos_region: Option<String>,

    /// Batch file path containing target phone numbers
    #[arg(short, long)]
    batch: Option<String>,

    /// Batch concurrency worker count
    #[arg(long, default_value_t = 10)]
    concurrency: usize,

    /// Read phone numbers line-by-line from stdin and stream JSON reports to stdout
    #[arg(long)]
    stream: bool,

    /// Start embedded Web Dashboard & REST API server
    #[arg(long)]
    serve: bool,

    /// Port for embedded web server
    #[arg(long, default_value_t = 8080)]
    port: u16,

    /// HTTP/HTTPS or SOCKS5 proxy URL (e.g. socks5://127.0.0.1:9050)
    #[arg(long)]
    proxy: Option<String>,

    /// Route outbound traffic through local Tor daemon (socks5://127.0.0.1:9050)
    #[arg(long)]
    tor: bool,

    /// Mask/redact sensitive PII in reports and terminal output
    #[arg(long)]
    redact: bool,

    /// Generate standalone interactive HTML/D3 relationship graph report
    #[arg(long, default_value_t = true)]
    html: bool,

    /// Generate STIX 2.1 Threat Intel JSON export
    #[arg(long, default_value_t = true)]
    stix: bool,

    /// Disable PDF generation
    #[arg(long)]
    no_pdf: bool,

    /// Only export JSON report
    #[arg(long)]
    json_only: bool,

    /// Custom output directory for generated reports
    #[arg(short, long)]
    output_dir: Option<String>,
}

fn prompt_input(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input.trim().to_string()
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut config = AppConfig::load();

    if let Some(out_dir) = cli.output_dir {
        config.report_dir = out_dir;
    }

    if cli.tor {
        config.proxy_url = Some("socks5://127.0.0.1:9050".to_string());
    } else if let Some(ref p) = cli.proxy {
        config.proxy_url = Some(p.clone());
    }

    if cli.redact {
        config.redact = true;
    }

    // 1. Embedded Web Server Mode
    if cli.serve {
        if let Err(e) = web::run_web_server(config, cli.port).await {
            eprintln!("{}: {}", "❌ Server Error".red().bold(), e);
        }
        return;
    }

    // 2. Stdin / Stdout Unix Streaming Mode
    if cli.stream {
        let region = cli.region.or(cli.pos_region).unwrap_or_else(|| "us".to_string());
        batch::run_stream_scan(&config, &region).await;
        return;
    }

    // 3. Batch File Scan Mode
    if let Some(ref batch_file) = cli.batch {
        let region = cli.region.or(cli.pos_region).unwrap_or_else(|| "us".to_string());
        if let Err(e) = batch::run_batch_scan(&config, batch_file, &region, cli.concurrency).await {
            eprintln!("{}: {}", "❌ Batch Error".red().bold(), e);
        }
        return;
    }

    // 4. Interactive or Single Target Scan Mode
    reporter::terminal::print_banner();

    let provided_phone = cli.phone.or(cli.pos_phone);
    let provided_region = cli.region.or(cli.pos_region);

    let (phone_number, region) = match provided_phone {
        Some(ref p) if !p.trim().is_empty() => {
            let reg = provided_region.unwrap_or_else(|| "us".to_string());
            (p.trim().to_string(), reg)
        }
        _ => {
            let prompt = format!("  {} ", "📱 Enter phone number to investigate:".bright_green().bold());
            let input = prompt_input(&prompt);
            if input.is_empty() {
                eprintln!("\n  {}", "❌ Error: Phone number cannot be empty.".red().bold());
                return;
            }
            let reg_prompt = format!("  {} ", "🌍 Enter default region code [default: us]:".bright_green().bold());
            let reg_input = prompt_input(&reg_prompt);
            let reg = if reg_input.is_empty() {
                "us".to_string()
            } else {
                reg_input
            };
            println!();
            (input, reg)
        }
    };

    // Validate and parse phone metadata
    let phone_info = validator::validate_and_parse(&phone_number, &region);

    // Print Target Profile Card
    reporter::terminal::print_target_card(&phone_number, &region, phone_info.as_ref());

    // Run parallel OSINT scans
    let mut results = osint::run_osint_scans(&config, &phone_number, &region).await;
    results.phone_info = phone_info;

    // Evaluate Threat Reputation & Risk Score
    let threat_rep = threat::evaluate_threat_reputation(
        &phone_number,
        results.numverify.as_ref(),
        results.hudsonrock.as_ref(),
        &results,
    );
    results.threat_reputation = Some(threat_rep);

    // Display formatted results in terminal
    reporter::terminal::display_full_report(&results);

    // Prepare report metadata and filenames
    let now = Local::now();
    let timestamp_str = now.format("%Y%m%d_%H%M%S").to_string();
    let clean_number = phone_number.replace(['+', ' '], "");

    let json_filename = format!("phone_{clean_number}_{timestamp_str}.json");
    let pdf_filename = format!("phone_{clean_number}_{timestamp_str}.pdf");
    let html_filename = format!("phone_{clean_number}_{timestamp_str}.html");
    let stix_filename = format!("phone_{clean_number}_{timestamp_str}.stix.json");

    let report = OsintReport {
        metadata: ReportMetadata {
            phone: phone_number.clone(),
            region: region.clone(),
            timestamp: now.to_rfc3339(),
            tool: "SearchPhone OSINT (Rust)".to_string(),
        },
        results: results.clone(),
    };

    let report = if config.redact {
        reporter::redaction::redact_report(&report)
    } else {
        report
    };

    // Export JSON
    let mut exported_json_path = None;
    match reporter::json::export_json(&report, &config.report_dir, &json_filename) {
        Ok(path) => {
            exported_json_path = Some(path);
        }
        Err(e) => {
            eprintln!("  {} {}", "❌ Error exporting JSON report:".red(), e);
        }
    }

    // Export PDF (unless disabled)
    let mut exported_pdf_path = None;
    if !cli.no_pdf && !cli.json_only {
        match reporter::pdf::export_pdf(&report, &config.report_dir, &pdf_filename) {
            Ok(path) => {
                exported_pdf_path = Some(path);
            }
            Err(e) => {
                eprintln!("  {} {}", "❌ Error exporting PDF report:".red(), e);
            }
        }
    }

    // Export HTML (D3 Graph Visualizer)
    let mut exported_html_path = None;
    if cli.html && !cli.json_only {
        match reporter::html::export_html(&report, &config.report_dir, &html_filename) {
            Ok(path) => {
                exported_html_path = Some(path);
            }
            Err(e) => {
                eprintln!("  {} {}", "❌ Error exporting HTML graph:".red(), e);
            }
        }
    }

    // Export STIX 2.1 Threat Intel
    let mut exported_stix_path = None;
    if cli.stix && !cli.json_only {
        match reporter::stix::export_stix(&report, &config.report_dir, &stix_filename) {
            Ok(path) => {
                exported_stix_path = Some(path);
            }
            Err(e) => {
                eprintln!("  {} {}", "❌ Error exporting STIX bundle:".red(), e);
            }
        }
    }

    // Display Executive Summary & Output Card
    reporter::terminal::display_summary_card(
        &results,
        exported_json_path.as_deref(),
        exported_pdf_path.as_deref(),
        exported_html_path.as_deref(),
        exported_stix_path.as_deref(),
    );
}
