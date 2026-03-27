//! CSV ↔ grid data conversion for Tabulator integration.
//!
//! Converts between CSV text (master format) and structured grid rows
//! that can be serialized to JSON for Tabulator's setData/getData API.

use road_section::StationData;

/// A single row in the grid editor.
/// Maps to Tabulator columns: 測点名, 単延長L, 左幅員, 右幅員
#[derive(Clone, Debug, PartialEq)]
pub struct GridRow {
    pub name: String,
    pub x: f64,
    pub wl: f64,
    pub wr: f64,
}

/// Parse CSV text (master format) into grid rows.
///
/// Handles:
/// - Header row detection and skip
/// - Empty lines
/// - Full-width digits (全角→半角 conversion)
/// - Whitespace trimming
pub fn csv_to_grid(csv: &str) -> Vec<GridRow> {
    let mut rows = Vec::new();

    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            continue;
        }

        // Skip header rows
        let first_lower = parts[0].to_lowercase();
        if first_lower.contains("測点") || first_lower.contains("name")
            || first_lower.contains("station") || first_lower.contains("延長")
        {
            continue;
        }

        // Try parse x (column 1) as number — skip if not numeric
        let x_str = normalize_number(parts.get(1).unwrap_or(&""));
        let x: f64 = match x_str.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let name = parts[0].to_string();
        let wl_str = normalize_number(parts.get(2).unwrap_or(&"0"));
        let wr_str = normalize_number(parts.get(3).unwrap_or(&"0"));
        let wl: f64 = wl_str.parse().unwrap_or(0.0);
        let wr: f64 = wr_str.parse().unwrap_or(0.0);

        rows.push(GridRow { name, x, wl, wr });
    }

    rows
}

/// Convert grid rows back to CSV text (master format).
pub fn grid_to_csv(rows: &[GridRow]) -> String {
    let mut output = String::from("name,x,wl,wr\n");
    for row in rows {
        output.push_str(&format!("{},{},{},{}\n", row.name, row.x, row.wl, row.wr));
    }
    output
}

/// Convert grid rows to StationData for road-section processing.
pub fn grid_to_stations(rows: &[GridRow]) -> Vec<StationData> {
    rows.iter()
        .map(|r| StationData::new(&r.name, r.x, r.wl, r.wr))
        .collect()
}

/// Normalize full-width digits and special characters to ASCII.
fn normalize_number(s: &str) -> String {
    s.chars().map(|c| match c {
        '０' => '0', '１' => '1', '２' => '2', '３' => '3', '４' => '4',
        '５' => '5', '６' => '6', '７' => '7', '８' => '8', '９' => '9',
        '．' => '.', '－' | 'ー' => '-',
        _ => c,
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    // CSV → Grid: basic parsing
    // ================================================================

    #[test]
    fn test_csv_to_grid_basic() {
        let csv = "name,x,wl,wr\nNo.0,0.0,3.45,3.55\nNo.1,20.0,3.50,3.50\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "No.0");
        assert!((rows[0].x - 0.0).abs() < 1e-9);
        assert!((rows[0].wl - 3.45).abs() < 1e-9);
        assert!((rows[0].wr - 3.55).abs() < 1e-9);
        assert_eq!(rows[1].name, "No.1");
    }

    #[test]
    fn test_csv_to_grid_no_header() {
        let csv = "No.0,0.0,0.8,0.0\n0+1.2,1.15,0.63,0.0\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_csv_to_grid_japanese_header() {
        let csv = "測点名,単延長L,幅員W,幅員右\nNo.0,0,0.8,0\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "No.0");
    }

    #[test]
    fn test_csv_to_grid_missing_wr() {
        let csv = "name,x,wl\nNo.0,0.0,3.45\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 1);
        assert!((rows[0].wr - 0.0).abs() < 1e-9, "Missing wr should default to 0");
    }

    // ================================================================
    // Grid → CSV: roundtrip
    // ================================================================

    #[test]
    fn test_grid_to_csv_basic() {
        let rows = vec![
            GridRow { name: "No.0".into(), x: 0.0, wl: 3.45, wr: 3.55 },
            GridRow { name: "No.1".into(), x: 20.0, wl: 3.50, wr: 3.50 },
        ];
        let csv = grid_to_csv(&rows);
        assert!(csv.starts_with("name,x,wl,wr\n"));
        assert!(csv.contains("No.0,0,3.45,3.55"));
        assert!(csv.contains("No.1,20,3.5,3.5"));
    }

    #[test]
    fn test_roundtrip_csv_grid_csv() {
        let original = "name,x,wl,wr\nNo.0,0,3.45,3.55\nNo.1,20,3.5,3.5\n";
        let rows = csv_to_grid(original);
        let regenerated = grid_to_csv(&rows);
        let rows2 = csv_to_grid(&regenerated);

        assert_eq!(rows.len(), rows2.len());
        for (a, b) in rows.iter().zip(rows2.iter()) {
            assert_eq!(a.name, b.name);
            assert!((a.x - b.x).abs() < 1e-9);
            assert!((a.wl - b.wl).abs() < 1e-9);
            assert!((a.wr - b.wr).abs() < 1e-9);
        }
    }

    #[test]
    fn test_roundtrip_preserves_station_names() {
        let original = "name,x,wl,wr\nNo.0,0,1,1\n0+5.5,5.5,1,1\nNo.1,20,1,1\n";
        let rows = csv_to_grid(original);
        let csv = grid_to_csv(&rows);
        assert!(csv.contains("No.0"));
        assert!(csv.contains("0+5.5"));
        assert!(csv.contains("No.1"));
    }

    // ================================================================
    // Edge cases: empty, whitespace, comments
    // ================================================================

    #[test]
    fn test_csv_to_grid_empty() {
        assert!(csv_to_grid("").is_empty());
    }

    #[test]
    fn test_csv_to_grid_only_header() {
        assert!(csv_to_grid("name,x,wl,wr\n").is_empty());
    }

    #[test]
    fn test_csv_to_grid_empty_lines() {
        let csv = "name,x,wl,wr\n\nNo.0,0,1,1\n\nNo.1,20,1,1\n\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_csv_to_grid_whitespace_padding() {
        let csv = "  No.0  ,  0.0  ,  3.45  ,  3.55  \n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "No.0");
        assert!((rows[0].wl - 3.45).abs() < 1e-9);
    }

    // ================================================================
    // Edge cases: full-width digits (全角数字)
    // ================================================================

    #[test]
    fn test_csv_to_grid_fullwidth_digits() {
        let csv = "No.０,０,３.４５,３.５５\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 1);
        assert!((rows[0].x - 0.0).abs() < 1e-9);
        assert!((rows[0].wl - 3.45).abs() < 1e-9);
        assert!((rows[0].wr - 3.55).abs() < 1e-9);
    }

    #[test]
    fn test_csv_to_grid_fullwidth_minus() {
        let csv = "No.0,0,－1.5,0\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 1);
        assert!((rows[0].wl - (-1.5)).abs() < 1e-9);
    }

    // ================================================================
    // Edge cases: negative width, zero width
    // ================================================================

    #[test]
    fn test_csv_to_grid_negative_width() {
        let csv = "No.0,0,-2.5,0\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 1);
        assert!((rows[0].wl - (-2.5)).abs() < 1e-9);
    }

    #[test]
    fn test_csv_to_grid_zero_width() {
        let csv = "No.0,0,0,0\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 1);
        assert!((rows[0].wl - 0.0).abs() < 1e-9);
        assert!((rows[0].wr - 0.0).abs() < 1e-9);
    }

    // ================================================================
    // Grid → StationData conversion
    // ================================================================

    #[test]
    fn test_grid_to_stations() {
        let rows = vec![
            GridRow { name: "No.0".into(), x: 0.0, wl: 3.0, wr: 3.0 },
            GridRow { name: "No.1".into(), x: 20.0, wl: 2.5, wr: 2.5 },
        ];
        let stations = grid_to_stations(&rows);
        assert_eq!(stations.len(), 2);
        assert_eq!(stations[0].name, "No.0");
        assert!((stations[0].wl - 3.0).abs() < 1e-9);
        assert!((stations[1].x - 20.0).abs() < 1e-9);
    }

    #[test]
    fn test_grid_to_stations_empty() {
        assert!(grid_to_stations(&[]).is_empty());
    }

    // ================================================================
    // normalize_number
    // ================================================================

    #[test]
    fn test_normalize_fullwidth_all_digits() {
        assert_eq!(normalize_number("０１２３４５６７８９"), "0123456789");
    }

    #[test]
    fn test_normalize_fullwidth_decimal() {
        assert_eq!(normalize_number("３．１４"), "3.14");
    }

    #[test]
    fn test_normalize_fullwidth_negative() {
        assert_eq!(normalize_number("－５．０"), "-5.0");
    }

    #[test]
    fn test_normalize_ascii_passthrough() {
        assert_eq!(normalize_number("3.14"), "3.14");
        assert_eq!(normalize_number("-5.0"), "-5.0");
    }

    #[test]
    fn test_normalize_katakana_minus() {
        // ー (katakana prolonged sound mark) sometimes used as minus
        assert_eq!(normalize_number("ー５"), "-5");
    }

    // ================================================================
    // Tabulator CDN check (index.html)
    // ================================================================

    #[test]
    fn test_index_html_exists() {
        let index_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("index.html");
        assert!(index_path.exists(), "web/index.html must exist");
    }

    // ================================================================
    // Grid data edge: non-numeric x values are skipped
    // ================================================================

    #[test]
    fn test_csv_to_grid_non_numeric_x_skipped() {
        let csv = "No.0,abc,1,1\nNo.1,20,1,1\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 1, "Row with non-numeric x should be skipped");
        assert_eq!(rows[0].name, "No.1");
    }

    #[test]
    fn test_csv_to_grid_mixed_valid_invalid() {
        let csv = "name,x,wl,wr\nNo.0,0,1,1\nbad,not_a_number,1,1\nNo.2,40,2,2\n";
        let rows = csv_to_grid(csv);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "No.0");
        assert_eq!(rows[1].name, "No.2");
    }
}
