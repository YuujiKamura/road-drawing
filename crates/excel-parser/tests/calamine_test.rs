//! Integration tests: calamine xlsx reading
//!
//! No real .xlsx files in csv_to_dxf/data/, so we programmatically create
//! xlsx workbooks with rust_xlsxwriter, then read them back with calamine.

use std::io::Cursor;

use calamine::{open_workbook_from_rs, Reader, Xlsx};
use rust_xlsxwriter::Workbook;

use excel_parser::section_detector::extract_section_data;

/// Helper: create xlsx bytes from rows of string data
fn create_xlsx_bytes(sheet_name: &str, rows: &[Vec<&str>]) -> Vec<u8> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet().set_name(sheet_name).unwrap();

    for (r, row) in rows.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            // Try as f64 first, fall back to string
            if let Ok(num) = cell.parse::<f64>() {
                worksheet.write_number(r as u32, c as u16, num).unwrap();
            } else {
                worksheet.write_string(r as u32, c as u16, *cell).unwrap();
            }
        }
    }

    workbook.save_to_buffer().unwrap()
}

/// Helper: read xlsx bytes with calamine and return sheet content as CSV-like string
fn xlsx_to_csv_string(bytes: &[u8], sheet_name: &str) -> String {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor).expect("Failed to open xlsx");

    let range = workbook
        .worksheet_range(sheet_name)
        .expect("Sheet not found");

    let mut csv = String::new();
    for row in range.rows() {
        let cells: Vec<String> = row
            .iter()
            .map(|cell| match cell {
                calamine::Data::Float(f) => format!("{f}"),
                calamine::Data::Int(i) => format!("{i}"),
                calamine::Data::String(s) => s.clone(),
                calamine::Data::Empty => String::new(),
                calamine::Data::Bool(b) => format!("{b}"),
                calamine::Data::DateTime(dt) => format!("{dt}"),
                calamine::Data::Error(e) => format!("{e:?}"),
                calamine::Data::DateTimeIso(s) => s.clone(),
                calamine::Data::DurationIso(s) => s.clone(),
            })
            .collect();
        csv.push_str(&cells.join(","));
        csv.push('\n');
    }
    csv
}

// ================================================================
// Basic: create xlsx, read with calamine, verify cell values
// ================================================================

#[test]
fn test_marking_calamine_read_basic_xlsx() {
    let rows = vec![
        vec!["name", "x", "wl", "wr"],
        vec!["No.0", "0", "2.5", "2.5"],
        vec!["No.1", "20", "3.0", "3.0"],
    ];
    let bytes = create_xlsx_bytes("Sheet1", &rows);

    let cursor = Cursor::new(&bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor).unwrap();

    let sheets = workbook.sheet_names().to_vec();
    assert_eq!(sheets, vec!["Sheet1"]);

    let range = workbook.worksheet_range("Sheet1").unwrap();
    assert_eq!(range.get_size().0, 3, "Should have 3 rows (header + 2 data)");
    assert_eq!(range.get_size().1, 4, "Should have 4 columns");
}

// ================================================================
// Round-trip: xlsx → calamine → CSV string → extract_section_data
// ================================================================

#[test]
fn test_marking_calamine_xlsx_to_section_data() {
    let rows = vec![
        vec!["name", "x", "wl", "wr"],
        vec!["No.0", "0", "0.8", "0"],
        vec!["0+1.2", "1.15", "0.63", "0"],
        vec!["0+3.2", "3.25", "0.5", "0"],
        vec!["0+5.4", "5.4", "0.5", "0"],
        vec!["0+7", "7", "0.5", "0"],
    ];
    let bytes = create_xlsx_bytes("区間1", &rows);
    let csv = xlsx_to_csv_string(&bytes, "区間1");

    let section_rows = extract_section_data(&csv, "区間1").unwrap();
    assert_eq!(section_rows.len(), 5, "Should have 5 data rows");
    assert_eq!(section_rows[0].name, "No.0");
    assert!((section_rows[0].wl - 0.8).abs() < 1e-9);
    assert_eq!(section_rows[4].name, "0+7");
    assert!((section_rows[4].x - 7.0).abs() < 1e-9);
}

// ================================================================
// Japanese headers in xlsx
// ================================================================

#[test]
fn test_marking_calamine_xlsx_japanese_headers() {
    let rows = vec![
        vec!["測点名", "単延長L", "幅員W", "平均幅員Wa", "面積m2"],
        vec!["No.0", "0", "0.8", "", ""],
        vec!["", "1.15", "0.63", "0.715", "0.82"],
    ];
    let bytes = create_xlsx_bytes("Sheet1", &rows);
    let csv = xlsx_to_csv_string(&bytes, "Sheet1");

    let section_rows = extract_section_data(&csv, "区間1").unwrap();
    assert_eq!(section_rows.len(), 2);
    assert_eq!(section_rows[0].name, "No.0");
    assert!((section_rows[0].wl - 0.8).abs() < 1e-9);
}

// ================================================================
// Multiple sheets (section detection)
// ================================================================

#[test]
fn test_marking_calamine_xlsx_multiple_sheets() {
    let rows = vec![
        vec!["name", "x", "wl", "wr"],
        vec!["No.0", "0", "1.0", "1.0"],
    ];
    let bytes = create_xlsx_bytes("区間1", &rows);

    let cursor = Cursor::new(&bytes);
    let workbook: Xlsx<_> = open_workbook_from_rs(cursor).unwrap();

    let sheets = workbook.sheet_names().to_vec();
    assert!(sheets.contains(&"区間1".to_string()));
}

// ================================================================
// Empty sheet
// ================================================================

#[test]
fn test_marking_calamine_xlsx_empty_sheet() {
    let rows: Vec<Vec<&str>> = vec![];
    let bytes = create_xlsx_bytes("Empty", &rows);

    let cursor = Cursor::new(&bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor).unwrap();

    let range = workbook.worksheet_range("Empty").unwrap();
    assert_eq!(range.get_size(), (0, 0), "Empty sheet should have 0x0 size");
}

// ================================================================
// xlsx with section header pattern (区間X,台形計算)
// ================================================================

#[test]
fn test_marking_calamine_xlsx_section_header_pattern() {
    let rows = vec![
        vec!["区間1", "台形計算", "", "", ""],
        vec!["測点名", "単延長L", "幅員W", "平均幅員Wa", "面積m2"],
        vec!["No.0", "0", "0.8", "", ""],
        vec!["", "1.15", "0.63", "0.715", "0.82"],
    ];
    let bytes = create_xlsx_bytes("Sheet1", &rows);
    let csv = xlsx_to_csv_string(&bytes, "Sheet1");

    let sections = excel_parser::section_detector::get_available_sections(&csv, "sheet.xlsx");
    assert_eq!(sections, vec!["区間1"]);

    let section_rows = extract_section_data(&csv, "区間1").unwrap();
    assert_eq!(section_rows.len(), 2);
    assert_eq!(section_rows[0].name, "No.0");
}

// ================================================================
// Full pipeline: xlsx → calamine → CSV → extract → transform
// ================================================================

#[test]
fn test_marking_calamine_xlsx_full_pipeline() {
    let rows = vec![
        vec!["測点名", "単延長L", "幅員W", "平均幅員Wa", "面積m2"],
        vec!["No.0", "0", "0.8", "", ""],
        vec!["", "1.15", "0.63", "0.715", "0.82"],
        vec!["", "2.1", "0.5", "0.565", "1.19"],
        vec!["", "2.15", "0.5", "0.5", "1.08"],
        vec!["0+7", "1.6", "0.5", "0.5", "0.8"],
    ];
    let bytes = create_xlsx_bytes("Sheet1", &rows);
    let csv = xlsx_to_csv_string(&bytes, "Sheet1");

    let result = excel_parser::transform::extract_and_transform_text(&csv, "区間1");
    let transformed = result.unwrap();
    assert_eq!(transformed.len(), 5);

    // After transform: cumulative distances, filled names, rounded values
    assert_eq!(transformed[0].name, "No.0");
    assert!((transformed[0].x - 0.0).abs() < 0.01);
    assert!((transformed[4].x - 7.0).abs() < 0.01);
    assert_eq!(transformed[4].name, "0+7");
}
