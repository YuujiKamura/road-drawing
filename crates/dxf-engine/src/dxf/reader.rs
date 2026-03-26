//! DXF reader: parse DXF text format into entity structs
//!
//! Parses ENTITIES section from DXF text format, extracting:
//! - LINE → DxfLine
//! - TEXT → DxfText
//! - CIRCLE → DxfCircle
//! - LWPOLYLINE → DxfLwPolyline

use super::entities::{DxfLine, DxfText, DxfCircle, DxfLwPolyline, HorizontalAlignment, VerticalAlignment};

/// Parsed DXF document
#[derive(Clone, Debug, Default)]
pub struct DxfDocument {
    pub lines: Vec<DxfLine>,
    pub texts: Vec<DxfText>,
    pub circles: Vec<DxfCircle>,
    pub polylines: Vec<DxfLwPolyline>,
}

/// Parse error
#[derive(Debug)]
pub enum ReaderError {
    NoEntitiesSection,
    MalformedGroupCode(String),
}

impl std::fmt::Display for ReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReaderError::NoEntitiesSection => write!(f, "No ENTITIES section found"),
            ReaderError::MalformedGroupCode(s) => write!(f, "Malformed group code: {}", s),
        }
    }
}

/// Parse DXF text content into a DxfDocument
pub fn parse_dxf(content: &str) -> Result<DxfDocument, ReaderError> {
    // Collect all lines, trimming each
    let raw_lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();

    // Find ENTITIES section
    let entities_start = find_section_start(&raw_lines, "ENTITIES")
        .ok_or(ReaderError::NoEntitiesSection)?;

    let mut doc = DxfDocument::default();
    let mut i = entities_start;

    while i + 1 < raw_lines.len() {
        let code = raw_lines[i];
        let value = raw_lines[i + 1];

        // End of section
        if code == "0" && value == "ENDSEC" {
            break;
        }

        if code == "0" {
            match value {
                "LINE" => {
                    let (line, next) = parse_line_entity(&raw_lines, i + 2);
                    doc.lines.push(line);
                    i = next;
                }
                "TEXT" => {
                    let (text, next) = parse_text_entity(&raw_lines, i + 2);
                    doc.texts.push(text);
                    i = next;
                }
                "CIRCLE" => {
                    let (circle, next) = parse_circle_entity(&raw_lines, i + 2);
                    doc.circles.push(circle);
                    i = next;
                }
                "LWPOLYLINE" => {
                    let (poly, next) = parse_lwpolyline_entity(&raw_lines, i + 2);
                    doc.polylines.push(poly);
                    i = next;
                }
                _ => {
                    i += 2; // skip unknown entity type marker
                }
            }
        } else {
            i += 2;
        }
    }

    Ok(doc)
}

/// Find the start index after "2\n<section_name>" inside a SECTION
fn find_section_start(lines: &[&str], section_name: &str) -> Option<usize> {
    let mut i = 0;
    while i + 3 < lines.len() {
        if lines[i] == "0" && lines[i + 1] == "SECTION"
            && lines[i + 2] == "2" && lines[i + 3] == section_name
        {
            return Some(i + 4);
        }
        i += 1;
    }
    None
}

/// Read group code pairs until we hit "0" (next entity or ENDSEC).
/// Returns (entity, next_index).
fn parse_line_entity(lines: &[&str], start: usize) -> (DxfLine, usize) {
    let mut line = DxfLine::default();
    let mut i = start;

    while i + 1 < lines.len() {
        let code = lines[i];
        let value = lines[i + 1];

        if code == "0" {
            break; // next entity
        }

        match code {
            "8" => line.layer = value.to_string(),
            "62" => line.color = value.parse().unwrap_or(7),
            "10" => line.x1 = value.parse().unwrap_or(0.0),
            "20" => line.y1 = value.parse().unwrap_or(0.0),
            "11" => line.x2 = value.parse().unwrap_or(0.0),
            "21" => line.y2 = value.parse().unwrap_or(0.0),
            _ => {} // skip handles, subclass markers, etc.
        }
        i += 2;
    }

    (line, i)
}

fn parse_text_entity(lines: &[&str], start: usize) -> (DxfText, usize) {
    let mut text = DxfText::default();
    let mut i = start;

    while i + 1 < lines.len() {
        let code = lines[i];
        let value = lines[i + 1];

        if code == "0" {
            break;
        }

        match code {
            "8" => text.layer = value.to_string(),
            "62" => text.color = value.parse().unwrap_or(7),
            "10" => text.x = value.parse().unwrap_or(0.0),
            "20" => text.y = value.parse().unwrap_or(0.0),
            "40" => text.height = value.parse().unwrap_or(1.0),
            "1" => text.text = value.to_string(),
            "50" => text.rotation = value.parse().unwrap_or(0.0),
            "72" => {
                text.align_h = match value.parse().unwrap_or(0) {
                    1 => HorizontalAlignment::Center,
                    2 => HorizontalAlignment::Right,
                    _ => HorizontalAlignment::Left,
                };
            }
            "73" => {
                text.align_v = match value.parse().unwrap_or(0) {
                    1 => VerticalAlignment::Bottom,
                    2 => VerticalAlignment::Middle,
                    3 => VerticalAlignment::Top,
                    _ => VerticalAlignment::Baseline,
                };
            }
            _ => {}
        }
        i += 2;
    }

    (text, i)
}

fn parse_circle_entity(lines: &[&str], start: usize) -> (DxfCircle, usize) {
    let mut circle = DxfCircle::default();
    let mut i = start;

    while i + 1 < lines.len() {
        let code = lines[i];
        let value = lines[i + 1];

        if code == "0" {
            break;
        }

        match code {
            "8" => circle.layer = value.to_string(),
            "62" => circle.color = value.parse().unwrap_or(7),
            "10" => circle.x = value.parse().unwrap_or(0.0),
            "20" => circle.y = value.parse().unwrap_or(0.0),
            "40" => circle.radius = value.parse().unwrap_or(1.0),
            _ => {}
        }
        i += 2;
    }

    (circle, i)
}

fn parse_lwpolyline_entity(lines: &[&str], start: usize) -> (DxfLwPolyline, usize) {
    let mut poly = DxfLwPolyline::default();
    let mut i = start;
    let mut current_x: Option<f64> = None;

    while i + 1 < lines.len() {
        let code = lines[i];
        let value = lines[i + 1];

        if code == "0" {
            break;
        }

        match code {
            "8" => poly.layer = value.to_string(),
            "62" => poly.color = value.parse().unwrap_or(7),
            "70" => poly.closed = value.parse().unwrap_or(0) == 1,
            "10" => {
                // Flush previous vertex if we have a pending x
                if let Some(x) = current_x {
                    poly.vertices.push((x, 0.0)); // y=0 fallback
                }
                current_x = Some(value.parse().unwrap_or(0.0));
            }
            "20" => {
                if let Some(x) = current_x.take() {
                    let y: f64 = value.parse().unwrap_or(0.0);
                    poly.vertices.push((x, y));
                }
            }
            _ => {}
        }
        i += 2;
    }

    // Flush last vertex if pending
    if let Some(x) = current_x {
        poly.vertices.push((x, 0.0));
    }

    (poly, i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dxf::writer::DxfWriter;

    // ================================================================
    // Roundtrip: Writer output → Reader parse
    // ================================================================

    #[test]
    fn test_roundtrip_single_line() {
        let lines = vec![DxfLine::new(1.0, 2.0, 3.0, 4.0)];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &[]);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines.len(), 1);
        assert!((doc.lines[0].x1 - 1.0).abs() < 0.001);
        assert!((doc.lines[0].y1 - 2.0).abs() < 0.001);
        assert!((doc.lines[0].x2 - 3.0).abs() < 0.001);
        assert!((doc.lines[0].y2 - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip_line_with_style() {
        let lines = vec![DxfLine::with_style(10.0, 20.0, 30.0, 40.0, 5, "道路中心")];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &[]);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.lines[0].color, 5);
        assert_eq!(doc.lines[0].layer, "道路中心");
    }

    #[test]
    fn test_roundtrip_multiple_lines() {
        let lines = vec![
            DxfLine::new(0.0, 0.0, 10.0, 10.0),
            DxfLine::new(10.0, 10.0, 20.0, 20.0),
            DxfLine::new(20.0, 20.0, 30.0, 30.0),
        ];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &[]);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines.len(), 3);
    }

    #[test]
    fn test_roundtrip_single_text() {
        let texts = vec![DxfText::new(50.0, 60.0, "No.0")];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&[], &texts);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.texts.len(), 1);
        assert!((doc.texts[0].x - 50.0).abs() < 0.001);
        assert!((doc.texts[0].y - 60.0).abs() < 0.001);
        assert_eq!(doc.texts[0].text, "No.0");
    }

    #[test]
    fn test_roundtrip_text_with_height_rotation() {
        let texts = vec![
            DxfText::new(100.0, 200.0, "測点名")
                .height(350.0)
                .rotation(-90.0)
                .color(5)
        ];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&[], &texts);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.texts.len(), 1);
        assert!((doc.texts[0].height - 350.0).abs() < 0.001);
        assert!((doc.texts[0].rotation - (-90.0)).abs() < 0.001);
        assert_eq!(doc.texts[0].color, 5);
    }

    #[test]
    fn test_roundtrip_mixed_entities() {
        let lines = vec![
            DxfLine::new(0.0, 0.0, 100.0, 0.0),
            DxfLine::new(0.0, 0.0, 0.0, 100.0),
        ];
        let texts = vec![
            DxfText::new(50.0, 50.0, "Center"),
            DxfText::new(0.0, 0.0, "Origin"),
        ];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &texts);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines.len(), 2);
        assert_eq!(doc.texts.len(), 2);
    }

    #[test]
    fn test_roundtrip_circle() {
        let circles = vec![DxfCircle::new(10.0, 20.0, 5.0)];
        let mut writer = DxfWriter::new();
        let dxf_text = writer.write_all(&[], &[], &circles, &[]);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.circles.len(), 1);
        assert!((doc.circles[0].x - 10.0).abs() < 0.001);
        assert!((doc.circles[0].y - 20.0).abs() < 0.001);
        assert!((doc.circles[0].radius - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip_lwpolyline() {
        let polylines = vec![
            DxfLwPolyline::new(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]),
        ];
        let mut writer = DxfWriter::new();
        let dxf_text = writer.write_all(&[], &[], &[], &polylines);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.polylines.len(), 1);
        assert_eq!(doc.polylines[0].vertices.len(), 3);
        assert!(!doc.polylines[0].closed);
    }

    // ================================================================
    // Edge cases
    // ================================================================

    #[test]
    fn test_parse_empty_entities_section() {
        // Minimal DXF with empty ENTITIES
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert!(doc.lines.is_empty());
        assert!(doc.texts.is_empty());
    }

    #[test]
    fn test_parse_no_entities_section() {
        let dxf = "0\nSECTION\n2\nHEADER\n0\nENDSEC\n0\nEOF\n";
        let result = parse_dxf(dxf);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_japanese_text() {
        let texts = vec![DxfText::new(0.0, 0.0, "横断歩道")];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&[], &texts);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.texts[0].text, "横断歩道");
    }

    #[test]
    fn test_roundtrip_preserves_layer_names() {
        let lines = vec![
            DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 3, "中心線"),
            DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 5, "横断歩道"),
        ];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &[]);

        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines[0].layer, "中心線");
        assert_eq!(doc.lines[1].layer, "横断歩道");
    }

    // ================================================================
    // Additional roundtrip tests
    // ================================================================

    #[test]
    fn test_roundtrip_line_negative_coords() {
        let lines = vec![DxfLine::new(-100.0, -200.0, -300.0, -400.0)];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &[]);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert!((doc.lines[0].x1 - (-100.0)).abs() < 0.001);
        assert!((doc.lines[0].y2 - (-400.0)).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip_line_zero_length() {
        let lines = vec![DxfLine::new(5.0, 5.0, 5.0, 5.0)];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &[]);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert!((doc.lines[0].x1 - doc.lines[0].x2).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip_text_all_fields() {
        use crate::dxf::entities::{HorizontalAlignment, VerticalAlignment};
        let texts = vec![DxfText::new(10.0, 20.0, "Full")
            .height(5.0)
            .rotation(45.0)
            .color(3)
            .align_h(HorizontalAlignment::Center)
            .align_v(VerticalAlignment::Middle)
            .layer("MyLayer")];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&[], &texts);
        let doc = parse_dxf(&dxf_text).unwrap();
        let t = &doc.texts[0];
        assert!((t.x - 10.0).abs() < 0.001);
        assert!((t.y - 20.0).abs() < 0.001);
        assert_eq!(t.text, "Full");
        assert!((t.height - 5.0).abs() < 0.001);
        assert!((t.rotation - 45.0).abs() < 0.001);
        assert_eq!(t.color, 3);
        assert_eq!(t.align_h, HorizontalAlignment::Center);
        assert_eq!(t.align_v, VerticalAlignment::Middle);
        assert_eq!(t.layer, "MyLayer");
    }

    #[test]
    fn test_roundtrip_text_default_alignment() {
        let texts = vec![DxfText::new(0.0, 0.0, "Default")];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&[], &texts);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.texts[0].align_h, HorizontalAlignment::Left);
        assert_eq!(doc.texts[0].align_v, VerticalAlignment::Baseline);
    }

    #[test]
    fn test_roundtrip_circle_with_style() {
        let circles = vec![DxfCircle::new(100.0, 200.0, 50.0).color(3).layer("Circles")];
        let mut writer = DxfWriter::new();
        let dxf_text = writer.write_all(&[], &[], &circles, &[]);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.circles[0].color, 3);
        assert_eq!(doc.circles[0].layer, "Circles");
    }

    #[test]
    fn test_roundtrip_lwpolyline_closed() {
        let polylines = vec![DxfLwPolyline::closed(vec![
            (0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0),
        ])];
        let mut writer = DxfWriter::new();
        let dxf_text = writer.write_all(&[], &[], &[], &polylines);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert!(doc.polylines[0].closed);
        assert_eq!(doc.polylines[0].vertices.len(), 4);
    }

    #[test]
    fn test_roundtrip_lwpolyline_with_style() {
        let polylines = vec![DxfLwPolyline::new(vec![(1.0, 2.0), (3.0, 4.0)])
            .color(5)
            .layer("Outline")];
        let mut writer = DxfWriter::new();
        let dxf_text = writer.write_all(&[], &[], &[], &polylines);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.polylines[0].color, 5);
        assert_eq!(doc.polylines[0].layer, "Outline");
    }

    #[test]
    fn test_roundtrip_all_entity_types() {
        let lines = vec![DxfLine::new(0.0, 0.0, 10.0, 10.0)];
        let texts = vec![DxfText::new(5.0, 5.0, "Label")];
        let circles = vec![DxfCircle::new(20.0, 20.0, 5.0)];
        let polylines = vec![DxfLwPolyline::new(vec![(30.0, 30.0), (40.0, 40.0)])];
        let mut writer = DxfWriter::new();
        let dxf_text = writer.write_all(&lines, &texts, &circles, &polylines);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.texts.len(), 1);
        assert_eq!(doc.circles.len(), 1);
        assert_eq!(doc.polylines.len(), 1);
    }

    #[test]
    fn test_roundtrip_many_lines() {
        let lines: Vec<DxfLine> = (0..100).map(|i| {
            DxfLine::new(i as f64, 0.0, (i + 1) as f64, 0.0)
        }).collect();
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &[]);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines.len(), 100);
    }

    // ================================================================
    // Error cases & malformed DXF
    // ================================================================

    #[test]
    fn test_parse_completely_empty() {
        let result = parse_dxf("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_garbage_input() {
        let result = parse_dxf("this is not a dxf file at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_header_only() {
        let dxf = "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1015\n0\nENDSEC\n0\nEOF\n";
        let result = parse_dxf(dxf);
        assert!(result.is_err()); // No ENTITIES section
    }

    #[test]
    fn test_parse_entities_with_unknown_entity_type() {
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nSPLINE\n5\n100\n0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        // Unknown entity type should be skipped
        assert!(doc.lines.is_empty());
        assert!(doc.texts.is_empty());
    }

    #[test]
    fn test_parse_line_missing_end_coords() {
        // LINE with only start coords, no end coords
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nLINE\n8\n0\n10\n5.0\n20\n10.0\n0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.lines.len(), 1);
        assert!((doc.lines[0].x1 - 5.0).abs() < 0.001);
        // End coords default to 0
        assert!((doc.lines[0].x2 - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_text_minimal() {
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nTEXT\n1\nHello\n0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.texts.len(), 1);
        assert_eq!(doc.texts[0].text, "Hello");
    }

    #[test]
    fn test_parse_handles_extra_whitespace() {
        let dxf = "  0  \n  SECTION  \n  2  \n  ENTITIES  \n  0  \n  ENDSEC  \n  0  \n  EOF  \n";
        let doc = parse_dxf(dxf).unwrap();
        assert!(doc.lines.is_empty());
    }

    #[test]
    fn test_parse_document_default() {
        let doc = DxfDocument::default();
        assert!(doc.lines.is_empty());
        assert!(doc.texts.is_empty());
        assert!(doc.circles.is_empty());
        assert!(doc.polylines.is_empty());
    }

    #[test]
    fn test_parse_document_clone() {
        let mut doc = DxfDocument::default();
        doc.lines.push(DxfLine::new(1.0, 2.0, 3.0, 4.0));
        let cloned = doc.clone();
        assert_eq!(cloned.lines.len(), 1);
    }

    #[test]
    fn test_reader_error_display() {
        let err = ReaderError::NoEntitiesSection;
        assert_eq!(format!("{}", err), "No ENTITIES section found");

        let err2 = ReaderError::MalformedGroupCode("ABC".to_string());
        assert_eq!(format!("{}", err2), "Malformed group code: ABC");
    }

    #[test]
    fn test_roundtrip_text_empty_content() {
        let texts = vec![DxfText::new(0.0, 0.0, "")];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&[], &texts);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.texts.len(), 1);
        assert_eq!(doc.texts[0].text, "");
    }

    #[test]
    fn test_roundtrip_preserves_color() {
        let lines = vec![
            DxfLine::new(0.0, 0.0, 1.0, 1.0).color(1),
            DxfLine::new(0.0, 0.0, 1.0, 1.0).color(255),
        ];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &[]);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines[0].color, 1);
        assert_eq!(doc.lines[1].color, 255);
    }

    #[test]
    fn test_roundtrip_large_coordinates() {
        let lines = vec![DxfLine::new(999999.999, -888888.888, 777777.777, -666666.666)];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&lines, &[]);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert!((doc.lines[0].x1 - 999999.999).abs() < 0.01);
        assert!((doc.lines[0].y1 - (-888888.888)).abs() < 0.01);
    }

    #[test]
    fn test_parse_mixed_with_unknown_entities() {
        let dxf = "\
0\nSECTION\n2\nENTITIES\n\
0\nLINE\n8\n0\n10\n1\n20\n2\n11\n3\n21\n4\n\
0\nSPLINE\n8\n0\n\
0\nTEXT\n8\n0\n10\n5\n20\n6\n1\nHello\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.texts.len(), 1);
        assert_eq!(doc.texts[0].text, "Hello");
    }

    // ================================================================
    // Empty / header-only / malformed entity edge cases
    // ================================================================

    #[test]
    fn test_parse_empty_string_returns_error() {
        // Completely empty input — no ENTITIES section possible
        assert!(parse_dxf("").is_err());
    }

    #[test]
    fn test_parse_newlines_only() {
        assert!(parse_dxf("\n\n\n\n").is_err());
    }

    #[test]
    fn test_parse_whitespace_only() {
        assert!(parse_dxf("   \n   \n   \n   \n").is_err());
    }

    #[test]
    fn test_parse_header_and_tables_no_entities() {
        let dxf = "\
0\nSECTION\n2\nHEADER\n\
9\n$ACADVER\n1\nAC1015\n\
9\n$INSUNITS\n70\n4\n\
0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nENDSEC\n\
0\nEOF\n";
        let result = parse_dxf(dxf);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_header_with_all_variables_no_entities() {
        let dxf = "\
0\nSECTION\n2\nHEADER\n\
9\n$ACADVER\n1\nAC1015\n\
9\n$INSUNITS\n70\n4\n\
9\n$EXTMIN\n10\n0.0\n20\n0.0\n30\n0.0\n\
9\n$EXTMAX\n10\n100.0\n20\n100.0\n30\n0.0\n\
0\nENDSEC\n\
0\nEOF\n";
        assert!(parse_dxf(dxf).is_err());
    }

    #[test]
    fn test_parse_line_with_non_numeric_coords() {
        // Group code 10 has non-numeric value — should fall back to 0.0
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLINE\n8\n0\n10\nABC\n20\n2.0\n11\n3.0\n21\n4.0\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.lines.len(), 1);
        assert!((doc.lines[0].x1 - 0.0).abs() < 0.001); // fallback
        assert!((doc.lines[0].y1 - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_text_with_non_numeric_height() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n1\nHello\n40\nBAD\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.texts[0].height, 1.0); // fallback to default
    }

    #[test]
    fn test_parse_circle_with_non_numeric_radius() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nCIRCLE\n10\n5.0\n20\n5.0\n40\nXYZ\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.circles.len(), 1);
        assert!((doc.circles[0].radius - 1.0).abs() < 0.001); // fallback
    }

    #[test]
    fn test_parse_line_entity_with_no_attributes() {
        // LINE marker immediately followed by ENDSEC — empty entity
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nLINE\n0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.lines.len(), 1);
        // All coords should be 0 (default)
        assert!((doc.lines[0].x1).abs() < 0.001);
        assert!((doc.lines[0].y1).abs() < 0.001);
        assert!((doc.lines[0].x2).abs() < 0.001);
        assert!((doc.lines[0].y2).abs() < 0.001);
        assert_eq!(doc.lines[0].color, 7); // default
        assert_eq!(doc.lines[0].layer, "0"); // default
    }

    #[test]
    fn test_parse_text_entity_with_no_attributes() {
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nTEXT\n0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.texts.len(), 1);
        assert_eq!(doc.texts[0].text, "");
        assert_eq!(doc.texts[0].height, 1.0);
    }

    #[test]
    fn test_parse_circle_entity_with_no_attributes() {
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nCIRCLE\n0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.circles.len(), 1);
        assert!((doc.circles[0].radius - 1.0).abs() < 0.001); // default
    }

    #[test]
    fn test_parse_lwpolyline_entity_with_no_vertices() {
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n0\n70\n0\n0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.polylines.len(), 1);
        assert!(doc.polylines[0].vertices.is_empty());
    }

    #[test]
    fn test_parse_entities_section_without_endsec() {
        // ENTITIES section that runs until EOF without ENDSEC
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLINE\n10\n1.0\n20\n2.0\n11\n3.0\n21\n4.0\n\
0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        // LINE should still be parsed (loop breaks on EOF entity type)
        assert_eq!(doc.lines.len(), 1);
    }

    #[test]
    fn test_parse_truncated_dxf_after_entity_marker() {
        // File ends right after entity type marker with no attributes
        let dxf = "0\nSECTION\n2\nENTITIES\n0\nLINE\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.lines.len(), 1); // empty entity with defaults
    }

    #[test]
    fn test_parse_line_with_extra_unknown_group_codes() {
        // LINE with extra group codes that should be ignored
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLINE\n\
5\nABCD\n\
330\n1F\n\
100\nAcDbEntity\n\
8\nMyLayer\n\
62\n3\n\
100\nAcDbLine\n\
10\n10.0\n20\n20.0\n30\n0.0\n\
11\n30.0\n21\n40.0\n31\n0.0\n\
999\nComment line\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.lines[0].layer, "MyLayer");
        assert_eq!(doc.lines[0].color, 3);
        assert!((doc.lines[0].x1 - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_multiple_entity_types_interleaved() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLINE\n10\n0.0\n20\n0.0\n11\n1.0\n21\n1.0\n\
0\nCIRCLE\n10\n5.0\n20\n5.0\n40\n3.0\n\
0\nTEXT\n10\n10.0\n20\n10.0\n1\nLabel\n\
0\nLWPOLYLINE\n90\n2\n70\n1\n10\n0.0\n20\n0.0\n10\n10.0\n20\n10.0\n\
0\nLINE\n10\n100.0\n20\n100.0\n11\n200.0\n21\n200.0\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.lines.len(), 2);
        assert_eq!(doc.circles.len(), 1);
        assert_eq!(doc.texts.len(), 1);
        assert_eq!(doc.polylines.len(), 1);
        assert!(doc.polylines[0].closed);
    }

    #[test]
    fn test_parse_text_with_special_characters() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nTEXT\n1\nNo.1+5 (左側)\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.texts[0].text, "No.1+5 (左側)");
    }

    #[test]
    fn test_parse_line_with_scientific_notation_coords() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLINE\n10\n1.5e3\n20\n2.5e3\n11\n3.5e3\n21\n4.5e3\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert!((doc.lines[0].x1 - 1500.0).abs() < 0.001);
        assert!((doc.lines[0].y1 - 2500.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_entities_section_appears_after_other_sections() {
        let dxf = "\
0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1015\n0\nENDSEC\n\
0\nSECTION\n2\nTABLES\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nLINE\n10\n1.0\n20\n2.0\n11\n3.0\n21\n4.0\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.lines.len(), 1);
    }

    // ================================================================
    // LWPOLYLINE parsing edge cases
    // ================================================================

    #[test]
    fn test_parse_lwpolyline_open_with_vertices() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLWPOLYLINE\n90\n4\n70\n0\n43\n0.0\n\
10\n0.0\n20\n0.0\n\
10\n100.0\n20\n0.0\n\
10\n100.0\n20\n100.0\n\
10\n0.0\n20\n100.0\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.polylines.len(), 1);
        assert_eq!(doc.polylines[0].vertices.len(), 4);
        assert!(!doc.polylines[0].closed);
        assert!((doc.polylines[0].vertices[0].0 - 0.0).abs() < 0.001);
        assert!((doc.polylines[0].vertices[1].0 - 100.0).abs() < 0.001);
        assert!((doc.polylines[0].vertices[2].1 - 100.0).abs() < 0.001);
        assert!((doc.polylines[0].vertices[3].0 - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_lwpolyline_closed_flag() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLWPOLYLINE\n90\n3\n70\n1\n\
10\n0.0\n20\n0.0\n\
10\n10.0\n20\n0.0\n\
10\n5.0\n20\n8.66\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert!(doc.polylines[0].closed);
        assert_eq!(doc.polylines[0].vertices.len(), 3);
    }

    #[test]
    fn test_parse_lwpolyline_with_layer_and_color() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLWPOLYLINE\n8\n外枠\n62\n3\n90\n2\n70\n0\n\
10\n0.0\n20\n0.0\n\
10\n50.0\n20\n50.0\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.polylines[0].layer, "外枠");
        assert_eq!(doc.polylines[0].color, 3);
        assert_eq!(doc.polylines[0].vertices.len(), 2);
    }

    #[test]
    fn test_parse_lwpolyline_single_vertex() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLWPOLYLINE\n90\n1\n70\n0\n\
10\n42.0\n20\n99.0\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.polylines[0].vertices.len(), 1);
        assert!((doc.polylines[0].vertices[0].0 - 42.0).abs() < 0.001);
        assert!((doc.polylines[0].vertices[0].1 - 99.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_lwpolyline_negative_coords() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLWPOLYLINE\n90\n2\n70\n0\n\
10\n-500.0\n20\n-1000.0\n\
10\n500.0\n20\n1000.0\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert!((doc.polylines[0].vertices[0].0 - (-500.0)).abs() < 0.001);
        assert!((doc.polylines[0].vertices[1].1 - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_multiple_lwpolylines() {
        let dxf = "0\nSECTION\n2\nENTITIES\n\
0\nLWPOLYLINE\n90\n2\n70\n0\n10\n0.0\n20\n0.0\n10\n10.0\n20\n10.0\n\
0\nLWPOLYLINE\n90\n3\n70\n1\n10\n20.0\n20\n20.0\n10\n30.0\n20\n20.0\n10\n25.0\n20\n30.0\n\
0\nENDSEC\n0\nEOF\n";
        let doc = parse_dxf(dxf).unwrap();
        assert_eq!(doc.polylines.len(), 2);
        assert!(!doc.polylines[0].closed);
        assert!(doc.polylines[1].closed);
        assert_eq!(doc.polylines[0].vertices.len(), 2);
        assert_eq!(doc.polylines[1].vertices.len(), 3);
    }

    // ================================================================
    // Full writer→reader→verify roundtrips with mixed entities
    // ================================================================

    #[test]
    fn test_roundtrip_verify_all_line_coords() {
        let original = vec![
            DxfLine::new(0.0, 0.0, 100.0, 200.0),
            DxfLine::new(-50.0, -75.0, 999.9, 0.001),
            DxfLine::new(1e6, -1e6, 0.0, 0.0),
        ];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&original, &[]);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines.len(), original.len());
        for (parsed, orig) in doc.lines.iter().zip(original.iter()) {
            assert!((parsed.x1 - orig.x1).abs() < 0.01, "x1 mismatch");
            assert!((parsed.y1 - orig.y1).abs() < 0.01, "y1 mismatch");
            assert!((parsed.x2 - orig.x2).abs() < 0.01, "x2 mismatch");
            assert!((parsed.y2 - orig.y2).abs() < 0.01, "y2 mismatch");
            assert_eq!(parsed.color, orig.color);
            assert_eq!(parsed.layer, orig.layer);
        }
    }

    #[test]
    fn test_roundtrip_verify_all_text_fields() {
        let original = vec![
            DxfText::new(0.0, 0.0, "Origin"),
            DxfText::new(100.0, 200.0, "No.1+5").height(250.0).rotation(-90.0).color(5).layer("測点"),
            DxfText::new(-50.0, -50.0, "横断歩道").height(150.0).color(3),
        ];
        let writer = DxfWriter::new();
        let dxf_text = writer.write(&[], &original);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.texts.len(), original.len());
        for (parsed, orig) in doc.texts.iter().zip(original.iter()) {
            assert!((parsed.x - orig.x).abs() < 0.01, "x mismatch for '{}'", orig.text);
            assert!((parsed.y - orig.y).abs() < 0.01, "y mismatch for '{}'", orig.text);
            assert_eq!(parsed.text, orig.text);
            assert!((parsed.height - orig.height).abs() < 0.01, "height mismatch for '{}'", orig.text);
            assert!((parsed.rotation - orig.rotation).abs() < 0.01, "rotation mismatch for '{}'", orig.text);
            assert_eq!(parsed.color, orig.color, "color mismatch for '{}'", orig.text);
            assert_eq!(parsed.layer, orig.layer, "layer mismatch for '{}'", orig.text);
        }
    }

    #[test]
    fn test_roundtrip_verify_circles() {
        let original = vec![
            DxfCircle::new(0.0, 0.0, 1.0),
            DxfCircle::new(500.0, -300.0, 99.9).color(3).layer("丸"),
            DxfCircle::new(-1000.0, 2000.0, 0.5),
        ];
        let mut writer = DxfWriter::new();
        let dxf_text = writer.write_all(&[], &[], &original, &[]);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.circles.len(), original.len());
        for (parsed, orig) in doc.circles.iter().zip(original.iter()) {
            assert!((parsed.x - orig.x).abs() < 0.01, "x mismatch");
            assert!((parsed.y - orig.y).abs() < 0.01, "y mismatch");
            assert!((parsed.radius - orig.radius).abs() < 0.01, "radius mismatch");
            assert_eq!(parsed.color, orig.color);
            assert_eq!(parsed.layer, orig.layer);
        }
    }

    #[test]
    fn test_roundtrip_verify_lwpolylines() {
        let original = vec![
            DxfLwPolyline::new(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]),
            DxfLwPolyline::closed(vec![(100.0, 100.0), (200.0, 100.0), (200.0, 200.0), (100.0, 200.0)])
                .color(5).layer("外枠"),
        ];
        let mut writer = DxfWriter::new();
        let dxf_text = writer.write_all(&[], &[], &[], &original);
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.polylines.len(), original.len());
        for (parsed, orig) in doc.polylines.iter().zip(original.iter()) {
            assert_eq!(parsed.vertices.len(), orig.vertices.len(), "vertex count mismatch");
            assert_eq!(parsed.closed, orig.closed, "closed mismatch");
            assert_eq!(parsed.color, orig.color);
            assert_eq!(parsed.layer, orig.layer);
            for (pv, ov) in parsed.vertices.iter().zip(orig.vertices.iter()) {
                assert!((pv.0 - ov.0).abs() < 0.01, "vertex x mismatch");
                assert!((pv.1 - ov.1).abs() < 0.01, "vertex y mismatch");
            }
        }
    }

    #[test]
    fn test_roundtrip_mixed_all_types_verify_counts_and_lint() {
        use crate::dxf::linter::DxfLinter;

        let lines = vec![
            DxfLine::with_style(0.0, 0.0, 1000.0, 0.0, 7, "中心線"),
            DxfLine::with_style(0.0, -500.0, 0.0, 500.0, 3, "横断歩道"),
        ];
        let texts = vec![
            DxfText::new(0.0, 50.0, "No.0").layer("測点").color(5).height(250.0).rotation(-90.0),
            DxfText::new(1000.0, 50.0, "No.1").layer("測点").color(5),
        ];
        let circles = vec![
            DxfCircle::new(500.0, 0.0, 25.0).layer("マーカー"),
        ];
        let polylines = vec![
            DxfLwPolyline::closed(vec![
                (0.0, -500.0), (1000.0, -500.0), (1000.0, 500.0), (0.0, 500.0),
            ]).layer("外枠").color(1),
        ];

        let mut writer = DxfWriter::new();
        let dxf_text = writer.write_all(&lines, &texts, &circles, &polylines);

        // Lint validation
        assert!(DxfLinter::is_valid(&dxf_text), "Writer output failed lint");

        // Parse back
        let doc = parse_dxf(&dxf_text).unwrap();
        assert_eq!(doc.lines.len(), 2);
        assert_eq!(doc.texts.len(), 2);
        assert_eq!(doc.circles.len(), 1);
        assert_eq!(doc.polylines.len(), 1);

        // Verify specific values survived roundtrip
        assert_eq!(doc.lines[0].layer, "中心線");
        assert_eq!(doc.lines[1].color, 3);
        assert_eq!(doc.texts[0].text, "No.0");
        assert!((doc.texts[0].rotation - (-90.0)).abs() < 0.01);
        assert!((doc.circles[0].radius - 25.0).abs() < 0.01);
        assert!(doc.polylines[0].closed);
        assert_eq!(doc.polylines[0].vertices.len(), 4);
        assert_eq!(doc.polylines[0].layer, "外枠");
    }

    #[test]
    fn test_roundtrip_double_parse_idempotent() {
        // Write → parse → write again → parse again, verify identical
        let lines = vec![DxfLine::with_style(10.0, 20.0, 30.0, 40.0, 5, "Layer1")];
        let texts = vec![DxfText::new(50.0, 60.0, "Hello").height(10.0)];

        let writer1 = DxfWriter::new();
        let dxf1 = writer1.write(&lines, &texts);
        let doc1 = parse_dxf(&dxf1).unwrap();

        // Re-write from parsed entities
        let writer2 = DxfWriter::new();
        let dxf2 = writer2.write(&doc1.lines, &doc1.texts);
        let doc2 = parse_dxf(&dxf2).unwrap();

        assert_eq!(doc1.lines.len(), doc2.lines.len());
        assert_eq!(doc1.texts.len(), doc2.texts.len());
        assert!((doc2.lines[0].x1 - 10.0).abs() < 0.01);
        assert_eq!(doc2.texts[0].text, "Hello");
    }
}
