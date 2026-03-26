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
        let length_a: f64 = parts[1].parse().map_err(|_| CsvError::InvalidNumber {
            row: row_num, col: "length_a", value: parts[1].to_string(),
        })?;
        let length_b: f64 = parts[2].parse().map_err(|_| CsvError::InvalidNumber {
            row: row_num, col: "length_b", value: parts[2].to_string(),
        })?;
        let length_c: f64 = parts[3].parse().map_err(|_| CsvError::InvalidNumber {
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
}
