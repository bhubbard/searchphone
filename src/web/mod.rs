use crate::config::AppConfig;
use crate::models::{OsintReport, ReportMetadata};
use crate::osint;
use crate::reporter;
use crate::threat;
use crate::validator;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::Local;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct WebState {
    pub config: AppConfig,
}

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub phone: String,
    pub region: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

pub async fn run_web_server(config: AppConfig, port: u16) -> Result<(), String> {
    let state = Arc::new(WebState { config });

    let app = Router::new()
        .route("/", get(dashboard_handler))
        .route("/api/health", get(health_handler))
        .route("/api/scan", post(scan_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!(
        "\n{} {}",
        "🌐 SearchPhone Web Dashboard & API live at:".bright_green().bold(),
        format!("http://localhost:{}", port).bright_cyan().underline()
    );
    println!("  • Press {} to stop\n", "Ctrl+C".yellow());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind port {}: {}", port, e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "1.0.0".to_string(),
    })
}

async fn scan_handler(
    State(state): State<Arc<WebState>>,
    Json(payload): Json<ScanRequest>,
) -> Result<Json<OsintReport>, (StatusCode, String)> {
    let phone = payload.phone.trim();
    if phone.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Phone cannot be empty".to_string()));
    }

    let region = payload.region.unwrap_or_else(|| "us".to_string());
    let phone_info = validator::validate_and_parse(phone, &region);

    let mut results = osint::run_osint_scans_quiet(&state.config, phone, &region).await;
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
            region,
            timestamp: now.to_rfc3339(),
            tool: "SearchPhone OSINT Web API".to_string(),
        },
        results,
    };

    let final_report = if state.config.redact {
        reporter::redaction::redact_report(&report)
    } else {
        report
    };

    Ok(Json(final_report))
}

async fn dashboard_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>SearchPhone - Web Intelligence Dashboard</title>
  <script src="https://d3js.org/d3.v7.min.js"></script>
  <style>
    :root {
      --bg: #0b0f19;
      --card-bg: #131b2e;
      --card-border: #1e293b;
      --text: #f1f5f9;
      --text-muted: #94a3b8;
      --accent: #38bdf8;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      padding: 30px;
    }
    .container { max-width: 1100px; margin: 0 auto; }
    header { margin-bottom: 24px; }
    h1 { color: var(--accent); font-size: 26px; font-weight: 800; }
    .search-box {
      display: flex;
      gap: 12px;
      margin-bottom: 24px;
    }
    input {
      flex: 1;
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      color: #fff;
      padding: 12px 16px;
      border-radius: 8px;
      font-size: 16px;
      outline: none;
    }
    input:focus { border-color: var(--accent); }
    button {
      background: #0284c7;
      color: #fff;
      border: none;
      padding: 12px 24px;
      font-weight: 700;
      border-radius: 8px;
      cursor: pointer;
      font-size: 15px;
    }
    button:hover { background: #0369a1; }
    .card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 20px;
      margin-bottom: 20px;
    }
    pre {
      background: #070a12;
      padding: 16px;
      border-radius: 8px;
      overflow-x: auto;
      font-size: 13px;
      color: #38bdf8;
    }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>🕵 SearchPhone OSINT Web Console</h1>
      <p style="color: var(--text-muted); font-size: 14px; margin-top: 4px;">Standalone High-Throughput REST API & Intelligence Scanner</p>
    </header>

    <div class="search-box">
      <input type="text" id="phoneInput" placeholder="Enter target telephone number (e.g. +19777334049)..." />
      <input type="text" id="regionInput" placeholder="Region (default: us)" style="max-width: 150px;" />
      <button onclick="runScan()">Investigate</button>
    </div>

    <div id="status" style="margin-bottom: 16px; font-weight: 600; color: #f59e0b;"></div>

    <div class="card" id="resultCard" style="display: none;">
      <h3 style="margin-bottom: 12px; font-size: 16px;">Scan Results</h3>
      <pre><code id="jsonOutput"></code></pre>
    </div>
  </div>

  <script>
    async function runScan() {
      const phone = document.getElementById('phoneInput').value.trim();
      const region = document.getElementById('regionInput').value.trim() || 'us';
      const statusDiv = document.getElementById('status');
      const resultCard = document.getElementById('resultCard');
      const jsonOutput = document.getElementById('jsonOutput');

      if (!phone) {
        statusDiv.innerText = "Please enter a phone number.";
        return;
      }

      statusDiv.innerText = "Scanning APIs, Threat Databases, and Code Repositories...";
      resultCard.style.display = "none";

      try {
        const resp = await fetch('/api/scan', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ phone, region })
        });
        const data = await resp.json();
        statusDiv.innerText = "Scan Complete!";
        jsonOutput.innerText = JSON.stringify(data, null, 2);
        resultCard.style.display = "block";
      } catch (err) {
        statusDiv.innerText = "Error querying API: " + err;
      }
    }
  </script>
</body>
</html>"#)
}
