use crate::config::AppConfig;
use crate::models::{OsintReport, ReportMetadata};
use crate::osint;
use crate::reporter;
use crate::threat;
use crate::validator;
use chrono::Local;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub async fn run_batch_scan(
    config: &AppConfig,
    file_path: &str,
    default_region: &str,
    concurrency: usize,
) -> Result<(), String> {
    let file = File::open(file_path).map_err(|e| format!("Cannot open batch file: {}", e))?;
    let reader = BufReader::new(file);

    let numbers: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    if numbers.is_empty() {
        println!("{}", "⚠️ No valid phone numbers found in batch file.".yellow());
        return Ok(());
    }

    println!(
        "\n{} {} {}",
        "🚀 Starting batch OSINT scan for".bright_cyan().bold(),
        numbers.len().to_string().green().bold(),
        "numbers...".bright_cyan().bold()
    );

    let pb = ProgressBar::new(numbers.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .map_err(|e| e.to_string())?
            .progress_chars("#>-"),
    );

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut tasks = Vec::new();

    for phone in numbers {
        let sem = semaphore.clone();
        let cfg = config.clone();
        let reg = default_region.to_string();
        let pb_clone = pb.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            pb_clone.set_message(format!("Scanning {}", phone));

            let phone_info = validator::validate_and_parse(&phone, &reg);
            let mut results = osint::run_osint_scans_quiet(&cfg, &phone, &reg).await;
            results.phone_info = phone_info;

            let threat_rep = threat::evaluate_threat_reputation(
                &phone,
                results.numverify.as_ref(),
                results.hudsonrock.as_ref(),
                &results,
            );
            results.threat_reputation = Some(threat_rep);

            let now = Local::now();
            let timestamp_str = now.format("%Y%m%d_%H%M%S").to_string();
            let clean_number = phone.replace(['+', ' '], "");

            let report = OsintReport {
                metadata: ReportMetadata {
                    phone: phone.clone(),
                    region: reg,
                    timestamp: now.to_rfc3339(),
                    tool: "SearchPhone OSINT".to_string(),
                },
                results,
            };

            let report = if cfg.redact {
                reporter::redaction::redact_report(&report)
            } else {
                report
            };

            let json_file = format!("phone_{}_{}.json", clean_number, timestamp_str);
            let _ = reporter::json::export_json(&report, &cfg.report_dir, &json_file);

            pb_clone.inc(1);
        }));
    }

    for task in tasks {
        let _ = task.await;
    }

    pb.finish_with_message("Batch processing complete!");
    println!(
        "\n{} {}",
        "✅ All batch reports exported to:".green().bold(),
        Path::new(&config.report_dir).display()
    );

    Ok(())
}
