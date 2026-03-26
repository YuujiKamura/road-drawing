//! Station name generation and completion
//!
//! Ported from csv_to_dxf/src/station_name_utils.py:
//!   fill_station_names(), _parse(), _name_from_dist()

use regex::Regex;

use crate::{RawRow, SPAN};

/// Parse station name into (main, sub) parts.
///
/// # Examples
/// - `"No.0"` -> `Some((0, 0.0))`
/// - `"0+10.5"` -> `Some((0, 10.5))`
/// - `"No.3"` -> `Some((3, 0.0))`
/// - `"3+5"` -> `Some((3, 5.0))`
/// - `"10m"` -> `None`
pub fn parse_station_name(name: &str) -> Option<(i32, f64)> {
    let re = Regex::new(r"^(?:No\.)?(\d+)(?:\+(\d+\.?\d*))?$").unwrap();
    let caps = re.captures(name)?;
    let main: i32 = caps.get(1)?.as_str().parse().ok()?;
    let sub: f64 = caps
        .get(2)
        .map_or(0.0, |m| m.as_str().parse().unwrap_or(0.0));
    Some((main, sub))
}

/// Generate station name from cumulative distance.
///
/// Main stations (sub==0) get "No." prefix; sub-stations use "main+sub" format.
/// Sub-distance is rounded to 0.1m. Integer subs are formatted without decimals.
///
/// # Examples
/// - `0.0` -> `"No.0"`
/// - `10.5` -> `"0+10.5"`
/// - `20.0` -> `"No.1"`
/// - `30.0` -> `"1+10"`
/// - `45.3` -> `"2+5.3"`
pub fn name_from_distance(dist: f64) -> String {
    let main = (dist / SPAN).floor() as i32;
    let sub = round_to_1(dist % SPAN);

    if sub.abs() < 1e-9 {
        return format!("No.{main}");
    }

    let sub_str = if (sub - sub.round()).abs() < 1e-9 {
        format!("{}", sub as i64)
    } else {
        format_sub(sub)
    };
    format!("{main}+{sub_str}")
}

/// Fill missing station names in a list of (name, x) pairs.
///
/// Preserves existing valid names and uses them as offset origins.
/// When an existing station name is encountered, the offset is recalculated
/// so that subsequent auto-generated names are relative to that station.
///
/// # Arguments
/// - `rows` - slice of `(name, x)` pairs where name may be empty
///
/// # Returns
/// A `Vec<String>` of filled station names, one per input row.
pub fn fill_station_names_pairs(rows: &[(String, f64)]) -> Vec<String> {
    let mut result: Vec<String> = rows.iter().map(|(name, _)| name.clone()).collect();
    let original_valid: Vec<bool> = rows
        .iter()
        .map(|(name, _)| parse_station_name(name).is_some())
        .collect();

    // x=0 with empty name -> No.0
    for (i, (name, x)) in rows.iter().enumerate() {
        if *x == 0.0 && name.is_empty() {
            result[i] = "No.0".to_string();
        }
    }

    let mut offset: f64 = 0.0;

    for i in 0..rows.len() {
        let x = rows[i].1;

        if original_valid[i] {
            if let Some((main, sub)) = parse_station_name(&rows[i].0) {
                offset = (main as f64) * SPAN + sub - x;
                // Normalize: main station without "No." prefix
                if sub == 0.0 && !rows[i].0.starts_with("No.") {
                    result[i] = format!("No.{main}");
                }
            }
            continue;
        }

        // Already filled (from x=0 check above)
        if parse_station_name(&result[i]).is_some() {
            continue;
        }

        // Auto-generate
        let cum = x + offset;
        result[i] = name_from_distance(cum);
    }

    result
}

/// Round to 0.1m (matching Python Decimal('0.1') ROUND_HALF_UP)
fn round_to_1(val: f64) -> f64 {
    (val * 10.0).round() / 10.0
}

/// Format sub-distance: strip trailing zeros but keep at least one decimal
fn format_sub(val: f64) -> String {
    let s = format!("{:.1}", val);
    // If more precision needed (e.g., 14.55), use more decimals
    if (val * 10.0 - (val * 10.0).round()).abs() > 1e-6 {
        let s2 = format!("{:.2}", val);
        return s2.trim_end_matches('0').to_string();
    }
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Fill missing station names in RawRow slices (used by transform pipeline).
///
/// Mutates rows in place. Delegates to the same logic as `fill_station_names_pairs`.
pub fn fill_station_names(rows: &mut [RawRow]) {
    let pairs: Vec<(String, f64)> = rows.iter().map(|r| (r.name.clone(), r.x)).collect();
    let names = fill_station_names_pairs(&pairs);
    for (row, name) in rows.iter_mut().zip(names.into_iter()) {
        row.name = name;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_station_name tests ──

    #[test]
    fn test_parse_no_prefix() {
        assert_eq!(parse_station_name("No.0"), Some((0, 0.0)));
        assert_eq!(parse_station_name("No.3"), Some((3, 0.0)));
    }

    #[test]
    fn test_parse_with_sub() {
        assert_eq!(parse_station_name("0+10.5"), Some((0, 10.5)));
        assert_eq!(parse_station_name("3+5"), Some((3, 5.0)));
    }

    #[test]
    fn test_parse_no_prefix_with_sub() {
        assert_eq!(parse_station_name("No.0+9.9"), Some((0, 9.9)));
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(parse_station_name("10m"), None);
        assert_eq!(parse_station_name(""), None);
        assert_eq!(parse_station_name("abc"), None);
    }

    // ── name_from_distance tests ──

    #[test]
    fn test_name_main_station() {
        assert_eq!(name_from_distance(0.0), "No.0");
        assert_eq!(name_from_distance(20.0), "No.1");
        assert_eq!(name_from_distance(40.0), "No.2");
    }

    #[test]
    fn test_name_sub_station() {
        assert_eq!(name_from_distance(10.0), "0+10");
        assert_eq!(name_from_distance(10.5), "0+10.5");
        assert_eq!(name_from_distance(30.0), "1+10");
        assert_eq!(name_from_distance(45.3), "2+5.3");
    }

    // ── fill_station_names_pairs tests ──

    #[test]
    fn test_fill_simple() {
        let rows = vec![
            ("".to_string(), 0.0),
            ("".to_string(), 10.0),
            ("".to_string(), 20.0),
        ];
        let names = fill_station_names_pairs(&rows);
        assert_eq!(names, vec!["No.0", "0+10", "No.1"]);
    }

    #[test]
    fn test_fill_preserves_existing() {
        let rows = vec![
            ("No.0".to_string(), 0.0),
            ("0+1.2".to_string(), 1.15),
            ("0+3.2".to_string(), 3.25),
        ];
        let names = fill_station_names_pairs(&rows);
        assert_eq!(names[0], "No.0");
        assert_eq!(names[1], "0+1.2");
        assert_eq!(names[2], "0+3.2");
    }

    #[test]
    fn test_fill_with_offset_reset() {
        let rows = vec![
            ("".to_string(), 0.0),
            ("".to_string(), 10.0),
            ("No.5".to_string(), 15.0), // offset = 5*20+0 - 15 = 85
            ("".to_string(), 25.0),     // cum = 25 + 85 = 110 -> No.5+10
        ];
        let names = fill_station_names_pairs(&rows);
        assert_eq!(names[0], "No.0");
        assert_eq!(names[1], "0+10");
        assert_eq!(names[2], "No.5");
        assert_eq!(names[3], "5+10");
    }

    // ── Legacy RawRow-based tests ──

    #[test]
    fn test_fill_rawrow_basic() {
        let mut rows = vec![
            RawRow { name: "No.0".into(), x: 0.0, wl: 0.8, wr: 0.0 },
            RawRow { name: "".into(), x: 1.15, wl: 0.63, wr: 0.0 },
            RawRow { name: "".into(), x: 3.25, wl: 0.50, wr: 0.0 },
            RawRow { name: "0+7".into(), x: 7.0, wl: 0.50, wr: 0.0 },
        ];
        fill_station_names(&mut rows);
        assert_eq!(rows[0].name, "No.0");
        assert_eq!(rows[1].name, "0+1.2");
        assert_eq!(rows[3].name, "0+7");
    }

    #[test]
    fn test_fill_rawrow_with_offset() {
        let mut rows = vec![
            RawRow { name: "0+7.9".into(), x: 0.0, wl: 0.5, wr: 0.0 },
            RawRow { name: "".into(), x: 2.03, wl: 0.5, wr: 0.0 },
        ];
        fill_station_names(&mut rows);
        assert_eq!(rows[0].name, "0+7.9");
        assert_eq!(rows[1].name, "0+9.9");
    }

    #[test]
    fn test_fill_rawrow_empty_x0() {
        let mut rows = vec![
            RawRow { name: "".into(), x: 0.0, wl: 1.0, wr: 0.0 },
        ];
        fill_station_names(&mut rows);
        assert_eq!(rows[0].name, "No.0");
    }
}
