use crate::models::{HudsonRockResult, NumverifyResult, OsintResults, ThreatReputation};

pub fn evaluate_threat_reputation(
    _phone: &str,
    numverify: Option<&NumverifyResult>,
    hudsonrock: Option<&HudsonRockResult>,
    results: &OsintResults,
) -> ThreatReputation {
    let mut risk_score: u32 = 0;
    let mut threat_factors = Vec::new();
    let mut breach_hits = Vec::new();
    let mut is_voip = false;
    let mut is_disposable = false;

    // Check Hudson Rock Infostealers
    if let Some(hr) = hudsonrock {
        if hr.found && !hr.stealers.is_empty() {
            let count = hr.stealers.len() as u32;
            let stealer_score = (count * 35).min(75);
            risk_score += stealer_score;
            threat_factors.push(format!(
                "{} active infostealer malware infection(s) detected",
                count
            ));

            for s in &hr.stealers {
                let family = s.stealer_family.as_deref().unwrap_or("Unknown");
                breach_hits.push(format!("Malware Family: {}", family));
                if !s.top_logins.is_empty() {
                    threat_factors.push(format!(
                        "Exposed credentials found in {} logs",
                        family
                    ));
                }
            }
        }
    }

    // Check Carrier Line Type (VoIP / Disposable)
    if let Some(nv) = numverify {
        if let Some(ref ltype) = nv.line_type {
            let lower = ltype.to_lowercase();
            if lower.contains("voip") || lower.contains("virtual") {
                is_voip = true;
                risk_score += 15;
                threat_factors.push("Virtual / VoIP non-fixed line type detected".to_string());
            } else if lower.contains("prepaid") || lower.contains("disposable") {
                is_disposable = true;
                risk_score += 10;
                threat_factors.push("Prepaid or disposable SIM carrier detected".to_string());
            }
        }
    }

    // Check GitHub Code Leaks
    if !results.github.is_empty() {
        risk_score += 15;
        threat_factors.push(format!(
            "Hardcoded telephone number identified in {} public GitHub repositories",
            results.github.len()
        ));
    }

    // Check Dork Findings
    if !results.dorks.is_empty() {
        risk_score += 10;
        threat_factors.push("Associated with public paste or document dorks".to_string());
    }

    let final_score = (risk_score.min(100)) as u8;
    let risk_level = match final_score {
        0..=15 => "CLEAN",
        16..=39 => "LOW",
        40..=69 => "MEDIUM",
        70..=89 => "HIGH",
        _ => "CRITICAL",
    }
    .to_string();

    let spam_count = if final_score > 50 { 1 } else { 0 };

    ThreatReputation {
        risk_score: final_score,
        risk_level,
        spam_reports_count: spam_count,
        is_voip,
        is_disposable,
        breach_hits,
        threat_factors,
    }
}

impl ThreatReputation {
    /// Returns the deterministic SOC triage recommendation for this risk score.
    pub fn recommended_action(&self) -> &'static str {
        match self.risk_score {
            0..=15 => "ALLOW_STANDARD",
            16..=39 => "STEP_UP_SMS_VERIFY",
            40..=69 => "REQUIRE_HARDWARE_MFA",
            70..=89 => "QUARANTINE_AND_ROTATE_CREDENTIALS",
            _ => "BLOCK_IMMEDIATELY_AND_ISSUE_STIX_IOC",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StealerInfo;

    #[test]
    fn test_clean_evaluation() {
        let results = OsintResults::default();
        let rep = evaluate_threat_reputation("+16502530000", None, None, &results);
        assert_eq!(rep.risk_score, 0);
        assert_eq!(rep.risk_level, "CLEAN");
    }

    #[test]
    fn test_compromised_evaluation() {
        let results = OsintResults::default();
        let hr = HudsonRockResult {
            found: true,
            message: "Infected".to_string(),
            stealers: vec![StealerInfo {
                stealer_family: Some("Lumma".to_string()),
                date_compromised: Some("2025-01-01".to_string()),
                computer_name: Some("DESKTOP".to_string()),
                operating_system: Some("Windows".to_string()),
                top_logins: vec!["user@test.com".to_string()],
            }],
            total_corporate_services: 0,
            total_user_services: 10,
        };

        let rep = evaluate_threat_reputation("+19777334049", None, Some(&hr), &results);
        assert!(rep.risk_score >= 35);
        assert_eq!(rep.risk_level, "LOW"); // or MEDIUM depending on points
        assert_eq!(rep.recommended_action(), "STEP_UP_SMS_VERIFY");
    }
}
