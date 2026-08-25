use crate::models::OsintReport;
use serde_json::json;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

pub fn generate_stix_bundle(report: &OsintReport) -> serde_json::Value {
    let bundle_id = format!("bundle--{}", uuid_simple());
    let mut objects = Vec::new();

    // Identity Object
    let identity_id = format!("identity--{}", uuid_simple());
    objects.push(json!({
        "type": "identity",
        "spec_version": "2.1",
        "id": identity_id,
        "name": "SearchPhone OSINT Suite",
        "identity_class": "system",
        "created": report.metadata.timestamp,
        "modified": report.metadata.timestamp
    }));

    // Indicator Object (Phone Number)
    let indicator_id = format!("indicator--{}", uuid_simple());
    let phone_pattern = format!("[telephone-call:src_ref.value = '{}']", report.metadata.phone);
    objects.push(json!({
        "type": "indicator",
        "spec_version": "2.1",
        "id": indicator_id,
        "created_by_ref": identity_id,
        "name": format!("Phone Number Indicator: {}", report.metadata.phone),
        "description": "Suspect or investigated telephone number intelligence record",
        "pattern_type": "stix",
        "pattern": phone_pattern,
        "valid_from": report.metadata.timestamp,
        "created": report.metadata.timestamp,
        "modified": report.metadata.timestamp
    }));

    // Malware & Breach Objects if Hudson Rock found
    if let Some(hr) = &report.results.hudsonrock {
        for stealer in &hr.stealers {
            let malware_name = stealer.stealer_family.as_deref().unwrap_or("Unknown Malware");
            let malware_id = format!("malware--{}", uuid_simple());

            objects.push(json!({
                "type": "malware",
                "spec_version": "2.1",
                "id": malware_id,
                "created_by_ref": identity_id,
                "name": malware_name,
                "is_family": true,
                "created": report.metadata.timestamp,
                "modified": report.metadata.timestamp
            }));

            // Relationship Indicator -> Malware
            let rel_id = format!("relationship--{}", uuid_simple());
            objects.push(json!({
                "type": "relationship",
                "spec_version": "2.1",
                "id": rel_id,
                "source_ref": indicator_id,
                "target_ref": malware_id,
                "relationship_type": "indicates",
                "created": report.metadata.timestamp,
                "modified": report.metadata.timestamp
            }));
        }
    }

    json!({
        "type": "bundle",
        "id": bundle_id,
        "spec_version": "2.1",
        "objects": objects
    })
}

pub fn export_stix(report: &OsintReport, report_dir: &str, filename: &str) -> Result<String, String> {
    create_dir_all(report_dir).map_err(|e| e.to_string())?;
    let path = Path::new(report_dir).join(filename);

    let stix_bundle = generate_stix_bundle(report);
    let bytes = serde_json::to_vec_pretty(&stix_bundle).map_err(|e| e.to_string())?;

    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:016x}-{:08x}", nanos, (nanos % 0xFFFFFFFF) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OsintResults, ReportMetadata};

    #[test]
    fn test_stix_bundle_generation() {
        let report = OsintReport {
            metadata: ReportMetadata {
                phone: "+16502530000".to_string(),
                region: "US".to_string(),
                timestamp: "2026-08-25T00:00:00Z".to_string(),
                tool: "SearchPhone OSINT".to_string(),
            },
            results: OsintResults::default(),
        };

        let bundle = generate_stix_bundle(&report);
        assert_eq!(bundle["type"], "bundle");
        assert!(bundle["objects"].as_array().unwrap().len() >= 2);
    }
}
