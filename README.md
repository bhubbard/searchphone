<h1 align="center">SearchPhone 🕵🏽‍♂️ (Rust Edition)</h1>

<p align="center">
SearchPhone is a high-performance, asynchronous OSINT suite for investigating telephone number intelligence, threat exposure, and entity relationships across multiple search engines, code repositories, and infostealer databases.
</p>

<p align="center">
<i>This project was originally started as a Rust port of the original <a href="https://github.com/HackUnderway/SearchPhone">SearchPhone</a> tool, rewritten from scratch for maximum concurrency, memory safety, and standalone binary deployment, and subsequently expanded with advanced OSINT dorking, STIX 2.1 feeds, interactive D3.js visualization, and batch processing.</i>
</p>

<p align="center">
<img src="assets/Demo_SearchPhone.png" title="SearchPhone" alt="SearchPhone" width="600"/>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?logo=rust&logoColor=white" alt="Rust version">
  <img src="https://img.shields.io/badge/NUMVERIFY/SERPAPI/GITHUB-API-blue?logo=rapidapi&logoColor=white">
  <img src="https://img.shields.io/badge/License-MIT-green?logo=open-source-initiative&logoColor=white" alt="License">
</p>

## 🚀 Features & Capabilities

- ⚡ **Pure Rust & High Concurrency**: Built with `tokio` (1.43), `reqwest`, and `axum` with zero Python/Node dependencies.
- 🌍 **International Phone Validation**: Google `libphonenumber` integration with automatic E.164 standardization, carrier, and timezone parsing.
- 🛡️ **Composite Threat Intel & Risk Scoring**: Calculates a 0–100 numerical risk score with severity levels (`CLEAN`, `LOW`, `MEDIUM`, `HIGH`, `CRITICAL`), analyzing infostealer infections, credential leaks, and disposable VoIP line types.
- 🔍 **Multi-Engine Search & Smart Dorking**:
  - Google Search (SerpAPI)
  - Brave Search API
  - DuckDuckGo Instant Answers
  - Automated Search Dorks (Pastebin, Ghostbin, PDF/XLSX/CSV leaked documents, social directories)
- 👥 **Social & Code Footprinting**:
  - Reddit discussion & sentiment search
  - GitHub code repository leaks & hardcoded secrets lookup
- 📊 **Multi-Format Export & Visual Graph**:
  - **Terminal UI**: Modern cybersecurity ASCII banner and Unicode box cards.
  - **JSON**: Complete structured investigation reports.
  - **PDF**: Multi-page formatted reports.
  - **Interactive HTML Graph**: Self-contained standalone D3.js force-directed relationship graph (`report.html`).
  - **STIX 2.1 Threat Feed**: Machine-readable JSON bundles for SIEM/SOC platforms (Splunk, OpenCTI, MISP).
- ⚡ **Automation & High-Throughput Batching**:
  - Bulk processing from file (`--batch <FILE>`) with worker pool concurrency and `indicatif` progress bar.
  - Unix piping & JSON streaming (`cat numbers.txt | searchphone --stream | jq .`).
- 🌐 **Embedded Web Dashboard & REST API**: Run standalone web console via `searchphone --serve --port 8080`.
- 🔒 **Investigator OPSEC & PII Redaction**:
  - Built-in proxy routing (`--proxy <URL>`) and native Tor support (`--tor`).
  - Sensitive PII masking mode (`--redact`).

---

## 🛠️ Usage

### 1. Single Target Investigation
```bash
searchphone "+16502530000"
searchphone "+16502530000" us --redact
```

### 2. Batch Processing
```bash
searchphone --batch targets.txt --concurrency 20
```

### 3. Unix Pipeline / Streaming Mode
```bash
cat targets.txt | searchphone --stream | jq '.results.threat_reputation'
```

### 4. Embedded Web Dashboard & API
```bash
searchphone --serve --port 8080
# Open http://localhost:8080 in your browser
```

### 5. OPSEC Routing via Proxy or Tor
```bash
# Route via SOCKS5 or HTTP proxy
searchphone "+16502530000" --proxy socks5://127.0.0.1:9050

# Native Tor daemon route
searchphone "+16502530000" --tor
```

## 🔑 API Keys Configuration

| Service | Purpose | Link | Plan | Key |
|---------|---------|------|------|-----|
| **Numverify** | Phone number validation & carrier info | [numverify.com](https://numverify.com/) | Free (100 requests/month) | 🔑 (Optional) |
| **NumLookup** | Reverse phone caller-ID | [numlookup.com](https://www.numlookup.com/) | Free | 🔑 (Optional) |
| **SerpAPI** | Google Search results | [serpapi.com](https://serpapi.com/) | Free (250 searches/month) | 🔑 (Optional) |
| **GitHub Token** | GitHub code search | [GitHub Settings](https://github.com/settings/tokens) | Free (5000 requests/hour) | 🔑 (Optional) |
| **Hudson Rock** | Infostealer intelligence | [cavalier.hudsonrock.com](https://cavalier.hudsonrock.com) | Free (No API key required) | ❌ No key needed |

### Setting Up `.env`:

Copy `example.env` to `.env` and insert your API keys:

```bash
# Optional API Keys - Set in .env or export in environment
NUMVERIFY_KEY="your_numverify_api_key"
NUMLOOKUP_KEY="your_numlookupapi_key"
SERPAPI_KEY="your_serpapi_key"
BRAVE_API_KEY="your_brave_search_api_key"
GITHUB_TOKEN="your_github_personal_access_token"
```

* **NumLookup** (https://www.numlookup.com): Reverse phone caller-ID and directory lookup with automated portal link generation (API key optional).
* **Hudson Rock**: Free Infostealer threat intelligence (no API key required).
* **DuckDuckGo**: Free search engine (no API key required).
* **Reddit**: Free public search endpoint (no API key required).

---

## 🚀 Installation & Building

Ensure you have Rust and Cargo installed:

```bash
git clone https://github.com/HackUnderway/SearchPhone.git
cd SearchPhone
cargo build --release
```

The compiled binary will be located at `./target/release/searchphone`.

---

## 💻 Usage

### 1. Interactive Mode
Run without arguments to be prompted interactively:
```bash
cargo run --release
# or
./target/release/searchphone
```

### 2. Command Line Arguments
Pass the phone number and country code directly (default region is `us`):
```bash
searchphone "+16502530000"
# or specify region
searchphone "+447911123456" "gb"
# or using flags
searchphone -p "+19777334049" -r "us"
```

### 3. CLI Options
```text
Usage: searchphone [OPTIONS] [POS_PHONE] [POS_REGION]

Arguments:
  [POS_PHONE]   Positional phone number
  [POS_REGION]  Positional region [default: us]

Options:
  -p, --phone <PHONE>            Phone number to search (e.g. +16502530000 or +447911123456)
  -r, --region <REGION>          Country region code (ISO 3166-1 alpha-2, e.g. 'us', 'gb', 'pe')
      --no-pdf                   Disable PDF generation
      --json-only                Only export JSON report
  -o, --output-dir <OUTPUT_DIR>  Custom output directory for generated reports
  -h, --help                     Print help
  -V, --version                  Print version
```

---

# 📦 Example Output

<p align="center">
<img src="assets/Demo_SearchPhone.png" title="SearchPhone" alt="SearchPhone" width="600"/>
</p>

<p align="center">
<img src="assets/Demo_SearchPhone_Update.png" title="SearchPhone" alt="SearchPhone" width="600"/>
</p>

---

# 🧪 Running Tests

```bash
cargo test
```

---

# 📄 License

This project is licensed under the [MIT License](LICENSE).
