use crate::models::OsintReport;
use printpdf::*;
use std::fs::{create_dir_all, File};
use std::io::BufWriter;
use std::path::Path;

pub fn clean_text(input: &str) -> String {
    let mut s = input
        .replace('→', "->")
        .replace('←', "<-")
        .replace(['•', '★'], "*")
        .replace('✓', "V")
        .replace('✗', "X")
        .replace("⚠️", "!")
        .replace("✅", "[OK]")
        .replace("❌", "[X]")
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace('ú', "u")
        .replace('ñ', "n")
        .replace('Á', "A")
        .replace('É', "E")
        .replace('Í', "I")
        .replace('Ó', "O")
        .replace('Ú', "U")
        .replace('Ñ', "N");

    s.retain(|c| c.is_ascii() && !c.is_control());
    s
}

struct PdfCursor<'a> {
    doc: &'a PdfDocumentReference,
    current_layer: PdfLayerReference,
    font_bold: IndirectFontRef,
    font_regular: IndirectFontRef,
    font_oblique: IndirectFontRef,
    y: f32, // in mm
}

impl<'a> PdfCursor<'a> {
    fn new(
        doc: &'a PdfDocumentReference,
        page: PdfPageIndex,
        layer: PdfLayerIndex,
        font_bold: IndirectFontRef,
        font_regular: IndirectFontRef,
        font_oblique: IndirectFontRef,
    ) -> Self {
        let current_layer = doc.get_page(page).get_layer(layer);
        Self {
            doc,
            current_layer,
            font_bold,
            font_regular,
            font_oblique,
            y: 280.0,
        }
    }

    fn check_page_break(&mut self, needed_mm: f32) {
        if self.y - needed_mm < 20.0 {
            let (new_page, new_layer) = self.doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            self.current_layer = self.doc.get_page(new_page).get_layer(new_layer);
            self.y = 280.0;
        }
    }

    fn write_line(&mut self, text: &str, font_type: &str, font_size: f32, line_height: f32) {
        self.check_page_break(line_height);
        let font = match font_type {
            "bold" => &self.font_bold,
            "oblique" => &self.font_oblique,
            _ => &self.font_regular,
        };

        let cleaned = clean_text(text);
        self.current_layer.use_text(cleaned, font_size, Mm(15.0), Mm(self.y), font);
        self.y -= line_height;
    }

    fn write_horizontal_rule(&mut self) {
        self.write_line(
            "____________________________________________________________________________",
            "regular",
            10.0,
            8.0,
        );
    }
}

pub fn export_pdf(
    report: &OsintReport,
    report_dir: &str,
    filename: &str,
) -> Result<String, String> {
    create_dir_all(report_dir).map_err(|e| e.to_string())?;
    let path = Path::new(report_dir).join(filename);

    let (doc, page1, layer1) =
        PdfDocument::new("SearchPhone OSINT - Report", Mm(210.0), Mm(297.0), "Layer 1");

    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| e.to_string())?;
    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;
    let font_oblique = doc
        .add_builtin_font(BuiltinFont::HelveticaOblique)
        .map_err(|e| e.to_string())?;

    let mut cursor = PdfCursor::new(&doc, page1, layer1, font_bold, font_regular, font_oblique);

    // Title
    cursor.write_line("SearchPhone OSINT - Report", "bold", 16.0, 10.0);
    cursor.write_line(&format!("Phone: {}", report.metadata.phone), "regular", 10.0, 6.0);
    cursor.write_line(
        &format!("Region: {}", report.metadata.region.to_uppercase()),
        "regular",
        10.0,
        6.0,
    );
    cursor.write_line(
        &format!("Date: {}", report.metadata.timestamp),
        "regular",
        10.0,
        6.0,
    );
    cursor.write_horizontal_rule();

    // Basic phone info
    cursor.write_line("BASIC INFORMATION", "bold", 12.0, 8.0);
    if let Some(info) = &report.results.phone_info {
        cursor.write_line(
            &format!("  International: {}", info.international),
            "regular",
            10.0,
            6.0,
        );
        cursor.write_line(
            &format!("  Country: {}", info.country),
            "regular",
            10.0,
            6.0,
        );
        cursor.write_line(
            &format!("  Carrier: {}", info.carrier),
            "regular",
            10.0,
            6.0,
        );
        cursor.write_line(
            &format!("  Time Zone: {}", info.timezone.join(", ")),
            "regular",
            10.0,
            6.0,
        );
    } else {
        cursor.write_line("  Phone number not valid per international standards", "oblique", 10.0, 6.0);
    }
    cursor.y -= 4.0;

    // Numverify
    if let Some(nv) = &report.results.numverify {
        cursor.write_line("NUMVERIFY", "bold", 12.0, 8.0);
        if let Some(carrier) = &nv.carrier {
            cursor.write_line(&format!("  Carrier: {carrier}"), "regular", 10.0, 6.0);
        }
        if let Some(line_type) = &nv.line_type {
            cursor.write_line(&format!("  Type: {line_type}"), "regular", 10.0, 6.0);
        }
        if let Some(country) = &nv.country {
            cursor.write_line(&format!("  Country: {country}"), "regular", 10.0, 6.0);
        }
        cursor.y -= 4.0;
    }

    // NumLookup
    if let Some(nl) = &report.results.numlookup {
        cursor.write_line("NUMLOOKUP", "bold", 12.0, 8.0);
        if let Some(carrier) = &nl.carrier {
            cursor.write_line(&format!("  Carrier: {carrier}"), "regular", 10.0, 6.0);
        }
        if let Some(line_type) = &nl.line_type {
            cursor.write_line(&format!("  Type: {line_type}"), "regular", 10.0, 6.0);
        }
        if let Some(country) = &nl.country_name {
            cursor.write_line(&format!("  Country: {country}"), "regular", 10.0, 6.0);
        }
        cursor.write_line(&format!("  Portal: {}", nl.web_url), "oblique", 9.0, 6.0);
        cursor.y -= 4.0;
    }

    // Hudson Rock
    if let Some(hr) = &report.results.hudsonrock {
        if hr.found {
            cursor.write_line("HUDSON ROCK (Infostealer Intelligence)", "bold", 12.0, 8.0);
            cursor.write_line("  Status: COMPROMISED", "bold", 10.0, 6.0);
            cursor.write_line(
                &format!("  Corporate services: {}", hr.total_corporate_services),
                "regular",
                10.0,
                6.0,
            );
            cursor.write_line(
                &format!("  Personal services: {}", hr.total_user_services),
                "regular",
                10.0,
                6.0,
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

                cursor.write_line(
                    &format!("  [{}] Stealer: {family}", i + 1),
                    "regular",
                    10.0,
                    6.0,
                );
                cursor.write_line(&format!("      Date: {date}"), "regular", 10.0, 6.0);
                cursor.write_line(&format!("      Computer: {computer}"), "regular", 10.0, 6.0);

                if !stealer.top_logins.is_empty() {
                    let logins = stealer.top_logins.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
                    cursor.write_line(
                        &format!("      Leaked logins: {logins}"),
                        "regular",
                        10.0,
                        6.0,
                    );
                }
            }
            cursor.y -= 4.0;
        }
    }

    // Google
    if !report.results.google.is_empty() {
        cursor.write_line("GOOGLE", "bold", 12.0, 8.0);
        for (i, item) in report.results.google.iter().take(5).enumerate() {
            let title: String = item.title.chars().take(80).collect();
            cursor.write_line(&format!("  {}. {title}", i + 1), "regular", 10.0, 6.0);
            if !item.link.is_empty() {
                let link: String = item.link.chars().take(80).collect();
                cursor.write_line(&format!("     URL: {link}"), "oblique", 8.0, 5.0);
            }
            if !item.snippet.is_empty() {
                let snippet: String = item.snippet.chars().take(120).collect();
                cursor.write_line(&format!("     {snippet}"), "oblique", 8.0, 5.0);
            }
        }
        cursor.y -= 4.0;
    }

    // Reddit
    if !report.results.reddit.is_empty() {
        cursor.write_line("REDDIT", "bold", 12.0, 8.0);
        for (i, post) in report.results.reddit.iter().take(3).enumerate() {
            let title: String = post.title.chars().take(80).collect();
            cursor.write_line(&format!("  {}. {title}", i + 1), "regular", 10.0, 6.0);
            if !post.url.is_empty() {
                let url: String = post.url.chars().take(80).collect();
                cursor.write_line(&format!("     URL: {url}"), "oblique", 8.0, 5.0);
            }
        }
        cursor.y -= 4.0;
    }

    // GitHub
    if !report.results.github.is_empty() {
        cursor.write_line("GITHUB", "bold", 12.0, 8.0);
        for (i, item) in report.results.github.iter().take(3).enumerate() {
            let mut display = item.repository.clone();
            if !item.path.is_empty() {
                display = format!("{} -> {}", display, item.path);
            }
            let display: String = display.chars().take(80).collect();
            cursor.write_line(&format!("  {}. {display}", i + 1), "regular", 10.0, 6.0);
            if !item.url.is_empty() {
                let url: String = item.url.chars().take(80).collect();
                cursor.write_line(&format!("     URL: {url}"), "oblique", 8.0, 5.0);
            }
        }
        cursor.y -= 4.0;
    }

    // Resumen
    cursor.write_line("SUMMARY", "bold", 12.0, 8.0);
    let mut total_found = 0;
    let services = [
        ("Google", report.results.google.len()),
        ("Reddit", report.results.reddit.len()),
        ("GitHub", report.results.github.len()),
        ("DuckDuckGo", report.results.duckduckgo.len()),
    ];

    for (name, count) in services {
        if count > 0 {
            cursor.write_line(&format!("  {name}: {count} result(s)"), "regular", 10.0, 6.0);
            total_found += count;
        }
    }

    if let Some(hr) = &report.results.hudsonrock {
        if hr.found {
            let count = hr.stealers.len();
            cursor.write_line(&format!("  Hudson Rock: {count} infection(s) found"), "regular", 10.0, 6.0);
            total_found += count;
        }
    }

    cursor.write_line(&format!("  TOTAL RESULTS: {total_found}"), "bold", 10.0, 6.0);

    let file = File::create(&path).map_err(|e| e.to_string())?;
    doc.save(&mut BufWriter::new(file)).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OsintResults, ReportMetadata};

    #[test]
    fn test_clean_text() {
        let dirty = "⚠️ Error: [✓] Found in Perú! • Test";
        let cleaned = clean_text(dirty);
        assert_eq!(cleaned, "! Error: [V] Found in Peru! * Test");
    }

    #[test]
    fn test_pdf_generation() {
        let report = OsintReport {
            metadata: ReportMetadata {
                phone: "+16502530000".to_string(),
                region: "US".to_string(),
                timestamp: "2026-08-25T00:00:00Z".to_string(),
                tool: "SearchPhone OSINT".to_string(),
            },
            results: OsintResults::default(),
        };

        let temp_dir = std::env::temp_dir();
        let res = export_pdf(&report, &temp_dir.to_string_lossy(), "test_report.pdf");
        assert!(res.is_ok());
    }
}
