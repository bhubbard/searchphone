use crate::models::{OsintResults, PhoneInfo};
use colored::Colorize;
use std::path::Path;

pub const BANNER_LOGO: &str = r#"
  ███████╗███████╗ █████╗ ██████╗  ██████╗██╗  ██╗██████╗ ██╗  ██╗ ██████╗ ███╗   ██╗███████╗
  ██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██║  ██║██╔══██╗██║  ██║██╔═══██╗████╗  ██║██╔════╝
  ███████╗█████╗  ███████║██████╔╝██║     ███████║██████╔╝███████║██║   ██║██╔██╗ ██║█████╗  
  ╚════██║██╔══╝  ██╔══██║██╔══██╗██║     ██╔══██║██╔═══╝ ██╔══██║██║   ██║██║╚██╗██║██╔══╝  
  ███████║███████╗██║  ██║██║  ██║╚██████╗██║  ██║██║     ██║  ██║╚██████╔╝██║ ╚████║███████╗
  ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝"#;

pub fn print_banner() {
    println!("{}", BANNER_LOGO.cyan().bold());
    println!("{}", "  ─────────────────────────────────────────────────────────────────────────────────────────────".bright_black());
    println!(
        "  {}  {}  {}",
        "🕵  OSINT Phone Intelligence Suite".white().bold(),
        "│".bright_black(),
        "v1.0.0 (Rust Edition)".green()
    );
    println!("{}", "  ─────────────────────────────────────────────────────────────────────────────────────────────\n".bright_black());
}

pub fn print_target_card(phone: &str, region: &str, info: Option<&PhoneInfo>) {
    println!("{}", "╭── Target Profile ──────────────────────────────────────────────────────────────────────────╮".bright_blue());
    println!(
        "│  {:<16} {}",
        "Target Number:".bright_white().bold(),
        phone.cyan().bold()
    );
    println!(
        "│  {:<16} {}",
        "Default Region:".bright_white(),
        region.to_uppercase().yellow()
    );

    if let Some(valid_info) = info {
        println!(
            "│  {:<16} {}",
            "International:".bright_white(),
            valid_info.international.white()
        );
        println!(
            "│  {:<16} {}",
            "National:".bright_white(),
            valid_info.national.white()
        );
        println!(
            "│  {:<16} {}",
            "E.164 Format:".bright_white(),
            valid_info.e164.green()
        );
        println!(
            "│  {:<16} {}",
            "Country:".bright_white(),
            valid_info.country.bright_yellow()
        );
        println!(
            "│  {:<16} {}",
            "Carrier:".bright_white(),
            valid_info.carrier.white()
        );
        println!(
            "│  {:<16} {}",
            "Time Zone:".bright_white(),
            valid_info.timezone.join(", ").white()
        );
        println!(
            "│  {:<16} {}",
            "Validation:".bright_white(),
            "[✓ VALID TELEPHONE NUMBER]".green().bold()
        );
    } else {
        println!(
            "│  {:<16} {}",
            "Validation:".bright_white(),
            "[! UNSTANDARDIZED / NON-E164 FORMAT - PROCEEDING DEEP SEARCH]".yellow().bold()
        );
    }
    println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────────────────────╯".bright_blue());
}

pub fn display_full_report(results: &OsintResults) {
    // Numverify Card
    if let Some(nv) = &results.numverify {
        println!("{}", "╭── [CARRIER INTEL] Numverify Telecom Data ──────────────────────────────────────────────────╮".bright_cyan());
        if let Some(carrier) = &nv.carrier {
            println!("│  {:<16} {}", "Carrier:".bright_white(), carrier.white());
        }
        if let Some(line_type) = &nv.line_type {
            println!("│  {:<16} {}", "Line Type:".bright_white(), line_type.yellow());
        }
        if let Some(country) = &nv.country {
            println!("│  {:<16} {}", "Country:".bright_white(), country.white());
        }
        if let Some(loc) = &nv.location {
            if !loc.is_empty() {
                println!("│  {:<16} {}", "Location:".bright_white(), loc.white());
            }
        }
        println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────────────────────╯".bright_cyan());
    }

    // NumLookup Card
    if let Some(nl) = &results.numlookup {
        println!("{}", "╭── [REVERSE LOOKUP] NumLookup Directory Intelligence ──────────────────────────────────────╮".bright_cyan());
        if let Some(carrier) = &nl.carrier {
            println!("│  {:<16} {}", "Carrier:".bright_white(), carrier.white());
        }
        if let Some(line_type) = &nl.line_type {
            println!("│  {:<16} {}", "Line Type:".bright_white(), line_type.yellow());
        }
        if let Some(loc) = &nl.location {
            println!("│  {:<16} {}", "Location:".bright_white(), loc.white());
        }
        println!("│  {:<16} {}", "Portal Link:".bright_white(), nl.web_url.bright_blue().underline());
        println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────────────────────╯".bright_cyan());
    }

    // Hudson Rock Card
    if let Some(hr) = &results.hudsonrock {
        if hr.found {
            println!("{}", "╭── [THREAT INTEL] Hudson Rock Infostealer Intelligence ─────────────────────────────────────╮".red());
            println!(
                "│  {:<22} {}",
                "Threat Status:".bright_white().bold(),
                "⚠️ COMPROMISED (Malware Infection Detected)".red().bold()
            );
            println!(
                "│  {:<22} {}",
                "Corporate Services:".bright_white(),
                hr.total_corporate_services.to_string().yellow()
            );
            println!(
                "│  {:<22} {}",
                "Personal Services:".bright_white(),
                hr.total_user_services.to_string().yellow()
            );

            for (i, stealer) in hr.stealers.iter().take(3).enumerate() {
                let family = stealer.stealer_family.as_deref().unwrap_or("Unknown");
                let date = stealer
                    .date_compromised
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(10)
                    .collect::<String>();
                let computer = stealer.computer_name.as_deref().unwrap_or("Unknown");
                let os = stealer.operating_system.as_deref().unwrap_or("Unknown");

                println!("│");
                println!(
                    "│  {} {} {}",
                    "Infection".red().bold(),
                    format!("#{}:", i + 1).red().bold(),
                    family.bright_red().bold()
                );
                println!("│    • {:<16} {}", "Compromised:".bright_white(), date.white());
                println!("│    • {:<16} {}", "Host Machine:".bright_white(), computer.white());
                println!("│    • {:<16} {}", "OS Build:".bright_white(), os.white());

                if !stealer.top_logins.is_empty() {
                    let logins = stealer
                        .top_logins
                        .iter()
                        .take(4)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("│    • {:<16} {}", "Exposed Logins:".bright_white(), logins.bright_yellow());
                }
            }
            println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────────────────────╯".red());
        }
    }

    // Dorks Findings Card
    if !results.dorks.is_empty() {
        println!("{}", "╭── [DORK INTEL] Pastebin & Document Findings ───────────────────────────────────────────────╮".bright_purple());
        for dork in &results.dorks {
            println!("│  {} {}", "• Category:".bright_white(), dork.category.cyan().bold());
            for r in &dork.results {
                println!("│    {} {}", "Link:".bright_white(), r.link.bright_blue().underline());
                if !r.snippet.is_empty() {
                    println!("│    {} {}...", "Text:".bright_white(), r.snippet.bright_black());
                }
            }
        }
        println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────────────────────╯".bright_purple());
    }

    // Google Search Card
    if !results.google.is_empty() {
        println!("{}", "╭── [SEARCH INTEL] Google Results (SerpAPI) ─────────────────────────────────────────────────╮".bright_green());
        for (i, item) in results.google.iter().take(5).enumerate() {
            let title: String = item.title.chars().take(80).collect();
            let link: String = item.link.chars().take(90).collect();
            let snippet: String = item.snippet.chars().take(130).collect();

            println!("│  {}. {}", (i + 1).to_string().cyan().bold(), title.white().bold());
            if !link.is_empty() {
                println!("│     {} {}", "🔗".bright_black(), link.bright_blue().underline());
            }
            if !snippet.is_empty() {
                println!("│     {} {}...", "📝".bright_black(), snippet.bright_black());
            }
            if i + 1 < results.google.len().min(5) {
                println!("│");
            }
        }
        println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────────────────────╯".bright_green());
    }

    // Reddit Card
    if !results.reddit.is_empty() {
        println!("{}", "╭── [SOCIAL INTEL] Reddit Mentions & Discussions ────────────────────────────────────────────╮".bright_yellow());
        for (i, post) in results.reddit.iter().take(3).enumerate() {
            let title: String = post.title.chars().take(75).collect();
            println!("│  {}. {}", (i + 1).to_string().cyan().bold(), title.white().bold());
            if !post.url.is_empty() {
                println!("│     {} {}", "🔗".bright_black(), post.url.bright_blue().underline());
            }
            if !post.subreddit.is_empty() {
                println!(
                    "│     {} {}  │  Score: {}",
                    "👥".bright_black(),
                    format!("r/{}", post.subreddit).bright_magenta(),
                    post.score.to_string().bright_yellow()
                );
            }
            if i + 1 < results.reddit.len().min(3) {
                println!("│");
            }
        }
        println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────────────────────╯".bright_yellow());
    }

    // GitHub Card
    if !results.github.is_empty() {
        println!("{}", "╭── [CODE RECON] GitHub Repository Leaks ────────────────────────────────────────────────────╮".bright_magenta());
        for (i, item) in results.github.iter().take(4).enumerate() {
            let mut display_name = item.repository.clone();
            if !item.path.is_empty() {
                display_name = format!("{} -> {}", display_name, item.path);
            }
            let display_name: String = display_name.chars().take(80).collect();

            println!("│  {}. {}", (i + 1).to_string().cyan().bold(), display_name.white().bold());
            if !item.url.is_empty() {
                println!("│     {} {}", "🔗".bright_black(), item.url.bright_blue().underline());
            }
            if !item.language.is_empty() {
                println!("│     {} Language: {}", "💻".bright_black(), item.language.green());
            }
            if i + 1 < results.github.len().min(4) {
                println!("│");
            }
        }
        println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────────────────────╯".bright_magenta());
    }
}

pub fn display_summary_card(
    results: &OsintResults,
    json_path: Option<&str>,
    pdf_path: Option<&str>,
    html_path: Option<&str>,
    stix_path: Option<&str>,
) {
    println!("{}", "╭── Executive Summary & Exported Reports ────────────────────────────────────────────────────╮".bright_cyan());

    let mut total_found = 0;
    let services = [
        ("Google Search", results.google.len()),
        ("Brave Search", results.brave.len()),
        ("Reddit Mentions", results.reddit.len()),
        ("GitHub Repos", results.github.len()),
        ("DuckDuckGo", results.duckduckgo.len()),
        ("Dork Findings", results.dorks.len()),
    ];

    for (name, count) in services {
        if count > 0 {
            println!("│  • {:<20} {} hit(s)", name.bright_white(), count.to_string().green().bold());
            total_found += count;
        }
    }

    if let Some(hr) = &results.hudsonrock {
        if hr.found {
            let infect_count = hr.stealers.len();
            println!("│  • {:<20} {} infection(s)", "Hudson Rock".bright_white(), infect_count.to_string().red().bold());
            total_found += infect_count;
        }
    }

    println!("│");
    println!("│  {:<22} {}", "Total Findings:".bright_white().bold(), total_found.to_string().cyan().bold());

    if let Some(ref rep) = results.threat_reputation {
        let level_display = match rep.risk_level.as_str() {
            "CRITICAL" => rep.risk_level.red().bold(),
            "HIGH" => rep.risk_level.red().bold(),
            "MEDIUM" => rep.risk_level.yellow().bold(),
            "LOW" => rep.risk_level.blue().bold(),
            _ => rep.risk_level.green().bold(),
        };
        println!(
            "│  {:<22} {} (Score: {}/100)",
            "Threat Assessment:".bright_white().bold(),
            level_display,
            rep.risk_score
        );
        for f in &rep.threat_factors {
            println!("│    {} {}", "• Factor:".bright_black(), f.yellow());
        }
    }

    println!("│");
    println!("│  {}", "Generated Artifacts:".bright_white().bold());
    if let Some(json_file) = json_path {
        let abs_path = Path::new(json_file).canonicalize().unwrap_or_else(|_| Path::new(json_file).to_path_buf());
        println!("│    {} {:<6} {}", "[JSON]".green().bold(), "Report:", abs_path.to_string_lossy().bright_white());
    }
    if let Some(pdf_file) = pdf_path {
        let abs_path = Path::new(pdf_file).canonicalize().unwrap_or_else(|_| Path::new(pdf_file).to_path_buf());
        println!("│    {} {:<6} {}", "[PDF]".bright_blue().bold(), "Report:", abs_path.to_string_lossy().bright_white());
    }
    if let Some(html_file) = html_path {
        let abs_path = Path::new(html_file).canonicalize().unwrap_or_else(|_| Path::new(html_file).to_path_buf());
        println!("│    {} {:<6} {}", "[HTML]".cyan().bold(), "Graph:", abs_path.to_string_lossy().bright_white());
    }
    if let Some(stix_file) = stix_path {
        let abs_path = Path::new(stix_file).canonicalize().unwrap_or_else(|_| Path::new(stix_file).to_path_buf());
        println!("│    {} {:<6} {}", "[STIX]".magenta().bold(), "Threat:", abs_path.to_string_lossy().bright_white());
    }

    println!("{}\n", "╰────────────────────────────────────────────────────────────────────────────────────────────╯".bright_cyan());
}
