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
}
