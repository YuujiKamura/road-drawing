//! Integration test: read a real CSV file from the original csv_to_dxf project data.

use std::path::Path;

use excel_parser::section_detector::extract_section_data_from_file;

#[test]
fn test_marking_e2e_real_file_kukan1_csv() {
    let path = Path::new(env!("HOMEPATH"))
        .join("StudioProjects")
        .join("csv_to_dxf")
        .join("data")
        .join("区間1.csv");

    if !path.exists() {
        // Skip on CI or machines without the test data
        eprintln!("SKIP: test data not found at {}", path.display());
        return;
    }

    let rows = extract_section_data_from_file(&path, "区間1")
        .expect("Failed to extract section data from 区間1.csv");

    assert_eq!(rows.len(), 5, "区間1.csv should have 5 data rows, got {}", rows.len());
    assert_eq!(rows[0].name, "No.0", "First station name should be No.0");
    assert!((rows[0].x - 0.0).abs() < 1e-9, "First station x should be 0.0");
    assert!((rows[0].wl - 0.80).abs() < 1e-9, "First station wl should be 0.80");
    assert!((rows[0].wr - 0.0).abs() < 1e-9, "First station wr should be 0.0");

    // Verify last row
    assert_eq!(rows[4].name, "0+7", "Last station name should be 0+7");
    assert!((rows[4].x - 7.0).abs() < 1e-9, "Last station x should be 7.0");
}
