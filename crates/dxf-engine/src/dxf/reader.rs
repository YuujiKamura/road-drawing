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
}
