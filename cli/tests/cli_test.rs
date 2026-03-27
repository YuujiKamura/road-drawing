//! CLI integration tests using real CSV fixtures.
//!
//! Runs the `road-drawing` binary via `std::process::Command` and verifies
//! stdout/stderr/exit-code and generated DXF output.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Path to the compiled binary
fn bin_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_BIN_EXE_road-drawing"));
    // Fallback: if the above doesn't resolve, try target/debug
    if !path.exists() {
        path = PathBuf::from("target/debug/road-drawing.exe");
    }
    path
}

/// Fixture directory (self-contained in repo)
fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("tests")
        .join("fixtures")
        .join("road-section")
}

fn skip_if_no_fixtures() -> bool {
    if !fixture_dir().exists() {
        eprintln!("SKIP: fixture dir not found at {}", fixture_dir().display());
        true
    } else {
        false
    }
}

// ================================================================
// --list-sections with multi-section CSV
// ================================================================

#[test]
fn test_marking_cli_list_sections_multi() {
    if skip_if_no_fixtures() {
        return;
    }

    let csv_path = fixture_dir().join("multi_section.csv");
    if !csv_path.exists() {
        eprintln!("SKIP: multi-section CSV not found");
        return;
    }

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            csv_path.to_str().unwrap(),
            "--output",
            "unused.dxf",
            "--list-sections",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI --list-sections should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sections: Vec<&str> = stdout.lines().collect();
    assert!(
        sections.len() >= 2,
        "Multi-section CSV should list at least 2 sections, got: {:?}",
        sections
    );
    assert!(
        sections.contains(&"区間1"),
        "Should list 区間1, got: {:?}",
        sections
    );
    assert!(
        sections.contains(&"区間3"),
        "Should list 区間3, got: {:?}",
        sections
    );
}

#[test]
fn test_marking_cli_list_sections_single_file() {
    if skip_if_no_fixtures() {
        return;
    }

    let csv_path = fixture_dir().join("区間1.csv");
    if !csv_path.exists() {
        eprintln!("SKIP: 区間1.csv not found");
        return;
    }

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            csv_path.to_str().unwrap(),
            "--output",
            "unused.dxf",
            "--list-sections",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI --list-sections should succeed for single-section CSV"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sections: Vec<&str> = stdout.lines().collect();
    assert!(
        !sections.is_empty(),
        "Should list at least one section"
    );
    // 区間1.csv filename → get_available_sections returns ["区間1"]
    assert!(
        sections.contains(&"区間1"),
        "Should list 区間1, got: {:?}",
        sections
    );
}

// ================================================================
// --type road-section: generate DXF from real CSV
// ================================================================

#[test]
fn test_marking_cli_generate_road_section() {
    if skip_if_no_fixtures() {
        return;
    }

    let csv_path = fixture_dir().join("data.csv");
    if !csv_path.exists() {
        eprintln!("SKIP: data.csv not found");
        return;
    }

    let tmp_dir = std::env::temp_dir();
    let output_path = tmp_dir.join("test_cli_road_section.dxf");

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            csv_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--type",
            "road-section",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI generate road-section should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "Output DXF file should be created");

    let dxf_content = fs::read_to_string(&output_path).unwrap();
    assert!(dxf_content.contains("SECTION"), "DXF should have SECTION");
    assert!(dxf_content.contains("LINE"), "DXF should have LINE entities");
    assert!(dxf_content.contains("EOF"), "DXF should end with EOF");

    // Validate with linter
    assert!(
        dxf_engine::DxfLinter::is_valid(&dxf_content),
        "Generated DXF must pass linter"
    );

    fs::remove_file(&output_path).ok();
}

// ================================================================
// --type triangle: generate DXF from inline triangle CSV
// ================================================================

#[test]
fn test_marking_cli_generate_triangle() {
    // Create a minimal triangle CSV fixture (no external file needed)
    let triangle_csv = "\
koujiname,テスト工事
rosenname,テスト路線
gyousyaname,テスト
zumennum,1
1,10.0,8.0,6.0,-1,-1
2,5.0,4.0,3.0,1,1
3,7.0,6.0,5.0,1,2
";

    let tmp_dir = std::env::temp_dir();
    let input_path = tmp_dir.join("test_cli_triangle_input.csv");
    let output_path = tmp_dir.join("test_cli_triangle_output.dxf");

    fs::write(&input_path, triangle_csv).unwrap();

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--type",
            "triangle",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI generate triangle should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "Output DXF file should be created");

    let dxf_content = fs::read_to_string(&output_path).unwrap();
    assert!(dxf_content.contains("SECTION"), "DXF should have SECTION");
    assert!(dxf_content.contains("LINE"), "DXF should have LINE entities");
    assert!(dxf_content.contains("TEXT"), "DXF should have TEXT entities");
    assert!(dxf_content.contains("EOF"), "DXF should end with EOF");

    // Validate with linter
    assert!(
        dxf_engine::DxfLinter::is_valid(&dxf_content),
        "Triangle DXF must pass linter"
    );

    // Check stderr for triangle info
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("3 triangles"),
        "Should report 3 triangles in stderr: {}",
        stderr
    );

    fs::remove_file(&input_path).ok();
    fs::remove_file(&output_path).ok();
}

// ================================================================
// --type triangle: verify area text in DXF
// ================================================================

#[test]
fn test_marking_cli_triangle_area_in_dxf() {
    // Single triangle: sides 3,4,5 → right triangle, area = 6.0
    let triangle_csv = "\
koujiname,面積テスト
rosenname,test
gyousyaname,test
zumennum,1
1,5.0,4.0,3.0,-1,-1
";

    let tmp_dir = std::env::temp_dir();
    let input_path = tmp_dir.join("test_cli_triangle_area_input.csv");
    let output_path = tmp_dir.join("test_cli_triangle_area_output.dxf");

    fs::write(&input_path, triangle_csv).unwrap();

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--type",
            "triangle",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());

    let dxf_content = fs::read_to_string(&output_path).unwrap();

    // Parse DXF and check area text = "6" (3-4-5 right triangle)
    let doc = dxf_engine::parse_dxf(&dxf_content).unwrap();
    let area_texts: Vec<_> = doc
        .texts
        .iter()
        .filter(|t| t.text.parse::<f64>().is_ok())
        .collect();
    assert!(
        !area_texts.is_empty(),
        "DXF should contain area text entities"
    );
    let area_val: f64 = area_texts[0].text.parse().unwrap();
    assert!(
        (area_val - 6.0).abs() < 0.01,
        "3-4-5 triangle area should be 6.0, got {}",
        area_val
    );

    fs::remove_file(&input_path).ok();
    fs::remove_file(&output_path).ok();
}

// ================================================================
// --section: generate DXF from multi-section CSV with section selection
// ================================================================

#[test]
fn test_cli_section_flag_generates_dxf() {
    if skip_if_no_fixtures() {
        return;
    }

    let csv_path = fixture_dir().join("multi_section.csv");
    if !csv_path.exists() {
        eprintln!("SKIP: multi-section CSV not found");
        return;
    }

    let tmp_dir = std::env::temp_dir();
    let output_path = tmp_dir.join("test_cli_section_flag.dxf");

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            csv_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--section",
            "区間3",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI --section 区間3 should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "Output DXF file should be created");

    let dxf_content = fs::read_to_string(&output_path).unwrap();
    assert!(dxf_content.contains("LINE"), "DXF should have LINE entities");
    assert!(
        dxf_engine::DxfLinter::is_valid(&dxf_content),
        "Generated DXF must pass linter"
    );

    // stderr should mention section name
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("区間3"),
        "stderr should mention section name: {}",
        stderr
    );

    fs::remove_file(&output_path).ok();
}

#[test]
fn test_cli_section_flag_single_csv() {
    if skip_if_no_fixtures() {
        return;
    }

    let csv_path = fixture_dir().join("区間1.csv");
    if !csv_path.exists() {
        eprintln!("SKIP: 区間1.csv not found");
        return;
    }

    let tmp_dir = std::env::temp_dir();
    let output_path = tmp_dir.join("test_cli_section_single.dxf");

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            csv_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--section",
            "区間1",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI --section 区間1 with single CSV should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists());
    let dxf_content = fs::read_to_string(&output_path).unwrap();
    assert!(dxf_engine::DxfLinter::is_valid(&dxf_content));

    fs::remove_file(&output_path).ok();
}

// ================================================================
// --type triangle: real connected.csv from trianglelist test resources
// ================================================================

/// Triangle fixture directory (self-contained in repo)
fn trianglelist_fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("tests")
        .join("fixtures")
        .join("triangle")
}

#[test]
fn test_cli_triangle_connected_csv() {
    let fixture_dir = trianglelist_fixture_dir();
    let csv_path = fixture_dir.join("connected.csv");
    if !csv_path.exists() {
        eprintln!("SKIP: connected.csv not found at {}", csv_path.display());
        return;
    }

    let tmp_dir = std::env::temp_dir();
    let output_path = tmp_dir.join("test_cli_triangle_connected.dxf");

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            csv_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--type",
            "triangle",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI --type triangle connected.csv should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "Output DXF file should be created");

    let dxf_content = fs::read_to_string(&output_path).unwrap();

    // Structural checks
    assert!(dxf_content.contains("SECTION"), "DXF should have SECTION");
    assert!(dxf_content.contains("LINE"), "DXF should have LINE entities");
    assert!(dxf_content.contains("TEXT"), "DXF should have TEXT entities");
    assert!(dxf_content.contains("EOF"), "DXF should end with EOF");

    // Lint validation
    assert!(
        dxf_engine::DxfLinter::is_valid(&dxf_content),
        "connected.csv triangle DXF must pass linter"
    );

    // 7 triangles → 21 lines (3 edges each) + 14 texts (area + number each)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("7 triangles"),
        "Should report 7 triangles: {}",
        stderr
    );

    // Parse DXF and verify entity counts
    let doc = dxf_engine::parse_dxf(&dxf_content).unwrap();
    assert_eq!(doc.lines.len(), 21, "7 triangles × 3 edges = 21 lines");
    assert_eq!(doc.texts.len(), 14, "7 triangles × 2 texts = 14 texts");

    // Verify header info is printed
    assert!(
        stderr.contains("接続形式テスト") || stderr.contains("テスト路線"),
        "Should print header info: {}",
        stderr
    );

    fs::remove_file(&output_path).ok();
}

#[test]
fn test_cli_triangle_connected_csv_areas() {
    let fixture_dir = trianglelist_fixture_dir();
    let csv_path = fixture_dir.join("connected.csv");
    if !csv_path.exists() {
        eprintln!("SKIP: connected.csv not found");
        return;
    }

    let tmp_dir = std::env::temp_dir();
    let output_path = tmp_dir.join("test_cli_triangle_connected_areas.dxf");

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            csv_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--type",
            "triangle",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(output.status.success());

    let dxf_content = fs::read_to_string(&output_path).unwrap();
    let doc = dxf_engine::parse_dxf(&dxf_content).unwrap();

    // Extract area texts (height=0.3, color=7). Triangle numbers have height=0.4, color=5.
    let area_texts: Vec<f64> = doc
        .texts
        .iter()
        .filter(|t| (t.height - 0.3).abs() < 0.01 && t.color == 7)
        .filter_map(|t| t.text.parse::<f64>().ok())
        .collect();

    assert_eq!(area_texts.len(), 7, "Should have 7 area values");

    // All areas must be positive
    for (i, &area) in area_texts.iter().enumerate() {
        assert!(area > 0.0, "Triangle {} area should be > 0, got {}", i + 1, area);
    }

    // Total area from stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("total area:"),
        "Should report total area: {}",
        stderr
    );

    fs::remove_file(&output_path).ok();
}

#[test]
fn test_cli_triangle_minimal_csv() {
    let fixture_dir = trianglelist_fixture_dir();
    let csv_path = fixture_dir.join("minimal.csv");
    if !csv_path.exists() {
        eprintln!("SKIP: minimal.csv not found");
        return;
    }

    let tmp_dir = std::env::temp_dir();
    let output_path = tmp_dir.join("test_cli_triangle_minimal.dxf");

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            csv_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--type",
            "triangle",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "CLI --type triangle minimal.csv should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let dxf_content = fs::read_to_string(&output_path).unwrap();
    assert!(dxf_engine::DxfLinter::is_valid(&dxf_content));

    let doc = dxf_engine::parse_dxf(&dxf_content).unwrap();
    // 3 independent triangles → 9 lines + 6 texts
    assert_eq!(doc.lines.len(), 9, "3 triangles × 3 edges = 9 lines");
    assert_eq!(doc.texts.len(), 6, "3 triangles × 2 texts = 6 texts");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("3 triangles"), "Should report 3 triangles: {}", stderr);

    fs::remove_file(&output_path).ok();
}

// ================================================================
// Unknown --type should fail
// ================================================================

#[test]
fn test_marking_cli_unknown_type_fails() {
    let tmp_dir = std::env::temp_dir();
    let input_path = tmp_dir.join("test_cli_dummy.csv");
    fs::write(&input_path, "dummy").unwrap();

    let output = Command::new(bin_path())
        .args([
            "generate",
            "--input",
            input_path.to_str().unwrap(),
            "--output",
            "unused.dxf",
            "--type",
            "nonexistent",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        !output.status.success(),
        "Unknown type should fail with non-zero exit"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown"),
        "Error should mention 'Unknown': {}",
        stderr
    );

    fs::remove_file(&input_path).ok();
}
