//! End-to-end tests: CSV string → parse → calculate → geometry_to_dxf → DxfWriter → DxfLinter
//!
//! Full pipeline validation ensuring generated DXF passes lint.

use dxf_engine::{DxfLinter, DxfWriter};
use road_section::{
    calculate_road_section, geometry_to_dxf, parse_road_section_csv, RoadSectionConfig,
};

/// Helper: run the full pipeline and return (dxf_string, line_count, text_count)
fn run_pipeline(csv: &str, config: &RoadSectionConfig) -> (String, usize, usize) {
    let stations = parse_road_section_csv(csv).expect("CSV parse failed");
    let geometry = calculate_road_section(&stations, config);
    let (lines, texts) = geometry_to_dxf(&geometry);
    let writer = DxfWriter::new();
    let dxf_content = writer.write(&lines, &texts);
    (dxf_content, lines.len(), texts.len())
}

// ================================================================
// Basic 2-station pipeline
// ================================================================

#[test]
fn test_marking_e2e_two_stations_valid_dxf() {
    let csv = "測点名,累積延長,左幅員,右幅員\nNo.0,0.0,2.5,2.5\nNo.1,20.0,2.5,2.5\n";
    let (dxf, line_count, text_count) = run_pipeline(csv, &RoadSectionConfig::default());

    assert!(line_count > 0, "Should produce DXF lines");
    assert!(text_count > 0, "Should produce DXF texts");
    assert!(
        DxfLinter::is_valid(&dxf),
        "2-station pipeline DXF must pass linter"
    );
}

// ================================================================
// Multi-station pipeline
// ================================================================

#[test]
fn test_marking_e2e_five_stations_valid_dxf() {
    let csv = "\
測点名,累積延長,左幅員,右幅員
No.0,0.0,3.0,3.0
No.0+10,10.0,3.0,3.5
No.1,20.0,2.5,2.5
No.1+10,30.0,2.5,3.0
No.2,40.0,3.0,3.0
";
    let (dxf, line_count, text_count) = run_pipeline(csv, &RoadSectionConfig::default());

    assert!(line_count > 10, "5 stations should produce many lines: got {line_count}");
    assert!(text_count > 5, "5 stations should produce many texts: got {text_count}");
    assert!(
        DxfLinter::is_valid(&dxf),
        "5-station pipeline DXF must pass linter"
    );
}

// ================================================================
// Asymmetric widths
// ================================================================

#[test]
fn test_marking_e2e_asymmetric_widths_valid_dxf() {
    let csv = "\
測点名,累積延長,左幅員,右幅員
No.0,0.0,5.0,1.0
No.1,20.0,1.0,5.0
No.2,40.0,3.0,3.0
";
    let (dxf, _, _) = run_pipeline(csv, &RoadSectionConfig::default());
    assert!(
        DxfLinter::is_valid(&dxf),
        "Asymmetric widths DXF must pass linter"
    );
}

// ================================================================
// Zero width on one side
// ================================================================

#[test]
fn test_marking_e2e_zero_left_width_valid_dxf() {
    let csv = "name,x,wl,wr\nNo.0,0.0,0.0,3.0\nNo.1,20.0,0.0,3.0\n";
    let (dxf, _, _) = run_pipeline(csv, &RoadSectionConfig::default());
    assert!(
        DxfLinter::is_valid(&dxf),
        "Zero left width DXF must pass linter"
    );
}

#[test]
fn test_marking_e2e_zero_right_width_valid_dxf() {
    let csv = "name,x,wl,wr\nNo.0,0.0,3.0,0.0\nNo.1,20.0,3.0,0.0\n";
    let (dxf, _, _) = run_pipeline(csv, &RoadSectionConfig::default());
    assert!(
        DxfLinter::is_valid(&dxf),
        "Zero right width DXF must pass linter"
    );
}

// ================================================================
// Headerless CSV
// ================================================================

#[test]
fn test_marking_e2e_headerless_csv_valid_dxf() {
    let csv = "No.0,0.0,2.5,3.0\nNo.1,10.0,2.5,3.0\nNo.2,20.0,2.5,3.0\n";
    let (dxf, _, _) = run_pipeline(csv, &RoadSectionConfig::default());
    assert!(
        DxfLinter::is_valid(&dxf),
        "Headerless CSV pipeline DXF must pass linter"
    );
}

// ================================================================
// Japanese headers
// ================================================================

#[test]
fn test_marking_e2e_japanese_headers_valid_dxf() {
    let csv = "測点名,累積延長,左幅員,右幅員\nNo.0,0.0,2.5,2.5\nNo.1,10.0,3.0,3.0\nNo.2,20.0,2.5,2.5\n";
    let (dxf, _, _) = run_pipeline(csv, &RoadSectionConfig::default());
    assert!(
        DxfLinter::is_valid(&dxf),
        "Japanese header CSV pipeline DXF must pass linter"
    );
}

// ================================================================
// Custom scale factor
// ================================================================

#[test]
fn test_marking_e2e_custom_scale_valid_dxf() {
    let csv = "name,x,wl,wr\nNo.0,0.0,2.0,2.0\nNo.1,20.0,2.0,2.0\n";
    let config = RoadSectionConfig {
        scale: 500.0,
        ..Default::default()
    };
    let (dxf, _, _) = run_pipeline(csv, &config);
    assert!(
        DxfLinter::is_valid(&dxf),
        "Custom scale (500) DXF must pass linter"
    );
}

// ================================================================
// Single station (no connecting lines)
// ================================================================

#[test]
fn test_marking_e2e_single_station_valid_dxf() {
    let csv = "name,x,wl,wr\nNo.0,0.0,2.5,2.5\n";
    let (dxf, line_count, _) = run_pipeline(csv, &RoadSectionConfig::default());

    assert!(line_count > 0, "Single station should still produce width lines");
    assert!(
        DxfLinter::is_valid(&dxf),
        "Single station DXF must pass linter"
    );
}

// ================================================================
// Many stations (stress test)
// ================================================================

#[test]
fn test_marking_e2e_many_stations_valid_dxf() {
    let mut csv = "name,x,wl,wr\n".to_string();
    for i in 0..50 {
        csv.push_str(&format!("No.{},{:.1},{:.1},{:.1}\n", i, i as f64 * 20.0, 2.5, 3.0));
    }
    let (dxf, line_count, _) = run_pipeline(&csv, &RoadSectionConfig::default());

    assert!(
        line_count > 100,
        "50 stations should produce many lines: got {line_count}"
    );
    assert!(
        DxfLinter::is_valid(&dxf),
        "50-station pipeline DXF must pass linter"
    );
}

// ================================================================
// CSV with comments and blank lines
// ================================================================

#[test]
fn test_marking_e2e_csv_with_noise_valid_dxf() {
    let csv = "\
name,x,wl,wr
# this is a comment
No.0,0.0,2.5,2.5

No.1,10.0,2.5,2.5
# another comment

No.2,20.0,3.0,3.0
";
    let (dxf, _, _) = run_pipeline(csv, &RoadSectionConfig::default());
    assert!(
        DxfLinter::is_valid(&dxf),
        "CSV with comments/blanks pipeline DXF must pass linter"
    );
}

// ================================================================
// DXF content structure checks
// ================================================================

#[test]
fn test_marking_e2e_dxf_contains_sections() {
    let csv = "name,x,wl,wr\nNo.0,0.0,2.5,2.5\nNo.1,20.0,2.5,2.5\n";
    let (dxf, _, _) = run_pipeline(csv, &RoadSectionConfig::default());

    assert!(dxf.contains("SECTION"), "DXF must contain SECTION");
    assert!(dxf.contains("ENTITIES"), "DXF must contain ENTITIES section");
    assert!(dxf.contains("ENDSEC"), "DXF must contain ENDSEC");
    assert!(dxf.contains("EOF"), "DXF must end with EOF");
}

#[test]
fn test_marking_e2e_dxf_contains_line_entities() {
    let csv = "name,x,wl,wr\nNo.0,0.0,2.5,2.5\nNo.1,20.0,2.5,2.5\n";
    let (dxf, _, _) = run_pipeline(csv, &RoadSectionConfig::default());

    assert!(dxf.contains("LINE"), "DXF must contain LINE entities");
    assert!(dxf.contains("TEXT"), "DXF must contain TEXT entities");
}

// ================================================================
// Station name color in DXF output
// ================================================================

#[test]
fn test_marking_e2e_station_names_blue() {
    let csv = "name,x,wl,wr\nNo.0,0.0,2.5,2.5\nNo.1,20.0,2.5,2.5\n";
    let stations = parse_road_section_csv(csv).unwrap();
    let geometry = calculate_road_section(&stations, &RoadSectionConfig::default());
    let (_, texts) = geometry_to_dxf(&geometry);

    let name_texts: Vec<_> = texts.iter().filter(|t| t.text.starts_with("No.")).collect();
    assert!(name_texts.len() >= 2, "Should have at least 2 station name texts");
    for t in &name_texts {
        assert_eq!(t.color, 5, "Station name '{}' must be blue (color 5)", t.text);
    }
}

// ================================================================
// Error cases: pipeline should fail gracefully
// ================================================================

#[test]
fn test_marking_e2e_empty_csv_errors() {
    let result = parse_road_section_csv("");
    assert!(result.is_err(), "Empty CSV must return error");
}

#[test]
fn test_marking_e2e_header_only_csv_errors() {
    let result = parse_road_section_csv("測点名,累積延長,左幅員,右幅員\n");
    assert!(result.is_err(), "Header-only CSV must return error");
}

// ================================================================
// 区間1 realistic data: full quality verification
// ================================================================

#[test]
fn test_marking_e2e_kukan1_dxf_quality() {
    use dxf_engine::parse_dxf;

    // Simulate 区間1 data: 6 stations with varying widths
    let csv = "\
測点名,累積延長,左幅員,右幅員
No.0,0.0,3.25,3.25
No.0+10,10.0,3.25,3.50
No.1,20.0,3.00,3.00
No.1+10,30.0,3.00,3.25
No.2,40.0,3.25,3.25
No.2+5,45.0,3.50,3.00
";

    let stations = parse_road_section_csv(csv).unwrap();
    assert_eq!(stations.len(), 6);

    let config = RoadSectionConfig::default();
    let geometry = calculate_road_section(&stations, &config);
    let (lines, texts) = geometry_to_dxf(&geometry);

    // Write DXF
    let writer = DxfWriter::new();
    let dxf_content = writer.write(&lines, &texts);

    // 1. Linter validation
    let lint_result = dxf_engine::DxfLinter::lint(&dxf_content);
    assert!(lint_result.is_ok(), "区間1 DXF lint failed: {:?}", lint_result.errors);
    assert!(lint_result.stats.entity_count > 0, "DXF must have entities");

    // 2. Roundtrip parse
    let doc = parse_dxf(&dxf_content).unwrap();
    assert_eq!(doc.lines.len(), lines.len(), "Roundtrip line count mismatch");
    assert_eq!(doc.texts.len(), texts.len(), "Roundtrip text count mismatch");

    // 3. Station name texts: must be blue (color 5), rotated -90°, height 350
    let station_names: Vec<_> = doc.texts.iter()
        .filter(|t| t.text.starts_with("No."))
        .collect();
    assert_eq!(station_names.len(), 6, "Should have 6 station name labels");
    for t in &station_names {
        assert_eq!(t.color, 5, "Station name '{}' must be blue (color 5)", t.text);
        assert!(
            (t.rotation - (-90.0)).abs() < 0.01,
            "Station name '{}' rotation should be -90°, got {}", t.text, t.rotation
        );
        assert!(
            (t.height - 350.0).abs() < 0.01,
            "Station name '{}' height should be 350, got {}", t.text, t.height
        );
    }

    // 4. Dimension texts: must have height 350, non-blue
    let dimension_texts: Vec<_> = doc.texts.iter()
        .filter(|t| !t.text.starts_with("No."))
        .collect();
    assert!(!dimension_texts.is_empty(), "Should have dimension texts");
    for t in &dimension_texts {
        assert!(
            (t.height - 350.0).abs() < 0.01,
            "Dimension text '{}' height should be 350, got {}", t.text, t.height
        );
        assert_ne!(t.color, 5, "Dimension text '{}' should NOT be blue", t.text);
    }

    // 5. Verify specific station names survived roundtrip
    let names: Vec<&str> = station_names.iter().map(|t| t.text.as_str()).collect();
    assert!(names.contains(&"No.0"));
    assert!(names.contains(&"No.0+10"));
    assert!(names.contains(&"No.1"));
    assert!(names.contains(&"No.2+5"));
}

#[test]
fn test_marking_e2e_kukan1_text_alignment_consistency() {
    use dxf_engine::{HorizontalAlignment, VerticalAlignment};

    let csv = "\
測点名,累積延長,左幅員,右幅員
No.0,0.0,3.25,3.25
No.1,20.0,3.00,3.00
No.2,40.0,3.25,3.25
";

    let stations = parse_road_section_csv(csv).unwrap();
    let geometry = calculate_road_section(&stations, &RoadSectionConfig::default());
    let (_, texts) = geometry_to_dxf(&geometry);

    // Station name texts should have Center horizontal alignment
    let station_texts: Vec<_> = texts.iter()
        .filter(|t| t.text.starts_with("No."))
        .collect();
    for t in &station_texts {
        assert_eq!(
            t.align_h, HorizontalAlignment::Center,
            "Station name '{}' should be center-aligned", t.text
        );
        assert_eq!(
            t.align_v, VerticalAlignment::Bottom,
            "Station name '{}' should be bottom-aligned", t.text
        );
    }

    // Width dimension texts should be rotated -90°
    let width_texts: Vec<_> = texts.iter()
        .filter(|t| !t.text.starts_with("No.") && t.rotation.abs() > 0.01)
        .collect();
    assert!(!width_texts.is_empty(), "Should have rotated width dimension texts");
    for t in &width_texts {
        assert!(
            (t.rotation - (-90.0)).abs() < 0.01,
            "Width text '{}' should be -90° rotated, got {}", t.text, t.rotation
        );
    }

    // Full DXF lint
    let writer = DxfWriter::new();
    let dxf = writer.write(&[], &texts);
    assert!(DxfLinter::is_valid(&dxf));
}

// ================================================================
// Gap: excel-parser → road-section integration (RawRow → StationData)
// ================================================================

#[test]
fn test_excel_parser_to_road_section_pipeline() {
    // Simulate excel-parser output (RawRow fields map to StationData)
    let csv_text = "\
区間1,台形計算
測点名,単延長L,幅員W,幅員右
No.0,0.00,3.25,3.25
,10.00,3.25,3.50
,10.00,3.00,3.00
";
    let sections = excel_parser::transform::list_sections_text(csv_text, "test.csv");
    assert!(!sections.is_empty(), "Should detect at least one section");

    let rows = excel_parser::transform::extract_and_transform_text(csv_text, &sections[0]).unwrap();
    assert!(rows.len() >= 3, "Should have at least 3 rows after transform");

    // Convert RawRow → StationData (same field mapping as CLI does)
    let stations: Vec<_> = rows.iter().map(|r| {
        road_section::StationData::new(&r.name, r.x, r.wl, r.wr)
    }).collect();

    // Verify station names were filled
    assert!(!stations[0].name.is_empty(), "First station should have a name");
    assert!(stations[0].name.starts_with("No."), "First station should be No.0, got '{}'", stations[0].name);

    // Full pipeline: calculate geometry → DXF → lint
    let config = RoadSectionConfig::default();
    let geometry = calculate_road_section(&stations, &config);
    let (lines, texts) = geometry_to_dxf(&geometry);
    let writer = DxfWriter::new();
    let dxf = writer.write(&lines, &texts);

    assert!(DxfLinter::is_valid(&dxf), "excel-parser → road-section → DXF pipeline must pass lint");
    let doc = dxf_engine::parse_dxf(&dxf).unwrap();
    assert_eq!(doc.lines.len(), lines.len(), "Roundtrip line count");
    assert_eq!(doc.texts.len(), texts.len(), "Roundtrip text count");
}

// ================================================================
// Gap: full pipeline with reader roundtrip verification
// ================================================================

#[test]
fn test_full_pipeline_csv_to_dxf_roundtrip() {
    let csv = "\
測点名,累積延長,左幅員,右幅員
No.0,0.0,2.50,2.50
No.0+10,10.0,2.50,3.00
No.1,20.0,3.00,3.00
No.1+10,30.0,3.00,2.50
No.2,40.0,2.50,2.50
";
    let stations = parse_road_section_csv(csv).unwrap();
    let config = RoadSectionConfig::default();
    let geometry = calculate_road_section(&stations, &config);
    let (lines, texts) = geometry_to_dxf(&geometry);

    let writer = DxfWriter::new();
    let dxf = writer.write(&lines, &texts);

    // Lint
    assert!(DxfLinter::is_valid(&dxf));

    // Parse back
    let doc = dxf_engine::parse_dxf(&dxf).unwrap();

    // Line count preserved
    assert_eq!(doc.lines.len(), lines.len());
    // Text count preserved
    assert_eq!(doc.texts.len(), texts.len());

    // All station names survive roundtrip
    let parsed_names: Vec<&str> = doc.texts.iter()
        .filter(|t| t.text.starts_with("No."))
        .map(|t| t.text.as_str())
        .collect();
    for s in &stations {
        assert!(parsed_names.contains(&s.name.as_str()),
            "Station '{}' not found in roundtrip DXF texts", s.name);
    }

    // First line coordinate precision
    let orig = &lines[0];
    let read = &doc.lines[0];
    assert!((orig.x1 - read.x1).abs() < 0.01, "x1 precision lost");
    assert!((orig.y1 - read.y1).abs() < 0.01, "y1 precision lost");
}
