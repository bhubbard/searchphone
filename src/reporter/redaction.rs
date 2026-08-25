use crate::models::OsintReport;

pub fn mask_phone(phone: &str) -> String {
    let clean: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if clean.len() >= 7 {
        let prefix = &clean[..clean.len().saturating_sub(4)];
        let visible_start = if prefix.len() > 3 { &prefix[..3] } else { prefix };
        format!("+{}***-{}", visible_start, &clean[clean.len().saturating_sub(2)..])
    } else {
        "***-***".to_string()
    }
}

pub fn mask_email(email: &str) -> String {
    if let Some((user, domain)) = email.split_once('@') {
        let masked_user = if user.len() <= 2 {
            "*".repeat(user.len())
        } else {
            format!("{}***{}", &user[..1], &user[user.len() - 1..])
        };
        format!("{}@{}", masked_user, domain)
    } else {
        email.to_string()
    }
}

pub fn redact_report(report: &OsintReport) -> OsintReport {
    let mut redacted = report.clone();
    redacted.metadata.phone = mask_phone(&report.metadata.phone);

    if let Some(ref mut info) = redacted.results.phone_info {
        info.international = mask_phone(&info.international);
        info.national = mask_phone(&info.national);
        info.e164 = mask_phone(&info.e164);
    }

    if let Some(ref mut hr) = redacted.results.hudsonrock {
        for s in &mut hr.stealers {
            s.top_logins = s.top_logins.iter().map(|l| mask_email(l)).collect();
        }
    }

    redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_phone() {
        let masked = mask_phone("+16502530000");
        assert!(masked.contains("***"));
    }

    #[test]
    fn test_mask_email() {
        let masked = mask_email("investigator@example.com");
        assert!(masked.starts_with("i***r@example.com"));
    }
}
