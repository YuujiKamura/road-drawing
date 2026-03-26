//! Compatibility tests: verify dxf-engine exports match the old rust-dxf API
//! used by trianglelist-web.
//!
//! trianglelist-web imports:
//!   use dxf::{DxfLine, DxfText, HorizontalAlignment, VerticalAlignment};
//!   dxf::DxfWriter::new()
//!   dxf::HorizontalAlignment::Left / Center / Right
//!   dxf::VerticalAlignment::Baseline / Bottom / Middle / Top
//!
//! All test functions prefixed with test_compat_.

use dxf_engine::{DxfLine, DxfText, DxfWriter, HorizontalAlignment, VerticalAlignment};

// ================================================================
// DxfLine API parity
// ================================================================

#[test]
fn test_compat_dxf_line_new() {
    let line = DxfLine::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(line.x1, 1.0);
    assert_eq!(line.y1, 2.0);
    assert_eq!(line.x2, 3.0);
    assert_eq!(line.y2, 4.0);
    assert_eq!(line.color, 7); // default white
    assert_eq!(line.layer, "0"); // default layer
}

#[test]
fn test_compat_dxf_line_with_style() {
    let line = DxfLine::with_style(0.0, 0.0, 10.0, 10.0, 5, "Layer1");
    assert_eq!(line.color, 5);
    assert_eq!(line.layer, "Layer1");
}

#[test]
fn test_compat_dxf_line_builder() {
    let line = DxfLine::new(0.0, 0.0, 10.0, 10.0)
        .color(3)
        .layer("MyLayer");
    assert_eq!(line.color, 3);
    assert_eq!(line.layer, "MyLayer");
}

// ================================================================
// DxfText API parity
// ================================================================

#[test]
fn test_compat_dxf_text_new() {
    let text = DxfText::new(50.0, 50.0, "Hello");
    assert_eq!(text.x, 50.0);
    assert_eq!(text.y, 50.0);
    assert_eq!(text.text, "Hello");
    assert_eq!(text.height, 1.0);
    assert_eq!(text.rotation, 0.0);
    assert_eq!(text.color, 7);
    assert_eq!(text.align_h, HorizontalAlignment::Left);
    assert_eq!(text.align_v, VerticalAlignment::Baseline);
}

#[test]
fn test_compat_dxf_text_builder() {
    let text = DxfText::new(0.0, 0.0, "Test")
        .height(2.5)
        .rotation(-90.0)
        .color(5)
        .align_h(HorizontalAlignment::Center)
        .align_v(VerticalAlignment::Middle)
        .layer("TextLayer");
    assert_eq!(text.height, 2.5);
    assert_eq!(text.rotation, -90.0);
    assert_eq!(text.color, 5);
    assert_eq!(text.align_h, HorizontalAlignment::Center);
    assert_eq!(text.align_v, VerticalAlignment::Middle);
    assert_eq!(text.layer, "TextLayer");
}

// ================================================================
// Alignment enum values (repr(i32) parity with old dxf crate)
// ================================================================

#[test]
fn test_compat_horizontal_alignment_values() {
    assert_eq!(HorizontalAlignment::Left as i32, 0);
    assert_eq!(HorizontalAlignment::Center as i32, 1);
    assert_eq!(HorizontalAlignment::Right as i32, 2);
}

#[test]
fn test_compat_vertical_alignment_values() {
    assert_eq!(VerticalAlignment::Baseline as i32, 0);
    assert_eq!(VerticalAlignment::Bottom as i32, 1);
    assert_eq!(VerticalAlignment::Middle as i32, 2);
    assert_eq!(VerticalAlignment::Top as i32, 3);
}

// ================================================================
// Alignment exhaustive match (trianglelist-web does match on all variants)
// ================================================================

#[test]
fn test_compat_horizontal_alignment_match() {
    // trianglelist-web matches: Left, Center, Right
    for &h in &[HorizontalAlignment::Left, HorizontalAlignment::Center, HorizontalAlignment::Right] {
        let _name = match h {
            HorizontalAlignment::Left => "left",
            HorizontalAlignment::Center => "center",
            HorizontalAlignment::Right => "right",
        };
    }
}

#[test]
fn test_compat_vertical_alignment_match() {
    // trianglelist-web matches: Top, Middle, Bottom | Baseline
    for &v in &[VerticalAlignment::Baseline, VerticalAlignment::Bottom,
                VerticalAlignment::Middle, VerticalAlignment::Top] {
        let _name = match v {
            VerticalAlignment::Top => "top",
            VerticalAlignment::Middle => "middle",
            VerticalAlignment::Bottom | VerticalAlignment::Baseline => "bottom",
        };
    }
}

// ================================================================
// DxfWriter API parity
// ================================================================

#[test]
fn test_compat_dxf_writer_roundtrip() {
    // trianglelist-web: let writer = dxf::DxfWriter::new();
    //                   writer.write(&lines, &texts)
    let lines = vec![
        DxfLine::new(0.0, 0.0, 100.0, 100.0),
        DxfLine::with_style(0.0, 100.0, 100.0, 0.0, 3, "Layer1"),
    ];
    let texts = vec![
        DxfText::new(50.0, 50.0, "Center")
            .align_h(HorizontalAlignment::Center)
            .align_v(VerticalAlignment::Middle),
    ];

    let writer = DxfWriter::new();
    let content = writer.write(&lines, &texts);

    assert!(content.contains("LINE"), "DXF should contain LINE entity");
    assert!(content.contains("TEXT"), "DXF should contain TEXT entity");
    assert!(content.contains("Center"), "DXF should contain text content");
    assert!(content.contains("Layer1"), "DXF should contain layer name");
    assert!(content.contains("EOF"), "DXF should end with EOF");
}
