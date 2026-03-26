//! Section detection and CSV/Excel extraction
//!
//! Ported from csv_to_dxf/src/processing.py:
//!   extract_section_data(), get_available_sections()

use std::fs;
use std::path::Path;

use regex::Regex;

use crate::RawRow;

/// Error type for CSV/section parsing failures.
#[derive(Debug)]
pub enum ParseError {
    NoData,
    InvalidFormat(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NoData => write!(f, "No data found"),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// List available section names from text content and filename.
///
/// Priority:
/// 1. Filename like `区間3.csv` → `["区間3"]`
/// 2. Body contains `区間X,台形計算` → all matches
/// 3. Fallback → `["区間1"]`
pub fn get_available_sections(text: &str, filename: &str) -> Vec<String> {
    // Rule 1: Filename match
    let fname_re = Regex::new(r"^(区間\d+)\.csv$").unwrap();
    if let Some(caps) = fname_re.captures(filename) {
        return vec![caps.get(1).unwrap().as_str().to_string()];
    }

    // Rule 2: Body search for 区間X,台形計算
    let section_re = Regex::new(r"(?m)^(区間\d+),台形計算").unwrap();
    let sections: Vec<String> = section_re
        .captures_iter(text)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .collect();
    if !sections.is_empty() {
        return sections;
    }

    // Rule 3: Fallback
    vec!["区間1".to_string()]
}

/// Extract rows for a named section (e.g. "区間1") from text content.
///
/// If the text contains `区間X,台形計算` headers, extracts just that block.
/// Otherwise treats the entire text as a single-section CSV.
pub fn extract_section_data(text: &str, section_name: &str) -> Result<Vec<RawRow>, ParseError> {
    // Try block extraction: 区間X,台形計算 ... (until next 区間 or end of string)
    let pattern = format!(
        r"{},台形計算[^\n]*\n([\s\S]+?)(?:\n区間\d+|\z)",
        regex::escape(section_name)
    );
    let re = Regex::new(&pattern).map_err(|e| ParseError::InvalidFormat(e.to_string()))?;

    let block = if let Some(caps) = re.captures(text) {
        caps.get(1).unwrap().as_str().trim().to_string()
    } else {
        // No section headers — treat whole text as plain CSV
        text.to_string()
    };

    parse_csv_block(&block)
}

/// Extract section data from a file path (convenience wrapper).
pub fn extract_section_data_from_file(
    path: &Path,
    section_name: &str,
) -> Result<Vec<RawRow>, ParseError> {
    let text = read_file_text(path)
        .map_err(|e| ParseError::InvalidFormat(e))?;
    extract_section_data(&text, section_name)
}

/// List available sections from a file path (convenience wrapper).
pub fn get_available_sections_from_file(path: &Path) -> Vec<String> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if let Ok(text) = read_file_text(path) {
        get_available_sections(&text, filename)
    } else {
        // Can't read file; try filename-only match
        get_available_sections("", filename)
    }
}

/// Parse a CSV block into RawRows.
///
/// Handles column mapping: 測点名→name, 単延長L→x, 幅員W→wl
fn parse_csv_block(block: &str) -> Result<Vec<RawRow>, ParseError> {
    let lines: Vec<&str> = block.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return Err(ParseError::NoData);
    }

    // Detect header row
    let first_parts: Vec<&str> = lines[0].split(',').map(|s| s.trim()).collect();
    let has_header = first_parts.iter().any(|p| {
        p.contains("測点") || p.contains("延長") || p.contains("幅員")
            || p.eq_ignore_ascii_case("name") || p.eq_ignore_ascii_case("x")
    });

    // Column indices (defaults for headerless 4-column CSV)
    let (mut name_col, mut x_col, mut wl_col, mut wr_col): (usize, usize, usize, usize) =
        (0, 1, 2, 3);
    let start_row;
    let mut has_wr_header = false;

    if has_header {
        start_row = 1;
        for (i, part) in first_parts.iter().enumerate() {
            let p = part.to_lowercase();
            if p.contains("測点") || p == "name" {
                name_col = i;
            } else if p.contains("単延長") || p.contains("延長") || p == "x" {
                x_col = i;
            } else if p.contains("平均幅員") {
                // skip — 平均幅員Wa is a computed column, not input
            } else if p.contains("幅員") || p == "wl" {
                wl_col = i;
            } else if p == "wr" {
                wr_col = i;
                has_wr_header = true;
            }
        }
        // If header exists but no "wr" column, wr defaults to 0.0 for all rows
        if !has_wr_header {
            wr_col = usize::MAX; // sentinel: no wr column
        }
    } else {
        start_row = 0;
        // For headerless CSV, check if 4th column exists in data
    }

    let mut rows = Vec::new();
    for line in &lines[start_row..] {
        // Skip comment lines
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        // x must be parseable
        let x_str = parts.get(x_col).unwrap_or(&"");
        let x: f64 = match x_str.parse() {
            Ok(v) => v,
            Err(_) => continue, // skip rows with non-numeric x
        };

        let name = parts.get(name_col).unwrap_or(&"").to_string();
        let wl: f64 = parts
            .get(wl_col)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        // wr: might not exist in column layout
        let wr: f64 = if wr_col == usize::MAX {
            0.0
        } else if wr_col < parts.len() {
            parts
                .get(wr_col)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0)
        } else {
            0.0
        };

        rows.push(RawRow { name, x, wl, wr });
    }

    if rows.is_empty() {
        return Err(ParseError::NoData);
    }
    Ok(rows)
}

fn read_file_text(path: &Path) -> Result<String, String> {
    // Try UTF-8 first
    if let Ok(text) = fs::read_to_string(path) {
        return Ok(text);
    }
    // Fallback: Shift_JIS
    let bytes =
        fs::read(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let (cow, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&bytes);
    if had_errors {
        return Err(format!(
            "Failed to decode {}: encoding error",
            path.display()
        ));
    }
    Ok(cow.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sections_from_filename() {
        assert_eq!(get_available_sections("", "区間1.csv"), vec!["区間1"]);
        assert_eq!(get_available_sections("", "区間3.csv"), vec!["区間3"]);
        assert_eq!(get_available_sections("", "data.csv"), vec!["区間1"]); // fallback
    }

    #[test]
    fn test_get_sections_from_body() {
        let text = ",,,,\n,,,,\n区間1,台形計算,,,\n測点名,単延長L,幅員W,平均幅員Wa,面積m2\nNo.0,0,0.8,,\n";
        assert_eq!(
            get_available_sections(text, "sheet.csv"),
            vec!["区間1"]
        );
    }

    #[test]
    fn test_extract_simple_csv() {
        let text = "name,x,wl,wr\nNo.0,0.00,0.80,0.00\n0+1.2,1.15,0.63,0.00\n";
        let rows = extract_section_data(text, "区間1").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "No.0");
        assert!((rows[0].x - 0.0).abs() < 0.001);
        assert!((rows[0].wl - 0.80).abs() < 0.001);
    }

    #[test]
    fn test_extract_headerless_csv() {
        let text = "No.0,0.0,3.45,3.55\n10m,10.0,3.50,3.50\n";
        let rows = extract_section_data(text, "区間1").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "No.0");
        assert!((rows[0].wl - 3.45).abs() < 0.001);
        assert!((rows[0].wr - 3.55).abs() < 0.001);
    }

    #[test]
    fn test_extract_section_block() {
        let text = ",,,,\n,,,,\n区間1,台形計算,,,\n測点名,単延長L,幅員W,平均幅員Wa,面積m2\nNo.0,0,0.8,,\n0+1.2,1.15,0.63,,\n";
        let rows = extract_section_data(text, "区間1").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "No.0");
        assert!((rows[0].wl - 0.80).abs() < 0.001);
        assert!((rows[0].wr - 0.0).abs() < 0.001); // wr defaults to 0
    }

    #[test]
    fn test_extract_no_wr_column() {
        let text = "name,x,wl\nNo.0,0.0,3.45\n";
        let rows = extract_section_data(text, "区間1").unwrap();
        assert!((rows[0].wr - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_plain_csv() {
        let text = "name,x,wl,wr\nNo.0,0.0,3.45,3.55\n10m,10.0,3.50,3.50\n";
        let rows = extract_section_data(text, "区間1").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "No.0");
        assert!((rows[0].wl - 3.45).abs() < 1e-9);
    }

    #[test]
    fn test_multi_section() {
        let text = "\
,,,,
区間1,台形計算,,,
測点名,単延長L,幅員W,平均幅員Wa,面積m2
No.0,0,0.8,,
,1.15,0.63,0.715,0.82
,,,,
区間3,台形計算,,,
測点名,単延長L,幅員W,平均幅員Wa,面積m2
0+7.9,0,0.5,,
,2.03,0.5,0.5,1.02
";
        let s1 = extract_section_data(text, "区間1").unwrap();
        assert_eq!(s1.len(), 2);
        assert_eq!(s1[0].name, "No.0");
        assert!((s1[0].x - 0.0).abs() < 1e-9);
        assert!((s1[1].x - 1.15).abs() < 1e-9);

        let s3 = extract_section_data(text, "区間3").unwrap();
        assert_eq!(s3.len(), 2);
        assert_eq!(s3[0].name, "0+7.9");
    }

    #[test]
    fn test_section_header_column_mapping() {
        let text = "測点名,単延長L,幅員W,平均幅員Wa,面積m2\nNo.0,0,0.8,,\n";
        let rows = extract_section_data(text, "区間1").unwrap();
        assert_eq!(rows[0].name, "No.0");
        assert!((rows[0].wl - 0.8).abs() < 1e-9);
    }

    // ================================================================
    // No sections in body → fallback
    // ================================================================

    #[test]
    fn test_get_sections_no_body_no_filename_match() {
        assert_eq!(get_available_sections("random content", "data.csv"), vec!["区間1"]);
    }

    // ================================================================
    // Multiple sections in body
    // ================================================================

    #[test]
    fn test_get_sections_multiple_in_body() {
        let text = "区間1,台形計算,,,\ndata\n区間2,台形計算,,,\ndata\n区間5,台形計算,,,\n";
        let sections = get_available_sections(text, "sheet.csv");
        assert_eq!(sections, vec!["区間1", "区間2", "区間5"]);
    }

    // ================================================================
    // Duplicate section names
    // ================================================================

    #[test]
    fn test_get_sections_duplicate_names() {
        let text = "区間1,台形計算,,,\ndata\n区間1,台形計算,,,\nmore data\n";
        let sections = get_available_sections(text, "sheet.csv");
        // Regex finds both occurrences
        assert_eq!(sections.len(), 2);
    }

    // ================================================================
    // Empty CSV block
    // ================================================================

    #[test]
    fn test_extract_empty_content() {
        let result = extract_section_data("", "区間1");
        assert!(result.is_err());
    }

    // ================================================================
    // Comment lines and blank lines
    // ================================================================

    #[test]
    fn test_extract_comment_lines_skipped() {
        let text = "name,x,wl,wr\n# comment\nNo.0,0.0,1.0,1.0\n# another comment\n";
        let rows = extract_section_data(text, "区間1").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "No.0");
    }

    // ================================================================
    // Non-numeric x values skipped
    // ================================================================

    #[test]
    fn test_extract_non_numeric_x_skipped() {
        let text = "name,x,wl,wr\nNo.0,abc,1.0,1.0\nNo.1,10.0,2.0,2.0\n";
        let rows = extract_section_data(text, "区間1").unwrap();
        assert_eq!(rows.len(), 1, "Row with non-numeric x should be skipped");
        assert_eq!(rows[0].name, "No.1");
    }

    // ================================================================
    // Section not found in multi-section file
    // ================================================================

    #[test]
    fn test_extract_section_not_found() {
        let text = "区間1,台形計算,,,\n測点名,単延長L,幅員W,平均幅員Wa,面積m2\nNo.0,0,0.8,,\n";
        // Looking for 区間2 which doesn't exist → falls back to whole text parse
        let result = extract_section_data(text, "区間2");
        // The whole text starts with "区間1,..." which has non-numeric x → may error
        // Actually the first row "区間1,台形計算,,," → "台形計算" can't parse as x → skipped
        // "測点名,単延長L,..." is detected as header
        // "No.0,0,0.8,," → valid
        assert!(result.is_ok());
    }

    // ================================================================
    // English column headers
    // ================================================================

    #[test]
    fn test_extract_english_headers() {
        let text = "name,x,wl,wr\nA,0.0,1.5,2.5\nB,5.0,1.0,2.0\n";
        let rows = extract_section_data(text, "区間1").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "A");
        assert!((rows[0].wr - 2.5).abs() < 1e-9);
    }

    // ================================================================
    // ParseError Display
    // ================================================================

    #[test]
    fn test_parse_error_display() {
        let e1 = ParseError::NoData;
        assert_eq!(format!("{}", e1), "No data found");

        let e2 = ParseError::InvalidFormat("test error".to_string());
        assert!(format!("{}", e2).contains("test error"));
    }
}
