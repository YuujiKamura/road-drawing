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
