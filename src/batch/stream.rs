use crate::config::AppConfig;
use crate::models::{OsintReport, ReportMetadata};
use crate::osint;
use crate::reporter;
use crate::threat;
use crate::validator;
use chrono::Local;
use std::io::{self, BufRead};

pub async fn run_stream_scan(config: &AppConfig, default_region: &str) {
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines().map_while(Result::ok) {
        let phone = line.trim();
        if phone.is_empty() || phone.starts_with('#') {
            continue;
        }

        let phone_info = validator::validate_and_parse(phone, default_region);
        let mut results = osint::run_osint_scans_quiet(config, phone, default_region).await;
        results.phone_info = phone_info;

        let threat_rep = threat::evaluate_threat_reputation(
            phone,
            results.numverify.as_ref(),
            results.hudsonrock.as_ref(),
            &results,
        );
        results.threat_reputation = Some(threat_rep);

        let now = Local::now();
        let report = OsintReport {
            metadata: ReportMetadata {
                phone: phone.to_string(),
                region: default_region.to_string(),
                timestamp: now.to_rfc3339(),
                tool: "SearchPhone OSINT".to_string(),
            },
            results,
        };

        let report = if config.redact {
            reporter::redaction::redact_report(&report)
        } else {
            report
        };

        if let Ok(json_line) = serde_json::to_string(&report) {
            println!("{}", json_line);
        }
    }
}
