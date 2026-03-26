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
    todo!("Implement: parse DXF text into entities")
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
