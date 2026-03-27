//! WASM Bridge tests — テスト役B担当
//!
//! (1) wasm-pack buildが通ること → CI / build.sh で検証済み前提、ここではRust層テスト
//! (2) JS呼び出しと同等のシグネチャでの正確性
//! (3) CSV→DXF変換エッジケース（空CSV、巨大CSV、不正値）

use road_drawing_wasm::{generate_dxf, get_preview_data, parse_csv};

// ─── parse_csv tests ───

#[test]
fn wb_parse_csv_valid_with_header() {
    let csv = "測点名,延長,左幅員,右幅員\nNo.0,0.0,3.45,3.55\nNo.1,20.0,3.50,3.50\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "unexpected error: {json}");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "No.0");
    assert_eq!(arr[0]["x"], 0.0);
    assert_eq!(arr[0]["wl"], 3.45);
    assert_eq!(arr[0]["wr"], 3.55);
    assert_eq!(arr[1]["name"], "No.1");
    assert_eq!(arr[1]["x"], 20.0);
}

#[test]
fn wb_parse_csv_valid_no_header() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "unexpected error: {json}");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn wb_parse_csv_empty() {
    let json = parse_csv("");
    assert!(json.contains("error") || json.contains("Empty"),
        "empty CSV should produce error, got: {json}");
}

#[test]
fn wb_parse_csv_header_only() {
    let csv = "測点名,延長,左幅員,右幅員\n";
    let json = parse_csv(csv);
    // No data rows → should be error
    assert!(json.contains("error") || json.contains("No valid"),
        "header-only CSV should produce error, got: {json}");
}

#[test]
fn wb_parse_csv_single_station() {
    let csv = "Sta,0.0,2.5,2.5\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "unexpected error: {json}");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "Sta");
}

#[test]
fn wb_parse_csv_missing_columns() {
    // Only 2 columns → should skip (needs >=4)
    let csv = "No.0,0.0\n";
    let json = parse_csv(csv);
    assert!(json.contains("error") || json.contains("No valid"),
        "2-column CSV should fail, got: {json}");
}

#[test]
fn wb_parse_csv_invalid_x_value() {
    let csv = "No.0,abc,3.0,3.0\n";
    let json = parse_csv(csv);
    // x column parse failure → should be error
    assert!(json.contains("error") || json.contains("invalid"),
        "non-numeric x should produce error, got: {json}");
}

#[test]
fn wb_parse_csv_negative_widths() {
    // Negative widths are geometrically unusual but should parse
    let csv = "No.0,0.0,-1.0,3.0\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "negative width should parse: {json}");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr[0]["wl"], -1.0);
}

#[test]
fn wb_parse_csv_zero_widths() {
    let csv = "No.0,0.0,0.0,0.0\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "zero widths should parse: {json}");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr[0]["wl"], 0.0);
    assert_eq!(arr[0]["wr"], 0.0);
}

#[test]
fn wb_parse_csv_whitespace_and_blank_lines() {
    let csv = "\n  No.0 , 0.0 , 3.0 , 3.0  \n\n  No.1 , 20.0 , 3.5 , 3.5  \n\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "whitespace CSV should parse: {json}");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn wb_parse_csv_comment_lines_skipped() {
    let csv = "# This is a comment\nNo.0,0.0,3.0,3.0\n# Another comment\nNo.1,20.0,3.5,3.5\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "comments should be skipped: {json}");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn wb_parse_csv_japanese_headers() {
    let csv = "測点名,単延長,左幅員,右幅員\nNo.0,0.0,3.45,3.55\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "Japanese header should work: {json}");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 1);
}

#[test]
fn wb_parse_csv_english_headers() {
    let csv = "name,x,wl,wr\nNo.0,0.0,3.0,3.0\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "English header should work: {json}");
}

// ─── generate_dxf tests ───

#[test]
fn wb_generate_dxf_valid() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"), "unexpected error: {dxf}");
    assert!(dxf.contains("SECTION"), "missing SECTION in DXF");
    assert!(dxf.contains("ENTITIES"), "missing ENTITIES in DXF");
    assert!(dxf.contains("EOF"), "missing EOF in DXF");
    assert!(dxf.contains("LINE"), "missing LINE entities");
}

#[test]
fn wb_generate_dxf_empty_csv() {
    let dxf = generate_dxf("");
    assert!(dxf.starts_with("ERROR"), "empty CSV should return ERROR: {dxf}");
}

#[test]
fn wb_generate_dxf_invalid_csv() {
    let dxf = generate_dxf("garbage,data");
    assert!(dxf.starts_with("ERROR"), "invalid CSV should return ERROR: {dxf}");
}

#[test]
fn wb_generate_dxf_station_names_in_output() {
    let csv = "TestSta,0.0,3.0,3.0\nNextSta,20.0,3.5,3.5\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"), "unexpected error: {dxf}");
    // Station names should appear as TEXT entities
    assert!(dxf.contains("TestSta"), "station name TestSta missing from DXF");
    assert!(dxf.contains("NextSta"), "station name NextSta missing from DXF");
}

#[test]
fn wb_generate_dxf_single_station() {
    let csv = "No.0,0.0,3.0,3.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"), "single station should work: {dxf}");
    assert!(dxf.contains("SECTION"));
    assert!(dxf.contains("EOF"));
}

#[test]
fn wb_generate_dxf_lint_valid() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\nNo.2,40.0,3.0,3.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    // DxfLinter validation
    assert!(dxf_engine::DxfLinter::is_valid(&dxf),
        "generated DXF should pass linter");
}

#[test]
fn wb_generate_dxf_large_csv_50_stations() {
    let mut csv = String::new();
    for i in 0..50 {
        csv.push_str(&format!("No.{},{:.1},3.0,3.0\n", i, i as f64 * 20.0));
    }
    let dxf = generate_dxf(&csv);
    assert!(!dxf.starts_with("ERROR"), "50 stations should work");
    assert!(dxf.contains("SECTION"));
    assert!(dxf.contains("EOF"));
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

#[test]
fn wb_generate_dxf_large_csv_500_stations() {
    let mut csv = String::new();
    for i in 0..500 {
        csv.push_str(&format!("No.{},{:.1},3.0,3.0\n", i, i as f64 * 20.0));
    }
    let dxf = generate_dxf(&csv);
    assert!(!dxf.starts_with("ERROR"), "500 stations should work");
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

#[test]
fn wb_generate_dxf_extreme_values() {
    // Very large coordinates
    let csv = "Far,99999.0,999.0,999.0\nFarther,100000.0,999.0,999.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"), "extreme values should work: {dxf}");
}

#[test]
fn wb_generate_dxf_tiny_widths() {
    let csv = "No.0,0.0,0.001,0.001\nNo.1,20.0,0.001,0.001\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"), "tiny widths should work");
}

// ─── get_preview_data tests ───

#[test]
fn wb_preview_data_valid() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n";
    let json = get_preview_data(csv);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["lines"].is_array(), "should have lines array");
    assert!(parsed["texts"].is_array(), "should have texts array");
    assert!(!parsed["lines"].as_array().unwrap().is_empty(), "lines should not be empty");
    assert!(!parsed["texts"].as_array().unwrap().is_empty(), "texts should not be empty");
}

#[test]
fn wb_preview_data_empty_csv() {
    let json = get_preview_data("");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    // Empty CSV → empty geometry
    assert!(parsed["lines"].as_array().unwrap().is_empty());
    assert!(parsed["texts"].as_array().unwrap().is_empty());
}

#[test]
fn wb_preview_data_invalid_csv() {
    let json = get_preview_data("not,a,valid,csv,really");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    // Invalid → graceful empty result
    assert!(parsed["lines"].is_array());
    assert!(parsed["texts"].is_array());
}

#[test]
fn wb_preview_data_line_structure() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n";
    let json = get_preview_data(csv);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let first_line = &parsed["lines"][0];
    // Each line should have x1,y1,x2,y2,color
    assert!(first_line["x1"].is_f64(), "line should have x1");
    assert!(first_line["y1"].is_f64(), "line should have y1");
    assert!(first_line["x2"].is_f64(), "line should have x2");
    assert!(first_line["y2"].is_f64(), "line should have y2");
    assert!(first_line["color"].is_i64(), "line should have color");
}

#[test]
fn wb_preview_data_text_structure() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n";
    let json = get_preview_data(csv);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let texts = parsed["texts"].as_array().unwrap();
    assert!(!texts.is_empty());
    let first_text = &texts[0];
    assert!(first_text["text"].is_string(), "text should have text field");
    assert!(first_text["x"].is_f64(), "text should have x");
    assert!(first_text["y"].is_f64(), "text should have y");
    assert!(first_text["rotation"].is_f64(), "text should have rotation");
    assert!(first_text["height"].is_f64(), "text should have height");
    assert!(first_text["color"].is_i64(), "text should have color");
}

#[test]
fn wb_preview_data_station_names_present() {
    let csv = "Alpha,0.0,3.0,3.0\nBeta,20.0,3.5,3.5\n";
    let json = get_preview_data(csv);
    assert!(json.contains("Alpha"), "station name Alpha should appear in preview");
    assert!(json.contains("Beta"), "station name Beta should appear in preview");
}

#[test]
fn wb_preview_data_scale_default_1000() {
    // Default scale = 1000 (m→mm)
    // Station at x=20m → x coordinate should be 20*1000 = 20000 range
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.0,3.0\n";
    let json = get_preview_data(csv);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let lines = parsed["lines"].as_array().unwrap();
    // At least some coordinate should be ≥ 1000 (scaled)
    let has_scaled = lines.iter().any(|l| {
        l["x2"].as_f64().unwrap_or(0.0).abs() >= 1000.0
    });
    assert!(has_scaled, "coordinates should be scaled by ~1000");
}

// ─── Consistency: parse_csv → generate_dxf use same data ───

#[test]
fn wb_parse_and_generate_consistency() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\nNo.2,40.0,3.0,3.0\n";
    let parsed_json = parse_csv(csv);
    let parsed: serde_json::Value = serde_json::from_str(&parsed_json).unwrap();
    let station_count = parsed.as_array().unwrap().len();
    assert_eq!(station_count, 3);

    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    // All station names should appear in both parse and DXF
    for name in &["No.0", "No.1", "No.2"] {
        assert!(dxf.contains(name), "DXF should contain station {name}");
    }
}

#[test]
fn wb_parse_and_preview_consistency() {
    let csv = "A,0.0,2.0,2.0\nB,10.0,3.0,3.0\n";
    let parsed_json = parse_csv(csv);
    assert!(!parsed_json.contains("error"));

    let preview_json = get_preview_data(csv);
    let preview: serde_json::Value = serde_json::from_str(&preview_json).unwrap();
    assert!(!preview["lines"].as_array().unwrap().is_empty());
}

// ─── JSON output is valid JSON ───

#[test]
fn wb_parse_csv_returns_valid_json() {
    let csv = "No.0,0.0,3.0,3.0\n";
    let json = parse_csv(csv);
    let result: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(result.is_ok(), "parse_csv should return valid JSON: {json}");
}

#[test]
fn wb_preview_data_returns_valid_json() {
    let csv = "No.0,0.0,3.0,3.0\n";
    let json = get_preview_data(csv);
    let result: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(result.is_ok(), "get_preview_data should return valid JSON: {json}");
}

#[test]
fn wb_parse_csv_error_returns_valid_json() {
    let json = parse_csv("");
    // Even error responses should be valid JSON
    let result: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(result.is_ok(), "error response should be valid JSON: {json}");
}
