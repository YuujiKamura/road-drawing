//! CSV parser for triangle list data
//!
//! Supports 3 formats from trianglelist:
//! - MIN (4 columns): number, length_a, length_b, length_c
//! - CONN (6 columns): + parent_number, connection_type
//! - FULL (28 columns): + name, points, color, dim alignment, angle, ...
//!
//! CSV header block:
//!   koujiname, <project name>
//!   rosenname, <route name>
//!   gyousyaname, <contractor name>
//!   zumennum, <drawing number>
//!   <triangle rows>
//!   <optional: Deduction rows>

/// CSV header metadata
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CsvHeader {
    pub koujiname: String,
    pub rosenname: String,
    pub gyousyaname: String,
    pub zumennum: String,
}

/// Raw triangle data parsed from CSV
#[derive(Clone, Debug, PartialEq)]
pub struct TriangleRow {
    pub number: i32,
    pub length_a: f64,
    pub length_b: f64,
    pub length_c: f64,
    pub parent_number: i32,     // -1 = independent
    pub connection_type: i32,   // -1 = none, 1 = parent's B, 2 = parent's C
}

/// Parsed CSV result
#[derive(Clone, Debug)]
pub struct ParsedCsv {
    pub header: CsvHeader,
    pub triangles: Vec<TriangleRow>,
}

#[derive(Debug)]
pub enum CsvError {
    InvalidFormat(String),
    TooFewColumns { row: usize, got: usize, need: usize },
    InvalidNumber { row: usize, col: &'static str, value: String },
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            CsvError::TooFewColumns { row, got, need } =>
                write!(f, "Row {}: got {} columns, need {}", row, got, need),
            CsvError::InvalidNumber { row, col, value } =>
                write!(f, "Row {}: invalid {} value '{}'", row, col, value),
        }
    }
}

/// Parse a finite f64 (rejects NaN, Inf)
fn parse_finite_f64(s: &str) -> Result<f64, ()> {
    let v: f64 = s.parse().map_err(|_| ())?;
    if v.is_finite() { Ok(v) } else { Err(()) }
}

/// Parse triangle list CSV text
/// Detects format automatically: MIN (4 col), CONN (6 col), or FULL (28 col)
pub fn parse_csv(text: &str) -> Result<ParsedCsv, CsvError> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Err(CsvError::InvalidFormat("Empty input".to_string()));
    }

    let mut header = CsvHeader::default();
    let mut data_start = 0;

    // Parse header lines (koujiname, rosenname, gyousyaname, zumennum)
    let header_keys = ["koujiname", "rosenname", "gyousyaname", "zumennum"];
    for (i, line) in lines.iter().enumerate() {
        let parts: Vec<&str> = line.splitn(2, ',').collect();
        if parts.len() < 2 {
            data_start = i;
            break;
        }
        let key = parts[0].trim().to_lowercase();
        let val = parts[1].trim().to_string();
        match key.as_str() {
            "koujiname" => header.koujiname = val,
            "rosenname" => header.rosenname = val,
            "gyousyaname" => header.gyousyaname = val,
            "zumennum" => {
                header.zumennum = val;
                data_start = i + 1;
                break;
            }
            _ => {
                data_start = i;
                break;
            }
        }
    }

    if data_start >= lines.len() {
        return Err(CsvError::InvalidFormat("No data rows after header".to_string()));
    }

    let mut triangles = Vec::new();
    for (row_offset, line) in lines[data_start..].iter().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let row_num = data_start + row_offset + 1;
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let col_count = parts.len();

        if col_count < 4 {
            return Err(CsvError::TooFewColumns { row: row_num, got: col_count, need: 4 });
        }

        let number: i32 = parts[0].parse().map_err(|_| CsvError::InvalidNumber {
            row: row_num, col: "number", value: parts[0].to_string(),
        })?;
        let length_a: f64 = parse_finite_f64(parts[1]).map_err(|_| CsvError::InvalidNumber {
            row: row_num, col: "length_a", value: parts[1].to_string(),
        })?;
        let length_b: f64 = parse_finite_f64(parts[2]).map_err(|_| CsvError::InvalidNumber {
            row: row_num, col: "length_b", value: parts[2].to_string(),
        })?;
        let length_c: f64 = parse_finite_f64(parts[3]).map_err(|_| CsvError::InvalidNumber {
            row: row_num, col: "length_c", value: parts[3].to_string(),
        })?;

        let (parent_number, connection_type) = if col_count >= 6 {
            let pn: i32 = parts[4].parse().map_err(|_| CsvError::InvalidNumber {
                row: row_num, col: "parent_number", value: parts[4].to_string(),
            })?;
            let ct: i32 = parts[5].parse().map_err(|_| CsvError::InvalidNumber {
                row: row_num, col: "connection_type", value: parts[5].to_string(),
            })?;
            (pn, ct)
        } else {
            (-1, -1)
        };

        triangles.push(TriangleRow {
            number,
            length_a,
            length_b,
            length_c,
            parent_number,
            connection_type,
        });
    }

    if triangles.is_empty() {
        return Err(CsvError::InvalidFormat("No triangle rows found".to_string()));
    }

    Ok(ParsedCsv { header, triangles })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    // Header parsing
    // ================================================================

    #[test]
    fn test_parse_header_minimal() {
        let csv = "\
koujiname, 最小形式テスト
rosenname, テスト路線
gyousyaname, テスト業者
zumennum, 1
1, 6.0, 5.0, 4.0
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.header.koujiname, "最小形式テスト");
        assert_eq!(result.header.rosenname, "テスト路線");
        assert_eq!(result.header.gyousyaname, "テスト業者");
        assert_eq!(result.header.zumennum, "1");
    }

    // ================================================================
    // MIN format (4 columns): number, A, B, C
    // From minimal.csv
    // ================================================================

    #[test]
    fn test_parse_min_format() {
        let csv = "\
koujiname, 最小形式テスト
rosenname, テスト路線
gyousyaname, テスト業者
zumennum, 1
1, 6.0, 5.0, 4.0
2, 5.5, 4.5, 3.5
3, 4.0, 3.5, 3.0
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.triangles.len(), 3);

        // Triangle 1: independent
        assert_eq!(result.triangles[0].number, 1);
        assert!((result.triangles[0].length_a - 6.0).abs() < 0.001);
        assert!((result.triangles[0].length_b - 5.0).abs() < 0.001);
        assert!((result.triangles[0].length_c - 4.0).abs() < 0.001);
        assert_eq!(result.triangles[0].parent_number, -1);
        assert_eq!(result.triangles[0].connection_type, -1);

        // Triangle 3
        assert_eq!(result.triangles[2].number, 3);
        assert!((result.triangles[2].length_a - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_min_format_all_independent() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, 5.0, 4.0
2, 5.5, 4.5, 3.5
3, 4.0, 3.5, 3.0
";
        let result = parse_csv(csv).unwrap();
        for t in &result.triangles {
            assert_eq!(t.parent_number, -1, "MIN format triangles should be independent");
            assert_eq!(t.connection_type, -1);
        }
    }

    // ================================================================
    // CONN format (6 columns): number, A, B, C, parent, connection_type
    // From connected.csv
    // ================================================================

    #[test]
    fn test_parse_conn_format() {
        let csv = "\
koujiname, 接続形式テスト
rosenname, テスト路線
gyousyaname, テスト業者
zumennum, 1
1, 6.0, 5.0, 4.0, -1, -1
2, 5.0, 4.0, 3.0, 1, 1
3, 4.0, 3.5, 3.0, 1, 2
4, 4.0, 3.5, 3.0, 2, 1
5, 3.0, 2.5, 2.0, 2, 2
6, 3.5, 3.0, 2.5, 3, 1
7, 3.0, 2.5, 2.0, 3, 2
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.triangles.len(), 7);

        // Triangle 1: independent
        assert_eq!(result.triangles[0].parent_number, -1);
        assert_eq!(result.triangles[0].connection_type, -1);

        // Triangle 2: connects to parent 1's B edge
        assert_eq!(result.triangles[1].number, 2);
        assert!((result.triangles[1].length_a - 5.0).abs() < 0.001);
        assert_eq!(result.triangles[1].parent_number, 1);
        assert_eq!(result.triangles[1].connection_type, 1); // parent's B edge

        // Triangle 3: connects to parent 1's C edge
        assert_eq!(result.triangles[2].number, 3);
        assert!((result.triangles[2].length_a - 4.0).abs() < 0.001);
        assert_eq!(result.triangles[2].parent_number, 1);
        assert_eq!(result.triangles[2].connection_type, 2); // parent's C edge
    }

    #[test]
    fn test_conn_format_edge_length_match() {
        // Child's A-edge must match parent's connection edge
        // Triangle 2 connects to parent 1's B edge:
        //   child.length_a (5.0) == parent.length_b (5.0) ✓
        // Triangle 3 connects to parent 1's C edge:
        //   child.length_a (4.0) == parent.length_c (4.0) ✓
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, 5.0, 4.0, -1, -1
2, 5.0, 4.0, 3.0, 1, 1
3, 4.0, 3.5, 3.0, 1, 2
";
        let result = parse_csv(csv).unwrap();
        let parent = &result.triangles[0];
        let child_b = &result.triangles[1];
        let child_c = &result.triangles[2];

        // child on B: child.A == parent.B
        assert!((child_b.length_a - parent.length_b).abs() < 0.001,
            "Child on B edge: A={} should match parent B={}", child_b.length_a, parent.length_b);

        // child on C: child.A == parent.C
        assert!((child_c.length_a - parent.length_c).abs() < 0.001,
            "Child on C edge: A={} should match parent C={}", child_c.length_a, parent.length_c);
    }

    // ================================================================
    // Error handling
    // ================================================================

    #[test]
    fn test_parse_too_few_columns() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, 5.0
";
        let result = parse_csv(csv);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_number() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, abc, 5.0, 4.0
";
        let result = parse_csv(csv);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty() {
        let csv = "";
        let result = parse_csv(csv);
        assert!(result.is_err());
    }

    // ================================================================
    // Missing columns edge cases
    // ================================================================

    #[test]
    fn test_parse_exactly_4_columns() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 3.0, 4.0, 5.0
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.triangles.len(), 1);
        assert_eq!(result.triangles[0].parent_number, -1);
    }

    #[test]
    fn test_parse_5_columns_treated_as_conn() {
        // 5 columns: not enough for CONN (needs 6), but parse should handle
        // columns 4 exists but 5 doesn't → should this be TooFewColumns?
        // Looking at the code: col_count >= 6 → no, so falls back to (-1, -1)
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 3.0, 4.0, 5.0, 1
";
        let result = parse_csv(csv).unwrap();
        // 5 columns < 6, so treated as MIN with extra column
        assert_eq!(result.triangles[0].parent_number, -1);
    }

    #[test]
    fn test_parse_3_columns_error() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 3.0, 4.0
";
        let result = parse_csv(csv);
        assert!(result.is_err());
        match result.unwrap_err() {
            CsvError::TooFewColumns { got: 3, need: 4, .. } => {},
            other => panic!("Expected TooFewColumns, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_1_column_error() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1
";
        let result = parse_csv(csv);
        assert!(result.is_err());
    }

    // ================================================================
    // Unicode names in header
    // ================================================================

    #[test]
    fn test_parse_unicode_header() {
        let csv = "\
koujiname, 道路補修工事　第１号
rosenname, 国道１２３号線
gyousyaname, 株式会社テスト建設
zumennum, 図面-001
1, 6.0, 5.0, 4.0
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.header.koujiname, "道路補修工事　第１号");
        assert_eq!(result.header.rosenname, "国道１２３号線");
        assert_eq!(result.header.gyousyaname, "株式会社テスト建設");
        assert_eq!(result.header.zumennum, "図面-001");
    }

    #[test]
    fn test_parse_header_with_commas_in_value() {
        // splitn(2, ',') should keep everything after first comma
        let csv = "\
koujiname, 工事A, 追加情報
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, 5.0, 4.0
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.header.koujiname, "工事A, 追加情報");
    }

    // ================================================================
    // Empty lines and whitespace
    // ================================================================

    #[test]
    fn test_parse_empty_lines_between_data() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, 5.0, 4.0

2, 5.0, 4.0, 3.0

3, 4.0, 3.5, 3.0
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.triangles.len(), 3);
    }

    #[test]
    fn test_parse_whitespace_in_values() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
 1 ,  6.0 ,  5.0 ,  4.0
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.triangles[0].number, 1);
        assert!((result.triangles[0].length_a - 6.0).abs() < 0.001);
    }

    // ================================================================
    // No header (data starts immediately)
    // ================================================================

    #[test]
    fn test_parse_no_header() {
        // First line doesn't match header keys → treated as data start
        let csv = "1, 6.0, 5.0, 4.0\n2, 5.0, 4.0, 3.0\n";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.triangles.len(), 2);
        assert_eq!(result.header.koujiname, "");
    }

    // ================================================================
    // Header only, no data
    // ================================================================

    #[test]
    fn test_parse_header_only_no_data() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
";
        let result = parse_csv(csv);
        assert!(result.is_err());
    }

    // ================================================================
    // Invalid number formats
    // ================================================================

    #[test]
    fn test_parse_invalid_triangle_number() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
abc, 6.0, 5.0, 4.0
";
        let result = parse_csv(csv);
        assert!(result.is_err());
        match result.unwrap_err() {
            CsvError::InvalidNumber { col: "number", .. } => {},
            other => panic!("Expected InvalidNumber for number, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_invalid_length_a() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, xyz, 5.0, 4.0
";
        let result = parse_csv(csv);
        assert!(result.is_err());
        match result.unwrap_err() {
            CsvError::InvalidNumber { col: "length_a", value, .. } => {
                assert_eq!(value, "xyz");
            },
            other => panic!("Expected InvalidNumber for length_a, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_invalid_length_b() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, abc, 4.0
";
        let result = parse_csv(csv);
        assert!(result.is_err());
        match result.unwrap_err() {
            CsvError::InvalidNumber { col: "length_b", value, .. } => {
                assert_eq!(value, "abc");
            },
            other => panic!("Expected InvalidNumber for length_b, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_invalid_length_c() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, 5.0, --
";
        let result = parse_csv(csv);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_parent_number() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, 5.0, 4.0, abc, 1
";
        let result = parse_csv(csv);
        assert!(result.is_err());
        match result.unwrap_err() {
            CsvError::InvalidNumber { col: "parent_number", .. } => {},
            other => panic!("Expected InvalidNumber for parent_number, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_invalid_connection_type() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, 5.0, 4.0, -1, xyz
";
        let result = parse_csv(csv);
        assert!(result.is_err());
        match result.unwrap_err() {
            CsvError::InvalidNumber { col: "connection_type", .. } => {},
            other => panic!("Expected InvalidNumber for connection_type, got {:?}", other),
        }
    }

    // ================================================================
    // CsvError Display
    // ================================================================

    #[test]
    fn test_csv_error_display_invalid_format() {
        let err = CsvError::InvalidFormat("test error".to_string());
        assert!(format!("{}", err).contains("test error"));
    }

    #[test]
    fn test_csv_error_display_too_few_columns() {
        let err = CsvError::TooFewColumns { row: 5, got: 2, need: 4 };
        let msg = format!("{}", err);
        assert!(msg.contains("5"));
        assert!(msg.contains("2"));
        assert!(msg.contains("4"));
    }

    #[test]
    fn test_csv_error_display_invalid_number() {
        let err = CsvError::InvalidNumber { row: 3, col: "length_a", value: "xyz".to_string() };
        let msg = format!("{}", err);
        assert!(msg.contains("xyz"));
        assert!(msg.contains("length_a"));
    }

    // ================================================================
    // CONN format roundtrip data integrity
    // ================================================================

    #[test]
    fn test_parse_conn_preserves_all_fields() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1
1, 6.0, 5.0, 4.0, -1, -1
2, 5.0, 4.0, 3.0, 1, 1
3, 4.0, 3.5, 3.0, 1, 2
";
        let result = parse_csv(csv).unwrap();
        let t2 = &result.triangles[1];
        assert_eq!(t2.number, 2);
        assert!((t2.length_a - 5.0).abs() < 0.001);
        assert!((t2.length_b - 4.0).abs() < 0.001);
        assert!((t2.length_c - 3.0).abs() < 0.001);
        assert_eq!(t2.parent_number, 1);
        assert_eq!(t2.connection_type, 1);
    }

    // ================================================================
    // Partial header
    // ================================================================

    #[test]
    fn test_parse_partial_header() {
        // Only koujiname and rosenname, then data starts
        let csv = "\
koujiname, test
rosenname, test
1, 6.0, 5.0, 4.0
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.header.koujiname, "test");
        assert_eq!(result.header.rosenname, "test");
        assert_eq!(result.header.gyousyaname, ""); // not provided
        assert_eq!(result.triangles.len(), 1);
    }

    // ================================================================
    // CsvHeader and TriangleRow derive traits
    // ================================================================

    #[test]
    fn test_csv_header_default() {
        let h = CsvHeader::default();
        assert_eq!(h.koujiname, "");
        assert_eq!(h.rosenname, "");
        assert_eq!(h.gyousyaname, "");
        assert_eq!(h.zumennum, "");
    }

    #[test]
    fn test_csv_header_clone_eq() {
        let h1 = CsvHeader {
            koujiname: "test".to_string(),
            rosenname: "route".to_string(),
            gyousyaname: "builder".to_string(),
            zumennum: "1".to_string(),
        };
        let h2 = h1.clone();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_triangle_row_clone_eq() {
        let r1 = TriangleRow {
            number: 1, length_a: 6.0, length_b: 5.0, length_c: 4.0,
            parent_number: -1, connection_type: -1,
        };
        let r2 = r1.clone();
        assert_eq!(r1, r2);
    }

    // ================================================================
    // Only empty lines after header
    // ================================================================

    #[test]
    fn test_parse_only_empty_lines_after_header() {
        let csv = "\
koujiname, test
rosenname, test
gyousyaname, test
zumennum, 1


";
        let result = parse_csv(csv);
        assert!(result.is_err());
    }

    // ================================================================
    // Large number of triangles
    // ================================================================

    #[test]
    fn test_parse_many_triangles() {
        let mut csv = String::from("koujiname, test\nrosenname, test\ngyousyaname, test\nzumennum, 1\n");
        for i in 1..=100 {
            csv.push_str(&format!("{}, 6.0, 5.0, 4.0\n", i));
        }
        let result = parse_csv(&csv).unwrap();
        assert_eq!(result.triangles.len(), 100);
        assert_eq!(result.triangles[99].number, 100);
    }

    // ================================================================
    // FAILING: FULL format (28 columns) — fields beyond col 6 are silently ignored
    // The parser treats >=6 cols as CONN format, discarding cols 7-28.
    // These tests document the expected behavior for FULL format.
    // ================================================================

    #[test]
    fn test_parse_full_format_28_columns() {
        // FULL format: number, A, B, C, parent, conn_type, name, px1, py1, px2, py2, px3, py3,
        //   color, dimA_align, dimB_align, dimC_align, angle, ...
        // For now, at minimum the first 6 fields should parse correctly
        let csv = "\
koujiname, FULL形式テスト
rosenname, テスト路線
gyousyaname, テスト業者
zumennum, 1
1, 6.0, 5.0, 4.0, -1, -1, 三角形1, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
2, 5.0, 4.0, 3.0, 1, 1, 三角形2, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.triangles.len(), 2);
        assert_eq!(result.triangles[0].number, 1);
        assert_eq!(result.triangles[0].length_a, 6.0);
        assert_eq!(result.triangles[1].parent_number, 1);
        assert_eq!(result.triangles[1].connection_type, 1);
    }

    #[test]
    fn test_parse_scientific_notation_in_lengths() {
        let csv = "koujiname, test\nrosenname, test\ngyousyaname, test\nzumennum, 1\n\
1, 1.5e2, 1.0e2, 8.0e1\n";
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.triangles[0].length_a, 150.0);
        assert_eq!(result.triangles[0].length_b, 100.0);
        assert_eq!(result.triangles[0].length_c, 80.0);
    }

    #[test]
    fn test_parse_negative_zero_in_lengths() {
        let csv = "koujiname, test\nrosenname, test\ngyousyaname, test\nzumennum, 1\n\
1, -0.0, 5.0, 4.0\n";
        // -0.0 is a valid finite f64, should parse successfully
        let result = parse_csv(csv).unwrap();
        assert_eq!(result.triangles[0].length_a, 0.0);
    }

    #[test]
    fn test_parse_trailing_comma_creates_empty_col() {
        // Trailing comma means one extra empty column
        let csv = "koujiname, test\nrosenname, test\ngyousyaname, test\nzumennum, 1\n\
1, 6.0, 5.0, 4.0,\n";
        // 5 columns: should be treated as MIN+extra (col_count >= 4 but < 6)
        let result = parse_csv(csv);
        // Column 5 is empty string, parent_number parse will fail if >=6
        // With exactly 5 columns, code does (-1, -1) since col_count < 6
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_nan_rejected() {
        let csv = "koujiname, test\nrosenname, test\ngyousyaname, test\nzumennum, 1\n\
1, NaN, 5.0, 4.0\n";
        let result = parse_csv(csv);
        assert!(result.is_err(), "NaN should be rejected by parse_finite_f64");
    }

    #[test]
    fn test_parse_infinity_rejected() {
        let csv = "koujiname, test\nrosenname, test\ngyousyaname, test\nzumennum, 1\n\
1, inf, 5.0, 4.0\n";
        let result = parse_csv(csv);
        assert!(result.is_err(), "Infinity should be rejected by parse_finite_f64");
    }
}
