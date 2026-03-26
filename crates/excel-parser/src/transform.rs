//! Pipeline integration: extract → to_cumulative → fill_station_names → round
//!
//! Ported from csv_to_dxf/src/processing.py: transform_section()

use std::path::Path;

use crate::distance::to_cumulative_rows;
use crate::section_detector::{
    extract_section_data, extract_section_data_from_file, get_available_sections,
    get_available_sections_from_file, ParseError,
};
use crate::station_name::fill_station_names;
use crate::{RawRow, ROUND_N};

/// Round a f64 to ROUND_N decimal places.
fn round_n(val: f64, n: u32) -> f64 {
    let factor = 10_f64.powi(n as i32);
    (val * factor).round() / factor
}

/// Transform raw section data through the full pipeline.
///
/// 1. Extract rows from CSV
/// 2. Convert to cumulative distances
/// 3. Fill missing station names
/// 4. Round numeric values
pub fn transform_section(rows: &mut Vec<RawRow>) {
    to_cumulative_rows(rows);
    fill_station_names(rows);
    for row in rows.iter_mut() {
        row.x = round_n(row.x, ROUND_N);
        row.wl = round_n(row.wl, ROUND_N);
        row.wr = round_n(row.wr, ROUND_N);
    }
}

/// Full pipeline: file path + section name → transformed rows.
pub fn extract_and_transform(path: &Path, section_name: &str) -> Result<Vec<RawRow>, ParseError> {
    let mut rows = extract_section_data_from_file(path, section_name)?;
    transform_section(&mut rows);
    Ok(rows)
}

/// Full pipeline from text content.
pub fn extract_and_transform_text(
    text: &str,
    section_name: &str,
) -> Result<Vec<RawRow>, ParseError> {
    let mut rows = extract_section_data(text, section_name)?;
    transform_section(&mut rows);
    Ok(rows)
}

/// List available sections in a file.
pub fn list_sections(path: &Path) -> Vec<String> {
    get_available_sections_from_file(path)
}

/// List available sections from text content.
pub fn list_sections_text(text: &str, filename: &str) -> Vec<String> {
    get_available_sections(text, filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_span_data() {
        let text = "\
測点名,単延長L,幅員W,平均幅員Wa,面積m2
No.0,0,0.8,,
,1.15,0.63,0.715,0.82
,2.1,0.5,0.565,1.19
,2.15,0.5,0.5,1.08
0+7,1.6,0.5,0.5,0.8";
        let rows = extract_and_transform_text(text, "区間1").unwrap();
        assert_eq!(rows.len(), 5);

        // Cumulative: 0, 1.15, 3.25, 5.40, 7.00
        assert_eq!(rows[0].name, "No.0");
        assert!((rows[0].x - 0.0).abs() < 0.01);

        assert!((rows[1].x - 1.15).abs() < 0.01);

        assert!((rows[2].x - 3.25).abs() < 0.01);

        assert!((rows[4].x - 7.0).abs() < 0.01);
        assert_eq!(rows[4].name, "0+7");
    }

    #[test]
    fn test_transform_cumulative_data() {
        let text = "name,x,wl,wr\nNo.0,0.0,3.45,3.55\n10m,10.0,3.50,3.50\nNo.1,20.0,3.45,3.55\n10m,30.0,3.45,3.55\nNo.2,40.0,3.55,3.55\n";
        let rows = extract_and_transform_text(text, "区間1").unwrap();
        // Already cumulative (monotonic, median diff = 10 < 16)
        assert_eq!(rows.len(), 5);
        assert!((rows[4].x - 40.0).abs() < 0.01);
        assert_eq!(rows[0].name, "No.0");
        assert_eq!(rows[2].name, "No.1");
        assert_eq!(rows[4].name, "No.2");
    }

    #[test]
    fn test_round_n() {
        assert!((round_n(3.456, 2) - 3.46).abs() < 1e-9);
        assert!((round_n(3.454, 2) - 3.45).abs() < 1e-9);
        assert!((round_n(0.005, 2) - 0.01).abs() < 1e-9);
    }

    // ================================================================
    // round_n additional cases
    // ================================================================

    #[test]
    fn test_round_n_zero() {
        assert!((round_n(0.0, 2) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_round_n_negative() {
        assert!((round_n(-3.456, 2) - (-3.46)).abs() < 1e-9);
    }

    #[test]
    fn test_round_n_zero_decimals() {
        assert!((round_n(3.7, 0) - 4.0).abs() < 1e-9);
    }

    // ================================================================
    // transform_section pipeline
    // ================================================================

    #[test]
    fn test_transform_section_rounding() {
        let mut rows = vec![
            RawRow { name: "No.0".into(), x: 0.0, wl: 1.234567, wr: 2.999 },
        ];
        transform_section(&mut rows);
        assert!((rows[0].wl - 1.23).abs() < 1e-9, "Should round to 2 decimals");
        assert!((rows[0].wr - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_transform_section_fills_names() {
        let mut rows = vec![
            RawRow { name: "".into(), x: 0.0, wl: 1.0, wr: 1.0 },
            RawRow { name: "".into(), x: 10.0, wl: 1.0, wr: 1.0 },
        ];
        transform_section(&mut rows);
        assert_eq!(rows[0].name, "No.0");
        assert_eq!(rows[1].name, "0+10");
    }

    // ================================================================
    // extract_and_transform_text end-to-end
    // ================================================================

    #[test]
    fn test_extract_and_transform_text_empty() {
        let result = extract_and_transform_text("", "区間1");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_and_transform_text_basic() {
        let text = "name,x,wl,wr\nNo.0,0.0,2.5,3.5\nNo.1,20.0,2.5,3.5\n";
        let rows = extract_and_transform_text(text, "区間1").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "No.0");
        assert_eq!(rows[1].name, "No.1");
    }

    // ================================================================
    // list_sections_text
    // ================================================================

    #[test]
    fn test_list_sections_text_fallback() {
        let sections = list_sections_text("no sections here", "random.csv");
        assert_eq!(sections, vec!["区間1"]);
    }

    #[test]
    fn test_list_sections_text_from_filename() {
        let sections = list_sections_text("", "区間3.csv");
        assert_eq!(sections, vec!["区間3"]);
    }
}
