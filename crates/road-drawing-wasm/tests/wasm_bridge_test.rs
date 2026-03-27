//! WASM Bridge tests — テスト役B担当
//!
//! (1) wasm-pack buildが通ること → CI / build.sh で検証済み前提、ここではRust層テスト
//! (2) JS呼び出しと同等のシグネチャでの正確性
//! (3) CSV→DXF変換エッジケース（空CSV、巨大CSV、不正値）

use road_drawing_wasm::{generate_dxf, get_preview_data, parse_csv};
use serde_json::Value;

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

// ═══════════════════════════════════════════════════════════
// CSV→DXF Edge Cases — Phase 2 (テスト役B追加)
// ═══════════════════════════════════════════════════════════

// ─── 同一x値 / 非単調増加 ───

#[test]
fn wb_edge_duplicate_x_values() {
    // 2つの測点が同じx座標 → distance=0、接続線なし
    let csv = "A,10.0,3.0,3.0\nB,10.0,4.0,4.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"), "duplicate x should work: {dxf}");
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

#[test]
fn wb_edge_non_monotonic_x() {
    // x値が減少 → 逆走。パースは通るがジオメトリに注意
    let csv = "A,20.0,3.0,3.0\nB,10.0,3.0,3.0\nC,30.0,3.0,3.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"), "non-monotonic x should not crash");
}

#[test]
fn wb_edge_x_starts_nonzero() {
    // 起点がx=100から始まる
    let csv = "No.5,100.0,3.0,3.0\nNo.6,120.0,3.5,3.5\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    assert!(dxf.contains("No.5"));
    assert!(dxf.contains("No.6"));
}

// ─── 左右非対称幅員 ───

#[test]
fn wb_edge_asymmetric_widths() {
    // 左幅員と右幅員が大きく異なる
    let csv = "No.0,0.0,1.0,5.0\nNo.1,20.0,5.0,1.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    let preview: Value = serde_json::from_str(&get_preview_data(csv)).unwrap();
    let lines = preview["lines"].as_array().unwrap();
    // 幅員線: 左+右×2測点 + 接続線3本 = 7本以上
    assert!(lines.len() >= 7, "asymmetric should produce >=7 lines, got {}", lines.len());
}

#[test]
fn wb_edge_left_only_width() {
    // 右幅員が0、左のみ
    let csv = "No.0,0.0,3.0,0.0\nNo.1,20.0,3.0,0.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

#[test]
fn wb_edge_right_only_width() {
    // 左幅員が0、右のみ
    let csv = "No.0,0.0,0.0,3.0\nNo.1,20.0,0.0,3.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

// ─── 幅員変化パターン ───

#[test]
fn wb_edge_width_narrows_to_zero() {
    // 幅員が徐々に狭くなってゼロになる
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,1.5,1.5\nNo.2,40.0,0.0,0.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

#[test]
fn wb_edge_width_expands_drastically() {
    // 幅員が急激に広がる
    let csv = "No.0,0.0,1.0,1.0\nNo.1,5.0,20.0,20.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

// ─── 距離パターン ───

#[test]
fn wb_edge_very_close_stations() {
    // 0.01m間隔 → dimension textの配置が問題になりうる
    let csv = "A,0.0,3.0,3.0\nB,0.01,3.0,3.0\nC,0.02,3.0,3.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
}

#[test]
fn wb_edge_very_far_stations() {
    // 10km間隔
    let csv = "Start,0.0,3.0,3.0\nEnd,10000.0,3.0,3.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

// ─── 実際の施工データに近いパターン ───

#[test]
fn wb_edge_realistic_road_section() {
    // 舗装工事: 測点20m間隔、幅員3.0-3.5m
    let csv = "\
測点名,延長,左幅員,右幅員
No.0,0.0,3.25,3.25
No.1,20.0,3.25,3.25
No.2,40.0,3.50,3.25
No.2+10,50.0,3.50,3.50
No.3,60.0,3.25,3.50
No.3+5,65.0,3.25,3.25
No.4,80.0,3.00,3.00
";
    let parsed: Value = serde_json::from_str(&parse_csv(csv)).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 7, "7 stations");

    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
    // 全測点名が出力に含まれる
    for name in &["No.0", "No.1", "No.2", "No.2+10", "No.3", "No.3+5", "No.4"] {
        assert!(dxf.contains(name), "missing station {name} in DXF");
    }
}

#[test]
fn wb_edge_realistic_uneven_intervals() {
    // 不等間隔: 10m, 25m, 15m, 30m
    let csv = "A,0.0,3.0,3.0\nB,10.0,3.0,3.0\nC,35.0,3.5,3.5\nD,50.0,3.0,3.0\nE,80.0,3.0,3.0\n";
    let dxf = generate_dxf(csv);
    assert!(!dxf.starts_with("ERROR"));
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

// ─── ジオメトリ精度検証 ───

#[test]
fn wb_edge_geometry_line_count_1_station() {
    // 1測点: 左幅員線1 + 右幅員線1 = 2本 (接続線なし)
    let preview: Value = serde_json::from_str(
        &get_preview_data("No.0,0.0,3.0,3.0\n")
    ).unwrap();
    let lines = preview["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 2, "1 station should produce 2 width lines");
}

#[test]
fn wb_edge_geometry_line_count_2_stations() {
    // 2測点: 幅員線4 + 接続線3(左/中/右) = 7本
    let preview: Value = serde_json::from_str(
        &get_preview_data("No.0,0.0,3.0,3.0\nNo.1,20.0,3.0,3.0\n")
    ).unwrap();
    let lines = preview["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 7, "2 stations should produce 7 lines, got {}", lines.len());
}

#[test]
fn wb_edge_geometry_text_count_2_stations() {
    // 2測点: 名前2 + 左幅員2 + 右幅員2 + 距離1 = 7テキスト
    let preview: Value = serde_json::from_str(
        &get_preview_data("No.0,0.0,3.0,3.0\nNo.1,20.0,3.0,3.0\n")
    ).unwrap();
    let texts = preview["texts"].as_array().unwrap();
    assert_eq!(texts.len(), 7, "2 stations should produce 7 texts, got {}", texts.len());
}

#[test]
fn wb_edge_geometry_coordinates_exact() {
    // 1測点 x=0, wl=3.0, wr=3.0, scale=1000
    // 左幅員線: (0,0)→(0,3000)  右幅員線: (0,0)→(0,-3000)
    let preview: Value = serde_json::from_str(
        &get_preview_data("No.0,0.0,3.0,3.0\n")
    ).unwrap();
    let lines = preview["lines"].as_array().unwrap();

    // 左幅員線
    let left = &lines[0];
    assert_eq!(left["x1"].as_f64().unwrap(), 0.0);
    assert_eq!(left["y1"].as_f64().unwrap(), 0.0);
    assert_eq!(left["x2"].as_f64().unwrap(), 0.0);
    assert_eq!(left["y2"].as_f64().unwrap(), 3000.0);

    // 右幅員線
    let right = &lines[1];
    assert_eq!(right["x1"].as_f64().unwrap(), 0.0);
    assert_eq!(right["y1"].as_f64().unwrap(), 0.0);
    assert_eq!(right["x2"].as_f64().unwrap(), 0.0);
    assert_eq!(right["y2"].as_f64().unwrap(), -3000.0);
}

#[test]
fn wb_edge_geometry_station_name_color_blue() {
    // 測点名ラベルはcolor 5 (blue)
    let preview: Value = serde_json::from_str(
        &get_preview_data("TestSta,0.0,3.0,3.0\n")
    ).unwrap();
    let texts = preview["texts"].as_array().unwrap();
    let name_text = texts.iter().find(|t| t["text"].as_str() == Some("TestSta")).unwrap();
    assert_eq!(name_text["color"].as_i64().unwrap(), 5, "station name should be blue (color 5)");
}

#[test]
fn wb_edge_geometry_dimension_rotation() {
    // 幅員テキストはrotation -90度
    let preview: Value = serde_json::from_str(
        &get_preview_data("No.0,0.0,3.0,3.0\n")
    ).unwrap();
    let texts = preview["texts"].as_array().unwrap();
    let width_text = texts.iter().find(|t| t["text"].as_str() == Some("3.00")).unwrap();
    assert_eq!(width_text["rotation"].as_f64().unwrap(), -90.0,
        "width dimension should have -90 rotation");
}

// ─── DXF roundtrip (generate → parse → verify) ───

#[test]
fn wb_edge_dxf_roundtrip_line_count() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\nNo.2,40.0,3.0,3.0\n";
    let dxf = generate_dxf(csv);
    let parsed = dxf_engine::parse_dxf(&dxf).unwrap();
    // 3 stations: 幅員線6 + 接続線6(左+中+右×2区間) = 12本
    assert!(parsed.lines.len() >= 10,
        "3-station DXF should have >=10 lines, got {}", parsed.lines.len());
}

#[test]
fn wb_edge_dxf_roundtrip_text_count() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n";
    let dxf = generate_dxf(csv);
    let parsed = dxf_engine::parse_dxf(&dxf).unwrap();
    // 2 stations: 名前2 + 左幅員2 + 右幅員2 + 距離1 = 7
    assert_eq!(parsed.texts.len(), 7,
        "2-station DXF should have 7 texts, got {}", parsed.texts.len());
}

#[test]
fn wb_edge_dxf_roundtrip_station_names() {
    let csv = "起点,0.0,3.0,3.0\n終点,100.0,3.0,3.0\n";
    let dxf = generate_dxf(csv);
    let parsed = dxf_engine::parse_dxf(&dxf).unwrap();
    let text_contents: Vec<&str> = parsed.texts.iter().map(|t| t.text.as_str()).collect();
    assert!(text_contents.contains(&"起点"), "Japanese station name 起点 missing");
    assert!(text_contents.contains(&"終点"), "Japanese station name 終点 missing");
}

// ─── CSV特殊文字 ───

#[test]
fn wb_edge_csv_crlf() {
    let csv = "No.0,0.0,3.0,3.0\r\nNo.1,20.0,3.5,3.5\r\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "CRLF should work: {json}");
    let arr: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(arr.as_array().unwrap().len(), 2);
}

#[test]
fn wb_edge_csv_mixed_crlf_lf() {
    let csv = "No.0,0.0,3.0,3.0\r\nNo.1,20.0,3.5,3.5\nNo.2,40.0,3.0,3.0\r\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"));
    let arr: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(arr.as_array().unwrap().len(), 3);
}

#[test]
fn wb_edge_csv_extra_columns_ignored() {
    let csv = "No.0,0.0,3.0,3.0,extra1,extra2\nNo.1,20.0,3.5,3.5,junk,\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"), "extra columns should be ignored: {json}");
}

#[test]
fn wb_edge_csv_unicode_station_names() {
    let csv = "🛣️Road,0.0,3.0,3.0\n日本語テスト,20.0,3.0,3.0\n";
    let json = parse_csv(csv);
    assert!(!json.contains("error"));
    let arr: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(arr[0]["name"].as_str().unwrap(), "🛣️Road");
    assert_eq!(arr[1]["name"].as_str().unwrap(), "日本語テスト");
}

#[test]
fn wb_edge_csv_decimal_precision() {
    // 小数点以下6桁
    let csv = "No.0,0.123456,3.141592,2.718281\n";
    let json = parse_csv(csv);
    let arr: Value = serde_json::from_str(&json).unwrap();
    let x = arr[0]["x"].as_f64().unwrap();
    assert!((x - 0.123456).abs() < 1e-10, "precision lost: {x}");
}

// ─── 1000局ストレステスト ───

#[test]
fn wb_edge_stress_1000_stations() {
    let mut csv = String::new();
    for i in 0..1000 {
        csv.push_str(&format!("No.{},{:.1},{:.2},{:.2}\n",
            i, i as f64 * 10.0,
            3.0 + (i as f64 * 0.001), 3.0 + (i as f64 * 0.001)));
    }
    let dxf = generate_dxf(&csv);
    assert!(!dxf.starts_with("ERROR"), "1000 stations should work");
    assert!(dxf_engine::DxfLinter::is_valid(&dxf));
}

#[test]
fn wb_edge_stress_preview_1000_stations() {
    let mut csv = String::new();
    for i in 0..1000 {
        csv.push_str(&format!("No.{},{:.1},3.0,3.0\n", i, i as f64 * 10.0));
    }
    let json = get_preview_data(&csv);
    let data: Value = serde_json::from_str(&json).unwrap();
    let lines = data["lines"].as_array().unwrap();
    // 1000局: 幅員線2000 + 接続線2997 = ~5000本
    assert!(lines.len() > 4000, "1000 stations should have >4000 lines, got {}", lines.len());
}

// ─── generate_dxf と get_preview_data の整合性 ───

#[test]
fn wb_edge_dxf_preview_line_count_match() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\nNo.2,40.0,3.0,3.0\n";
    let dxf = generate_dxf(csv);
    let dxf_parsed = dxf_engine::parse_dxf(&dxf).unwrap();

    let preview: Value = serde_json::from_str(&get_preview_data(csv)).unwrap();
    let preview_lines = preview["lines"].as_array().unwrap().len();

    assert_eq!(dxf_parsed.lines.len(), preview_lines,
        "DXF line count ({}) should match preview ({})",
        dxf_parsed.lines.len(), preview_lines);
}

#[test]
fn wb_edge_dxf_preview_text_count_match() {
    let csv = "No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n";
    let dxf = generate_dxf(csv);
    let dxf_parsed = dxf_engine::parse_dxf(&dxf).unwrap();

    let preview: Value = serde_json::from_str(&get_preview_data(csv)).unwrap();
    let preview_texts = preview["texts"].as_array().unwrap().len();

    assert_eq!(dxf_parsed.texts.len(), preview_texts,
        "DXF text count ({}) should match preview ({})",
        dxf_parsed.texts.len(), preview_texts);
}
