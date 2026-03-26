//! Cumulative / span distance conversion
//!
//! Ported from csv_to_dxf/src/processing.py:
//!   _is_cumulative(), to_cumulative()

const PITCH_M: f64 = 20.0;

/// Check if distance values are already cumulative.
///
/// Rules:
/// - Less than 4 values -> false (not enough data)
/// - Any diff <= 0 -> false (not monotonically increasing)
/// - Median of diffs < PITCH_M * 0.8 (16.0) -> true (cumulative)
pub fn is_cumulative(values: &[f64]) -> bool {
    if values.len() < 4 {
        return false;
    }
    let diffs: Vec<f64> = values.windows(2).map(|w| w[1] - w[0]).collect();
    if diffs.iter().any(|&d| d <= 0.0) {
        return false;
    }
    median(&diffs) < PITCH_M * 0.8
}

/// Convert single-segment distances to cumulative.
///
/// If already cumulative, return as-is.
/// Otherwise, compute running sum (cumsum).
pub fn to_cumulative(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return vec![];
    }
    if is_cumulative(values) {
        return values.to_vec();
    }
    let mut result = Vec::with_capacity(values.len());
    let mut acc = 0.0;
    for &v in values {
        acc += v;
        result.push(acc);
    }
    result
}

/// Convert x values to cumulative in-place on RawRow slices.
///
/// If already cumulative, returns rows unchanged.
pub fn to_cumulative_rows(rows: &mut [crate::RawRow]) {
    let values: Vec<f64> = rows.iter().map(|r| r.x).collect();
    let result = to_cumulative(&values);
    for (row, &val) in rows.iter_mut().zip(result.iter()) {
        row.x = val;
    }
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cumulative_true() {
        // 区間1.csv: 0.00, 1.15, 3.25, 5.40, 7.00
        // diffs: 1.15, 2.10, 2.15, 1.60 → median ~2.1 < 16 → cumulative
        let vals = vec![0.0, 1.15, 3.25, 5.40, 7.00];
        assert!(is_cumulative(&vals));
    }

    #[test]
    fn test_is_cumulative_false_single_segments() {
        // Single segments: each value is the distance of that segment only
        // e.g., 10, 20, 10, 15 (not monotonically increasing)
        let vals = vec![10.0, 20.0, 10.0, 15.0];
        assert!(!is_cumulative(&vals));
    }

    #[test]
    fn test_is_cumulative_too_few() {
        let vals = vec![0.0, 10.0, 20.0];
        assert!(!is_cumulative(&vals));
    }

    #[test]
    fn test_is_cumulative_large_diffs() {
        // Large diffs (>16m each) → not cumulative even if monotonic
        // This would be single segment distances that happen to be increasing
        let vals = vec![0.0, 20.0, 40.0, 60.0, 80.0];
        // diffs: 20, 20, 20, 20 → median = 20 >= 16 → NOT cumulative
        assert!(!is_cumulative(&vals));
    }

    #[test]
    fn test_to_cumulative_already_cumulative() {
        let vals = vec![0.0, 1.15, 3.25, 5.40, 7.00];
        let result = to_cumulative(&vals);
        assert_eq!(result, vals); // unchanged
    }

    #[test]
    fn test_to_cumulative_single_segments() {
        let vals = vec![10.0, 20.0, 10.0, 15.0];
        let result = to_cumulative(&vals);
        assert!((result[0] - 10.0).abs() < 0.001);
        assert!((result[1] - 30.0).abs() < 0.001);
        assert!((result[2] - 40.0).abs() < 0.001);
        assert!((result[3] - 55.0).abs() < 0.001);
    }

    #[test]
    fn test_to_cumulative_empty() {
        let vals: Vec<f64> = vec![];
        let result = to_cumulative(&vals);
        assert!(result.is_empty());
    }

    #[test]
    fn test_median_boundary() {
        // Exactly at boundary: median = 16.0, should NOT be cumulative
        // (Python uses strict <, not <=)
        let vals = vec![0.0, 16.0, 32.0, 48.0, 64.0];
        // diffs: 16, 16, 16, 16 → median = 16 → NOT < 16 → not cumulative
        assert!(!is_cumulative(&vals));
    }

    // ================================================================
    // is_cumulative: 1 and 2 values
    // ================================================================

    #[test]
    fn test_is_cumulative_one_value() {
        assert!(!is_cumulative(&[5.0]));
    }

    #[test]
    fn test_is_cumulative_two_values() {
        assert!(!is_cumulative(&[0.0, 1.0]));
    }

    // ================================================================
    // is_cumulative: decreasing values
    // ================================================================

    #[test]
    fn test_is_cumulative_decreasing() {
        let vals = vec![100.0, 80.0, 60.0, 40.0, 20.0];
        assert!(!is_cumulative(&vals), "Decreasing values are not cumulative");
    }

    // ================================================================
    // is_cumulative: just above boundary (15.9 < 16)
    // ================================================================

    #[test]
    fn test_is_cumulative_just_below_boundary() {
        // Median diff = 15.9 < 16 → cumulative
        let vals = vec![0.0, 15.9, 31.8, 47.7, 63.6];
        assert!(is_cumulative(&vals));
    }

    // ================================================================
    // to_cumulative: single value
    // ================================================================

    #[test]
    fn test_to_cumulative_single_value() {
        let vals = vec![42.0];
        let result = to_cumulative(&vals);
        assert_eq!(result, vec![42.0]);
    }

    // ================================================================
    // to_cumulative_rows
    // ================================================================

    #[test]
    fn test_to_cumulative_rows_span() {
        use crate::RawRow;
        let mut rows = vec![
            RawRow { name: "A".into(), x: 5.0, wl: 1.0, wr: 1.0 },
            RawRow { name: "B".into(), x: 10.0, wl: 1.0, wr: 1.0 },
            RawRow { name: "C".into(), x: 5.0, wl: 1.0, wr: 1.0 },
        ];
        to_cumulative_rows(&mut rows);
        // Not cumulative (not monotonic, < 4 values) → cumsum
        assert!((rows[0].x - 5.0).abs() < 1e-9);
        assert!((rows[1].x - 15.0).abs() < 1e-9);
        assert!((rows[2].x - 20.0).abs() < 1e-9);
    }

    #[test]
    fn test_to_cumulative_rows_already_cumulative() {
        use crate::RawRow;
        let mut rows = vec![
            RawRow { name: "A".into(), x: 0.0, wl: 1.0, wr: 1.0 },
            RawRow { name: "B".into(), x: 1.0, wl: 1.0, wr: 1.0 },
            RawRow { name: "C".into(), x: 3.0, wl: 1.0, wr: 1.0 },
            RawRow { name: "D".into(), x: 5.0, wl: 1.0, wr: 1.0 },
            RawRow { name: "E".into(), x: 7.0, wl: 1.0, wr: 1.0 },
        ];
        to_cumulative_rows(&mut rows);
        // Already cumulative (median diff = 2 < 16) → unchanged
        assert!((rows[4].x - 7.0).abs() < 1e-9);
    }

    // ================================================================
    // median edge case: even number of elements
    // ================================================================

    #[test]
    fn test_is_cumulative_even_diffs() {
        // 5 values → 4 diffs (even)
        let vals = vec![0.0, 2.0, 5.0, 9.0, 14.0];
        // diffs: 2, 3, 4, 5 → sorted: 2, 3, 4, 5 → median = (3+4)/2 = 3.5 < 16
        assert!(is_cumulative(&vals));
    }
}
