//! Integration tests: real CSV files → parse → build → verify
//!
//! Reads test fixtures from the original Kotlin trianglelist project:
//!   ~/StudioProjects/trianglelist/app/src/test/resources/

use triangle_core::connection::{build_connected_list, build_connected_list_lenient, verify_connection};
use triangle_core::csv_loader::parse_csv;

/// Path to Kotlin project test resources
const RESOURCE_DIR: &str = concat!(
    env!("USERPROFILE"),
    r"\StudioProjects\trianglelist\app\src\test\resources"
);

fn read_fixture(name: &str) -> String {
    let path = format!(r"{}\{}", RESOURCE_DIR, name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e))
}

// ================================================================
// minimal.csv: 3 independent triangles (MIN format, 4 columns)
// ================================================================

#[test]
fn test_integ_minimal_csv_parse() {
    let text = read_fixture("minimal.csv");
    let parsed = parse_csv(&text).unwrap();

    assert_eq!(parsed.header.koujiname, "最小形式テスト");
    assert_eq!(parsed.header.rosenname, "テスト路線");
    assert_eq!(parsed.triangles.len(), 3);
}

#[test]
fn test_integ_minimal_csv_all_independent() {
    let text = read_fixture("minimal.csv");
    let parsed = parse_csv(&text).unwrap();

    for t in &parsed.triangles {
        assert_eq!(t.parent_number, -1, "MIN format: all triangles independent");
        assert_eq!(t.connection_type, -1);
    }
}

#[test]
fn test_integ_minimal_csv_side_lengths() {
    let text = read_fixture("minimal.csv");
    let parsed = parse_csv(&text).unwrap();

    // Triangle 1: 6.0, 5.0, 4.0
    assert!((parsed.triangles[0].length_a - 6.0).abs() < 0.001);
    assert!((parsed.triangles[0].length_b - 5.0).abs() < 0.001);
    assert!((parsed.triangles[0].length_c - 4.0).abs() < 0.001);

    // Triangle 2: 5.5, 4.5, 3.5
    assert!((parsed.triangles[1].length_a - 5.5).abs() < 0.001);
    assert!((parsed.triangles[1].length_b - 4.5).abs() < 0.001);
    assert!((parsed.triangles[1].length_c - 3.5).abs() < 0.001);

    // Triangle 3: 4.0, 3.5, 3.0
    assert!((parsed.triangles[2].length_a - 4.0).abs() < 0.001);
    assert!((parsed.triangles[2].length_b - 3.5).abs() < 0.001);
    assert!((parsed.triangles[2].length_c - 3.0).abs() < 0.001);
}

#[test]
fn test_integ_minimal_csv_areas() {
    let text = read_fixture("minimal.csv");
    let parsed = parse_csv(&text).unwrap();

    let rows: Vec<_> = parsed.triangles.iter().map(|t| {
        (t.length_a, t.length_b, t.length_c, t.parent_number, t.connection_type)
    }).collect();

    let triangles = build_connected_list(&rows).unwrap();

    // All should have positive area (valid triangles)
    for (i, t) in triangles.iter().enumerate() {
        assert!(t.is_valid(), "Triangle {} should be valid", i + 1);
        assert!(t.area() > 0.0, "Triangle {} area should be > 0, got {}", i + 1, t.area());
    }

    // Heron's formula: Triangle(6,5,4) → s=7.5, area=sqrt(7.5*1.5*2.5*3.5)≈9.92
    assert!((triangles[0].area() - 9.92).abs() < 0.01,
        "Triangle 1 area: {} vs 9.92", triangles[0].area());
}

// ================================================================
// connected.csv: 7 triangles with parent-child connections (CONN format)
// ================================================================

#[test]
fn test_integ_connected_csv_parse() {
    let text = read_fixture("connected.csv");
    let parsed = parse_csv(&text).unwrap();

    assert_eq!(parsed.header.koujiname, "接続形式テスト");
    assert_eq!(parsed.triangles.len(), 7);
}

#[test]
fn test_integ_connected_csv_connection_fields() {
    let text = read_fixture("connected.csv");
    let parsed = parse_csv(&text).unwrap();

    // Triangle 1: independent
    assert_eq!(parsed.triangles[0].parent_number, -1);
    assert_eq!(parsed.triangles[0].connection_type, -1);

    // Triangle 2: parent 1, type 1 (B-edge)
    assert_eq!(parsed.triangles[1].parent_number, 1);
    assert_eq!(parsed.triangles[1].connection_type, 1);

    // Triangle 3: parent 1, type 2 (C-edge)
    assert_eq!(parsed.triangles[2].parent_number, 1);
    assert_eq!(parsed.triangles[2].connection_type, 2);

    // Triangles 4-7: deeper connections
    assert_eq!(parsed.triangles[3].parent_number, 2);
    assert_eq!(parsed.triangles[4].parent_number, 2);
    assert_eq!(parsed.triangles[5].parent_number, 3);
    assert_eq!(parsed.triangles[6].parent_number, 3);
}

#[test]
fn test_integ_connected_csv_edge_length_consistency() {
    let text = read_fixture("connected.csv");
    let parsed = parse_csv(&text).unwrap();
    let ts = &parsed.triangles;

    // child.A == parent's connection edge
    // T2(A=5.0) on T1.B(5.0)
    assert!((ts[1].length_a - ts[0].length_b).abs() < 0.001);
    // T3(A=4.0) on T1.C(4.0)
    assert!((ts[2].length_a - ts[0].length_c).abs() < 0.001);
    // T4(A=4.0) on T2.B(4.0)
    assert!((ts[3].length_a - ts[1].length_b).abs() < 0.001);
    // T5(A=3.0) on T2.C(3.0)
    assert!((ts[4].length_a - ts[1].length_c).abs() < 0.001);
    // T6(A=3.5) on T3.B(3.5)
    assert!((ts[5].length_a - ts[2].length_b).abs() < 0.001);
    // T7(A=3.0) on T3.C(3.0)
    assert!((ts[6].length_a - ts[2].length_c).abs() < 0.001);
}

#[test]
fn test_integ_connected_csv_build_and_verify() {
    let text = read_fixture("connected.csv");
    let parsed = parse_csv(&text).unwrap();

    let rows: Vec<_> = parsed.triangles.iter().map(|t| {
        (t.length_a, t.length_b, t.length_c, t.parent_number, t.connection_type)
    }).collect();

    let triangles = build_connected_list(&rows).unwrap();
    assert_eq!(triangles.len(), 7);

    // Verify all 6 parent-child connections
    assert!(verify_connection(&triangles[0], &triangles[1], 1), "T2→T1 B-edge");
    assert!(verify_connection(&triangles[0], &triangles[2], 2), "T3→T1 C-edge");
    assert!(verify_connection(&triangles[1], &triangles[3], 1), "T4→T2 B-edge");
    assert!(verify_connection(&triangles[1], &triangles[4], 2), "T5→T2 C-edge");
    assert!(verify_connection(&triangles[2], &triangles[5], 1), "T6→T3 B-edge");
    assert!(verify_connection(&triangles[2], &triangles[6], 2), "T7→T3 C-edge");
}

#[test]
fn test_integ_connected_csv_all_valid_areas() {
    let text = read_fixture("connected.csv");
    let parsed = parse_csv(&text).unwrap();

    let rows: Vec<_> = parsed.triangles.iter().map(|t| {
        (t.length_a, t.length_b, t.length_c, t.parent_number, t.connection_type)
    }).collect();

    let triangles = build_connected_list(&rows).unwrap();

    for (i, t) in triangles.iter().enumerate() {
        assert!(t.is_valid(), "Triangle {} should be valid", i + 1);
        assert!(t.area() > 0.0, "Triangle {} area should be > 0, got {}", i + 1, t.area());
    }

    // Total area = sum of all 7 triangles
    let total: f64 = triangles.iter().map(|t| t.area()).sum();
    assert!(total > 0.0, "Total area should be positive: {}", total);
}

#[test]
fn test_integ_connected_csv_vertex_precision() {
    let text = read_fixture("connected.csv");
    let parsed = parse_csv(&text).unwrap();

    let rows: Vec<_> = parsed.triangles.iter().map(|t| {
        (t.length_a, t.length_b, t.length_c, t.parent_number, t.connection_type)
    }).collect();

    let triangles = build_connected_list(&rows).unwrap();

    // Verify vertex distances match side lengths for every triangle (cumulative error < 0.01)
    for (i, t) in triangles.iter().enumerate() {
        let ca_ab = t.point_ca().distance_to(t.point_ab());
        let ab_bc = t.point_ab().distance_to(t.point_bc());
        let bc_ca = t.point_bc().distance_to(t.point_ca());

        assert!((ca_ab - t.lengths[0]).abs() < 0.01,
            "T{} CA→AB: {} vs {}", i + 1, ca_ab, t.lengths[0]);
        assert!((ab_bc - t.lengths[1]).abs() < 0.01,
            "T{} AB→BC: {} vs {}", i + 1, ab_bc, t.lengths[1]);
        assert!((bc_ca - t.lengths[2]).abs() < 0.01,
            "T{} BC→CA: {} vs {}", i + 1, bc_ca, t.lengths[2]);
    }
}

// ================================================================
// 4.11.csv: 30 triangles, FULL format (28 columns), with trailing
// metadata (ListAngle, ListScale, TextSize) and Deduction rows.
// Triangle 22 has connection_type=4 in col5, which is a FULL format
// extended connection type (uses ConnParam side/type/lcr from cols 17-19).
// build_connected_list only handles type 1/2, so we test parsing
// and validate standard connections separately.
// ================================================================

#[test]
fn test_integ_411_csv_parse() {
    let text = read_fixture("4.11.csv");
    let parsed = parse_csv(&text).unwrap();

    assert_eq!(parsed.header.rosenname, "新規路線");
    assert_eq!(parsed.triangles.len(), 30, "Should parse exactly 30 triangles");
}

#[test]
fn test_integ_411_csv_skips_metadata_rows() {
    // ListAngle, ListScale, TextSize, Deduction lines must be skipped
    let text = read_fixture("4.11.csv");
    let parsed = parse_csv(&text).unwrap();

    // Only triangle rows (number 1-30), no metadata/deduction
    assert_eq!(parsed.triangles.len(), 30);
    assert_eq!(parsed.triangles[0].number, 1);
    assert_eq!(parsed.triangles[29].number, 30);
}

#[test]
fn test_integ_411_csv_empty_header_fields() {
    let text = read_fixture("4.11.csv");
    let parsed = parse_csv(&text).unwrap();

    // koujiname, gyousyaname, zumennum are empty in this file
    assert_eq!(parsed.header.koujiname, "");
    assert_eq!(parsed.header.gyousyaname, "");
    assert_eq!(parsed.header.zumennum, "");
    assert_eq!(parsed.header.rosenname, "新規路線");
}

#[test]
fn test_integ_411_csv_triangle_1_independent() {
    let text = read_fixture("4.11.csv");
    let parsed = parse_csv(&text).unwrap();
    let t1 = &parsed.triangles[0];

    assert_eq!(t1.number, 1);
    assert!((t1.length_a - 5.45).abs() < 0.001);
    assert!((t1.length_b - 7.0).abs() < 0.001);
    assert!((t1.length_c - 4.08).abs() < 0.001);
    assert_eq!(t1.parent_number, -1);
    assert_eq!(t1.connection_type, -1);
}

#[test]
fn test_integ_411_csv_triangle_30_last() {
    let text = read_fixture("4.11.csv");
    let parsed = parse_csv(&text).unwrap();
    let t30 = &parsed.triangles[29];

    assert_eq!(t30.number, 30);
    assert!((t30.length_a - 8.9).abs() < 0.001);
    assert!((t30.length_b - 7.14).abs() < 0.001);
    assert!((t30.length_c - 5.4).abs() < 0.001);
    assert_eq!(t30.parent_number, 29);
}

#[test]
fn test_integ_411_csv_connection_type4_in_col5() {
    // Triangle 22 has connectionType=4 in FULL format col5.
    // This is an extended type (ConnParam from cols 17-19).
    // Parser normalizes non-standard types: 4 → 1 (B-edge fallback)
    let text = read_fixture("4.11.csv");
    let parsed = parse_csv(&text).unwrap();
    let t22 = &parsed.triangles[21]; // 0-indexed

    assert_eq!(t22.number, 22);
    assert_eq!(t22.parent_number, 21);
    // Type=4 is not standard 1/2; parser normalizes to 1
    assert_eq!(t22.connection_type, 1);
}

#[test]
fn test_integ_411_csv_standard_edge_length_consistency() {
    // For standard connections (type 1/2), child.A should match parent edge
    let text = read_fixture("4.11.csv");
    let parsed = parse_csv(&text).unwrap();
    let ts = &parsed.triangles;

    // Check all standard type 1/2 connections (skip T22 which is extended type=4)
    for (i, t) in ts.iter().enumerate() {
        if t.parent_number == -1 || t.number == 22 { continue; }
        let parent = &ts[(t.parent_number - 1) as usize];
        let parent_edge = match t.connection_type {
            1 => parent.length_b,
            2 => parent.length_c,
            _ => continue,
        };
        assert!((t.length_a - parent_edge).abs() < 0.01,
            "T{}: A={} should match parent T{} edge (type={}) = {}",
            t.number, t.length_a, t.parent_number, t.connection_type, parent_edge);
    }
}

#[test]
fn test_integ_411_csv_all_triangles_valid() {
    // All 30 parsed triangles should satisfy triangle inequality
    let text = read_fixture("4.11.csv");
    let parsed = parse_csv(&text).unwrap();

    use triangle_core::triangle::Triangle;

    for row in &parsed.triangles {
        let t = Triangle::new(row.length_a, row.length_b, row.length_c);
        assert!(t.is_valid(),
            "T{} ({}, {}, {}) should be valid",
            row.number, row.length_a, row.length_b, row.length_c);
        assert!(t.area() > 0.0,
            "T{} area should be > 0, got {}", row.number, t.area());
    }
}

#[test]
fn test_integ_411_csv_total_area() {
    let text = read_fixture("4.11.csv");
    let parsed = parse_csv(&text).unwrap();

    use triangle_core::triangle::Triangle;

    let total: f64 = parsed.triangles.iter().map(|row| {
        Triangle::new(row.length_a, row.length_b, row.length_c).area()
    }).sum();

    // 30 triangles, total area should be substantial
    assert!(total > 50.0, "Total area of 30 triangles should be > 50, got {}", total);
}
