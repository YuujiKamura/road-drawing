//! CSV → DXF → Viewer pipeline end-to-end tests (Issue #9).
//!
//! Verifies the complete data flow:
//! 1. CSV text → StationData parse
//! 2. StationData → RoadSectionGeometry
//! 3. Geometry → DXF string export
//! 4. DXF string → DxfDocument parse
//! 5. DxfDocument → BBox / viewport valid for rendering
//!
//! Also tests file-based pipeline: CSV file → DXF file → reload.

use road_section::{
    calculate_road_section, parse_road_section_csv, RoadSectionConfig, StationData,
};
use road_drawing_web::dxf_export::{geometry_to_dxf_string, stations_to_dxf};

// ================================================================
// In-memory pipeline: CSV text → geometry → DXF → parse → verify
// ================================================================

#[test]
fn test_e2e_csv_to_dxf_roundtrip_simple() {
    let csv = "No.0,0.0,2.5,2.5\nNo.1,20.0,2.5,2.5\n";
    let stations = parse_road_section_csv(csv).unwrap();
    assert_eq!(stations.len(), 2);

    let dxf = stations_to_dxf(&stations);
    assert!(dxf_engine::DxfLinter::is_valid(&dxf), "DXF must pass linter");

    let doc = dxf_engine::parse_dxf(&dxf).unwrap();
    assert_eq!(doc.lines.len(), 7, "2 stations: 4 width + 3 connecting = 7 lines");
    assert!(doc.texts.iter().any(|t| t.text == "No.0"));
    assert!(doc.texts.iter().any(|t| t.text == "No.1"));
}

#[test]
fn test_e2e_csv_to_dxf_roundtrip_real_fixture() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures/road-section/data.csv");
    let csv = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Fixture {:?} should exist: {e}", fixture_path));
    let stations = parse_road_section_csv(&csv).unwrap();
    assert!(stations.len() > 10, "Real fixture should have many stations");

    let dxf = stations_to_dxf(&stations);
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));

    let doc = dxf_engine::parse_dxf(&dxf).unwrap();
    assert!(!doc.lines.is_empty());
    assert!(!doc.texts.is_empty());

    // Station names should survive the roundtrip
    let names: Vec<&str> = doc.texts.iter().map(|t| t.text.as_str()).collect();
    assert!(names.contains(&"No.0"), "Should contain first station name");
}

#[test]
fn test_e2e_csv_to_geometry_to_viewport_valid() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.0,3.0\nNo.2,40.0,3.0,3.0\n";
    let stations = parse_road_section_csv(csv).unwrap();
    let config = RoadSectionConfig::default();
    let geometry = calculate_road_section(&stations, &config);

    // Geometry should have valid bounds for viewport calculation
    assert!(!geometry.lines.is_empty());

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for seg in &geometry.lines {
        min_x = min_x.min(seg.x1).min(seg.x2);
        max_x = max_x.max(seg.x1).max(seg.x2);
        min_y = min_y.min(seg.y1).min(seg.y2);
        max_y = max_y.max(seg.y1).max(seg.y2);
    }

    let data_w = max_x - min_x;
    let data_h = max_y - min_y;
    assert!(data_w > 0.0, "Data width must be positive: {data_w}");
    assert!(data_h > 0.0, "Data height must be positive: {data_h}");

    // Simulate viewport calculation (800x600 canvas)
    let canvas_w = 800.0_f32;
    let canvas_h = 600.0_f32;
    let scale = (canvas_w / data_w as f32).min(canvas_h / data_h as f32) * 0.9;
    assert!(scale > 0.0, "Viewport scale must be positive: {scale}");
    assert!(scale.is_finite(), "Viewport scale must be finite");
}

#[test]
fn test_e2e_csv_to_dxf_export_to_viewer_parse() {
    // Full pipeline: CSV → stations → DXF string → file → read back → parse → validate
    let csv = "No.0,0.0,2.0,3.0\n10m,10.0,2.0,3.0\nNo.1,20.0,2.5,2.5\n";
    let stations = parse_road_section_csv(csv).unwrap();
    let dxf = stations_to_dxf(&stations);

    // Write to temp file (simulates viewer's file source)
    let tmp_dir = tempfile::tempdir().unwrap();
    let dxf_path = tmp_dir.path().join("output.dxf");
    std::fs::write(&dxf_path, &dxf).unwrap();

    // Read back (as viewer would)
    let content = std::fs::read_to_string(&dxf_path).unwrap();
    let doc = dxf_engine::parse_dxf(&content).unwrap();

    // Validate entity counts
    let config = RoadSectionConfig::default();
    let geometry = calculate_road_section(&stations, &config);
    assert_eq!(doc.lines.len(), geometry.lines.len(),
        "Line count mismatch: file={}, geometry={}", doc.lines.len(), geometry.lines.len());

    // Validate station names present
    assert!(doc.texts.iter().any(|t| t.text == "No.0"));
    assert!(doc.texts.iter().any(|t| t.text == "No.1"));
}

// ================================================================
// Asymmetric / edge case pipelines
// ================================================================

#[test]
fn test_e2e_single_station_pipeline() {
    let csv = "No.0,0.0,5.0,5.0\n";
    let stations = parse_road_section_csv(csv).unwrap();
    assert_eq!(stations.len(), 1);

    let dxf = stations_to_dxf(&stations);
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));

    let doc = dxf_engine::parse_dxf(&dxf).unwrap();
    assert_eq!(doc.lines.len(), 2, "Single station: 2 width lines");
}

#[test]
fn test_e2e_asymmetric_widths_pipeline() {
    let csv = "No.0,0.0,5.0,0.0\nNo.1,20.0,0.0,4.0\n";
    let stations = parse_road_section_csv(csv).unwrap();
    let dxf = stations_to_dxf(&stations);
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));

    let doc = dxf_engine::parse_dxf(&dxf).unwrap();
    // Both width dimensions should exist
    let texts: Vec<&str> = doc.texts.iter().map(|t| t.text.as_str()).collect();
    assert!(texts.contains(&"5.00"), "Should have left width 5.00, got: {texts:?}");
    assert!(texts.contains(&"4.00"), "Should have right width 4.00, got: {texts:?}");
}

#[test]
fn test_e2e_many_stations_pipeline() {
    let stations: Vec<StationData> = (0..50)
        .map(|i| StationData::new(&format!("No.{i}"), i as f64 * 20.0, 2.5, 2.5))
        .collect();
    let dxf = stations_to_dxf(&stations);
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));

    let doc = dxf_engine::parse_dxf(&dxf).unwrap();
    // 50 stations: 50×2 width + 49×3 connecting = 247 lines
    assert_eq!(doc.lines.len(), 247, "50 stations should produce 247 lines");

    // All station names should survive
    for i in 0..50 {
        let name = format!("No.{i}");
        assert!(doc.texts.iter().any(|t| t.text == name), "Missing station name: {name}");
    }
}

// ================================================================
// File-based pipeline with hot-swap simulation
// ================================================================

#[test]
fn test_e2e_file_overwrite_new_data_parseable() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let dxf_path = tmp_dir.path().join("road.dxf");

    // Version 1: 2 stations
    let csv_v1 = "No.0,0.0,2.5,2.5\nNo.1,20.0,2.5,2.5\n";
    let stations_v1 = parse_road_section_csv(csv_v1).unwrap();
    let dxf_v1 = stations_to_dxf(&stations_v1);
    std::fs::write(&dxf_path, &dxf_v1).unwrap();

    let doc_v1 = dxf_engine::parse_dxf(&std::fs::read_to_string(&dxf_path).unwrap()).unwrap();
    let lines_v1 = doc_v1.lines.len();

    // Version 2: 3 stations (overwrite)
    let csv_v2 = "No.0,0.0,2.5,2.5\nNo.1,20.0,2.5,2.5\nNo.2,40.0,2.5,2.5\n";
    let stations_v2 = parse_road_section_csv(csv_v2).unwrap();
    let dxf_v2 = stations_to_dxf(&stations_v2);
    std::fs::write(&dxf_path, &dxf_v2).unwrap();

    let doc_v2 = dxf_engine::parse_dxf(&std::fs::read_to_string(&dxf_path).unwrap()).unwrap();
    let lines_v2 = doc_v2.lines.len();

    assert!(lines_v2 > lines_v1,
        "More stations should produce more lines: v1={lines_v1}, v2={lines_v2}");
    assert!(doc_v2.texts.iter().any(|t| t.text == "No.2"),
        "New station No.2 should appear in overwritten DXF");
}

// ================================================================
// Coordinate precision E2E
// ================================================================

#[test]
fn test_e2e_coordinate_precision_through_pipeline() {
    let csv = "No.0,0.0,3.456,3.789\nNo.1,25.123,2.567,4.321\n";
    let stations = parse_road_section_csv(csv).unwrap();

    // Verify parsed values
    assert!((stations[0].wl - 3.456).abs() < 0.001);
    assert!((stations[1].x - 25.123).abs() < 0.001);

    let config = RoadSectionConfig::default();
    let geometry = calculate_road_section(&stations, &config);
    let dxf = geometry_to_dxf_string(&geometry);
    let doc = dxf_engine::parse_dxf(&dxf).unwrap();

    // First line coordinate should match within precision
    let (orig_lines, _) = road_section::geometry_to_dxf(&geometry);
    for (i, (orig, read)) in orig_lines.iter().zip(doc.lines.iter()).enumerate() {
        assert!((orig.x1 - read.x1).abs() < 0.1,
            "Line {i} x1 drift: orig={}, read={}", orig.x1, read.x1);
        assert!((orig.y1 - read.y1).abs() < 0.1,
            "Line {i} y1 drift: orig={}, read={}", orig.y1, read.y1);
    }
}
