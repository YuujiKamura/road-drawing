//! Stress test: 100 sections × 50 stations each (5000 total rows)
//!
//! Verifies that section detection, extraction, and transformation
//! handle large CSV data without panics or data loss.

use excel_parser::transform::{extract_and_transform_text, list_sections_text};
use excel_parser::section_detector::{get_available_sections, extract_section_data};

/// Generate a CSV string with `n_sections` sections, each with `n_stations` rows.
/// Uses headerless 4-column format: name, span_distance, wl, wr
/// Distances are span values (all 20.0) — the transform pipeline converts to cumulative.
fn generate_large_csv(n_sections: usize, n_stations: usize) -> String {
    let mut csv = String::with_capacity(n_sections * n_stations * 40);
    for s in 1..=n_sections {
        csv.push_str(&format!("区間{},台形計算\n", s));
        for st in 0..n_stations {
            let name = format!("NO.{}", st);
            let span = 20.0; // span distance (not cumulative)
            let wl = 3.0 + (st % 5) as f64 * 0.1;
            let wr = 3.5 + (st % 3) as f64 * 0.1;
            csv.push_str(&format!("{},{:.1},{:.2},{:.2}\n", name, span, wl, wr));
        }
    }
    csv
}

// ================================================================
// Section detection at scale
// ================================================================

#[test]
fn test_stress_detect_100_sections() {
    let csv = generate_large_csv(100, 50);
    let sections = get_available_sections(&csv, "data.csv");
    assert_eq!(sections.len(), 100, "Should detect all 100 sections");
    assert_eq!(sections[0], "区間1");
    assert_eq!(sections[99], "区間100");
}

#[test]
fn test_stress_list_sections_text_100() {
    let csv = generate_large_csv(100, 50);
    let sections = list_sections_text(&csv, "data.csv");
    assert_eq!(sections.len(), 100);
}

// ================================================================
// Section extraction at scale
// ================================================================

#[test]
fn test_stress_extract_first_section() {
    let csv = generate_large_csv(100, 50);
    let rows = extract_section_data(&csv, "区間1").unwrap();
    assert_eq!(rows.len(), 50, "First section should have 50 rows");
    assert_eq!(rows[0].name, "NO.0");
    assert_eq!(rows[49].name, "NO.49");
}

#[test]
fn test_stress_extract_last_section() {
    let csv = generate_large_csv(100, 50);
    let rows = extract_section_data(&csv, "区間100").unwrap();
    assert_eq!(rows.len(), 50, "Last section should have 50 rows");
}

#[test]
fn test_stress_extract_middle_section() {
    let csv = generate_large_csv(100, 50);
    let rows = extract_section_data(&csv, "区間50").unwrap();
    assert_eq!(rows.len(), 50, "Middle section should have 50 rows");
}

#[test]
fn test_stress_extract_all_100_sections() {
    let csv = generate_large_csv(100, 50);
    let sections = get_available_sections(&csv, "data.csv");
    for section_name in &sections {
        let rows = extract_section_data(&csv, section_name).unwrap();
        assert_eq!(rows.len(), 50,
            "Section {} should have 50 rows, got {}", section_name, rows.len());
    }
}

// ================================================================
// Transform pipeline at scale
// ================================================================

#[test]
fn test_stress_transform_section() {
    let csv = generate_large_csv(100, 50);
    let rows = extract_and_transform_text(&csv, "区間1").unwrap();
    assert_eq!(rows.len(), 50);

    // Distances should be cumulative: span 20.0 each → 20, 40, 60, ..., 1000
    for (i, row) in rows.iter().enumerate() {
        let expected_dist = (i + 1) as f64 * 20.0;
        assert!((row.x - expected_dist).abs() < 0.01,
            "Row {}: distance {} vs expected {}", i, row.x, expected_dist);
    }

    // Width values should be preserved (headerless: col2=wl, col3=wr)
    // Station 0: wl = 3.0 + (0%5)*0.1 = 3.0
    assert!((rows[0].wl - 3.0).abs() < 0.01, "wl[0]={}", rows[0].wl);
}

#[test]
fn test_stress_transform_all_100_sections() {
    let csv = generate_large_csv(100, 50);
    let sections = get_available_sections(&csv, "data.csv");

    let mut total_rows = 0;
    for section_name in &sections {
        let rows = extract_and_transform_text(&csv, section_name).unwrap();
        assert_eq!(rows.len(), 50);
        total_rows += rows.len();
    }
    assert_eq!(total_rows, 5000, "Total rows across all sections");
}

// ================================================================
// Data integrity across sections
// ================================================================

#[test]
fn test_stress_sections_are_isolated() {
    // Each section's data must not leak into adjacent sections
    let csv = generate_large_csv(100, 50);

    let rows_1 = extract_section_data(&csv, "区間1").unwrap();
    let rows_2 = extract_section_data(&csv, "区間2").unwrap();

    // Both should have exactly 50 rows
    assert_eq!(rows_1.len(), 50);
    assert_eq!(rows_2.len(), 50);

    // First row of each should be NO.0, not carryover from previous section
    assert_eq!(rows_1[0].name, "NO.0");
    assert_eq!(rows_2[0].name, "NO.0");
}

#[test]
fn test_stress_width_values_preserved() {
    let csv = generate_large_csv(100, 50);

    // Headerless 4-col format: col0=name, col1=x, col2=wl, col3=wr
    // Station 25: wl = 3.0 + (25%5)*0.1 = 3.0, wr = 3.5 + (25%3)*0.1 = 3.6
    let rows = extract_section_data(&csv, "区間50").unwrap();
    let r25 = &rows[25];
    assert!((r25.wl - 3.0).abs() < 0.01, "wl={}", r25.wl);
    assert!((r25.wr - 3.6).abs() < 0.01, "wr={}", r25.wr);
}
