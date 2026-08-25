use crate::models::OsintReport;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

pub fn export_json(
    report: &OsintReport,
    report_dir: &str,
    filename: &str,
) -> Result<String, String> {
    create_dir_all(report_dir).map_err(|e| e.to_string())?;
    let path = Path::new(report_dir).join(filename);

    let json_bytes = serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?;

    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(&json_bytes).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OsintResults, ReportMetadata};

    #[test]
    fn test_export_json() {
        let report = OsintReport {
            metadata: ReportMetadata {
                phone: "+51999999999".to_string(),
                region: "PE".to_string(),
                timestamp: "2026-08-25T00:00:00Z".to_string(),
                tool: "SearchPhone OSINT (Rust)".to_string(),
            },
            results: OsintResults::default(),
        };

        let temp_dir = std::env::temp_dir();
        let filename = "test_searchphone.json";
        let res = export_json(&report, &temp_dir.to_string_lossy(), filename);
        assert!(res.is_ok());

        let file_path = temp_dir.join(filename);
        assert!(file_path.exists());
        let _ = std::fs::remove_file(file_path);
    }
}
