//! DXF file writer
//!
//! This module provides functionality to generate DXF text output
//! from DxfLine, DxfText, DxfCircle, and DxfLwPolyline entities.
//!
//! All entities are written with proper handles for AutoCAD compatibility.

use std::fmt::Write;

use crate::dxf::entities::{DxfCircle, DxfLine, DxfLwPolyline, DxfText};
use crate::dxf::handle::{HandleGenerator, owners};

/// DXF file writer
///
/// Generates DXF format text from line, text, circle, and polyline entities.
/// Each entity is assigned a unique handle for CAD software compatibility.
pub struct DxfWriter {
    handle_gen: HandleGenerator,
}

impl DxfWriter {
    /// Creates a new DxfWriter instance
    pub fn new() -> Self {
        Self {
            handle_gen: HandleGenerator::new(),
        }
    }

    /// Generates DXF text from lines and texts (legacy API, no handles)
    ///
    /// This method is kept for backward compatibility.
    /// For proper DXF with handles, use `write_all` instead.
    pub fn write(&self, lines: &[DxfLine], texts: &[DxfText]) -> String {
        let mut output = String::new();
        let mut handle_gen = HandleGenerator::new();

        // Header section
        Self::write_header(&mut output);

        // Entities section
        Self::write_entities_start(&mut output);

        // Write lines
        for line in lines {
            Self::write_line_with_handle(&mut output, line, &mut handle_gen);
        }

        // Write texts
        for text in texts {
            Self::write_text_with_handle(&mut output, text, &mut handle_gen);
        }

        // End entities and file
        Self::write_end(&mut output);

        output
    }

    /// Generates DXF text from all entity types with proper handles
    pub fn write_all(
        &mut self,
        lines: &[DxfLine],
        texts: &[DxfText],
        circles: &[DxfCircle],
        polylines: &[DxfLwPolyline],
    ) -> String {
        let mut output = String::new();

        // Header section
        Self::write_header(&mut output);

        // Entities section
        Self::write_entities_start(&mut output);

        // Write all entities with handles
        for line in lines {
            Self::write_line_with_handle(&mut output, line, &mut self.handle_gen);
        }

        for text in texts {
            Self::write_text_with_handle(&mut output, text, &mut self.handle_gen);
        }

        for circle in circles {
            Self::write_circle_with_handle(&mut output, circle, &mut self.handle_gen);
        }

        for polyline in polylines {
            Self::write_lwpolyline_with_handle(&mut output, polyline, &mut self.handle_gen);
        }

        // End entities and file
        Self::write_end(&mut output);

        output
    }

    /// Resets the handle generator for a new document
    pub fn reset(&mut self) {
        self.handle_gen = HandleGenerator::new();
    }

    /// Writes the DXF header section
    fn write_header(output: &mut String) {
        writeln!(output, "0\nSECTION").unwrap();
        writeln!(output, "2\nHEADER").unwrap();
        writeln!(output, "9\n$ACADVER").unwrap();
        writeln!(output, "1\nAC1015").unwrap();
        writeln!(output, "9\n$INSUNITS").unwrap();
        writeln!(output, "70\n4").unwrap(); // 4 = Millimeters
        writeln!(output, "0\nENDSEC").unwrap();
    }

    /// Writes the start of the entities section
    fn write_entities_start(output: &mut String) {
        writeln!(output, "0\nSECTION").unwrap();
        writeln!(output, "2\nENTITIES").unwrap();
    }

    /// Writes common entity header (handle, owner, subclass marker)
    fn write_entity_header(output: &mut String, entity_type: &str, layer: &str, color: i32, handle: &str) {
        writeln!(output, "0\n{}", entity_type).unwrap();
        writeln!(output, "5\n{}", handle).unwrap();
        writeln!(output, "330\n{}", owners::MODEL_SPACE).unwrap();
        writeln!(output, "100\nAcDbEntity").unwrap();
        writeln!(output, "8\n{}", layer).unwrap();
        writeln!(output, "62\n{}", color).unwrap();
    }

    /// Writes a LINE entity with handle
    fn write_line_with_handle(output: &mut String, line: &DxfLine, handle_gen: &mut HandleGenerator) {
        let handle = handle_gen.next();
        Self::write_entity_header(output, "LINE", &line.layer, line.color, &handle);
        writeln!(output, "100\nAcDbLine").unwrap();
        writeln!(output, "10\n{}", line.x1).unwrap();
        writeln!(output, "20\n{}", line.y1).unwrap();
        writeln!(output, "30\n0").unwrap();
        writeln!(output, "11\n{}", line.x2).unwrap();
        writeln!(output, "21\n{}", line.y2).unwrap();
        writeln!(output, "31\n0").unwrap();
    }

    /// Writes a TEXT entity with handle
    fn write_text_with_handle(output: &mut String, text: &DxfText, handle_gen: &mut HandleGenerator) {
        use crate::dxf::entities::{HorizontalAlignment, VerticalAlignment};

        let handle = handle_gen.next();
        Self::write_entity_header(output, "TEXT", &text.layer, text.color, &handle);
        writeln!(output, "100\nAcDbText").unwrap();
        writeln!(output, "10\n{}", text.x).unwrap();
        writeln!(output, "20\n{}", text.y).unwrap();
        writeln!(output, "30\n0").unwrap();
        writeln!(output, "40\n{}", text.height).unwrap();
        writeln!(output, "1\n{}", text.text).unwrap();
        writeln!(output, "50\n{}", text.rotation).unwrap();

        // For non-default alignment, specify second alignment point (group 11/21)
        let needs_second_point = text.align_h != HorizontalAlignment::Left
            || text.align_v != VerticalAlignment::Baseline;

        if needs_second_point {
            writeln!(output, "11\n{}", text.x).unwrap();
            writeln!(output, "21\n{}", text.y).unwrap();
            writeln!(output, "31\n0").unwrap();
        }

        writeln!(output, "72\n{}", text.align_h as i32).unwrap();
        writeln!(output, "100\nAcDbText").unwrap();
        writeln!(output, "73\n{}", text.align_v as i32).unwrap();
    }

    /// Writes a CIRCLE entity with handle
    fn write_circle_with_handle(output: &mut String, circle: &DxfCircle, handle_gen: &mut HandleGenerator) {
        let handle = handle_gen.next();
        Self::write_entity_header(output, "CIRCLE", &circle.layer, circle.color, &handle);
        writeln!(output, "100\nAcDbCircle").unwrap();
        writeln!(output, "10\n{}", circle.x).unwrap();
        writeln!(output, "20\n{}", circle.y).unwrap();
        writeln!(output, "30\n0").unwrap();
        writeln!(output, "40\n{}", circle.radius).unwrap();
    }

    /// Writes a LWPOLYLINE entity with handle
    fn write_lwpolyline_with_handle(output: &mut String, polyline: &DxfLwPolyline, handle_gen: &mut HandleGenerator) {
        if polyline.vertices.is_empty() {
            return;
        }

        let handle = handle_gen.next();
        Self::write_entity_header(output, "LWPOLYLINE", &polyline.layer, polyline.color, &handle);
        writeln!(output, "100\nAcDbPolyline").unwrap();
        writeln!(output, "90\n{}", polyline.vertices.len()).unwrap();
        writeln!(output, "70\n{}", if polyline.closed { 1 } else { 0 }).unwrap();
        writeln!(output, "43\n0").unwrap(); // Constant width

        for (x, y) in &polyline.vertices {
            writeln!(output, "10\n{}", x).unwrap();
            writeln!(output, "20\n{}", y).unwrap();
        }
    }

    /// Writes the end of the entities section and EOF
    fn write_end(output: &mut String) {
        writeln!(output, "0\nENDSEC").unwrap();
        writeln!(output, "0\nEOF").unwrap();
    }
}

impl Default for DxfWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dxf::entities::{HorizontalAlignment, VerticalAlignment};

    #[test]
    fn test_write_empty() {
        let writer = DxfWriter::new();
        let output = writer.write(&[], &[]);

        assert!(output.contains("SECTION"));
        assert!(output.contains("HEADER"));
        assert!(output.contains("$ACADVER"));
        assert!(output.contains("AC1015"));
        assert!(output.contains("$INSUNITS"));
        assert!(output.contains("ENTITIES"));
        assert!(output.contains("ENDSEC"));
        assert!(output.contains("EOF"));
    }

    #[test]
    fn test_write_single_line() {
        let writer = DxfWriter::new();
        let lines = vec![DxfLine::new(0.0, 0.0, 100.0, 100.0)];
        let output = writer.write(&lines, &[]);

        assert!(output.contains("LINE"));
        assert!(output.contains("5\n100")); // Handle
        assert!(output.contains("330\n1F")); // Owner
        assert!(output.contains("100\nAcDbEntity"));
        assert!(output.contains("100\nAcDbLine"));
    }

    #[test]
    fn test_write_line_with_style() {
        let writer = DxfWriter::new();
        let lines = vec![DxfLine::with_style(1.0, 2.0, 3.0, 4.0, 5, "TestLayer")];
        let output = writer.write(&lines, &[]);

        assert!(output.contains("LINE"));
        assert!(output.contains("8\nTestLayer"));
        assert!(output.contains("62\n5"));
    }

    #[test]
    fn test_write_single_text() {
        let writer = DxfWriter::new();
        let texts = vec![DxfText::new(50.0, 50.0, "Hello World")];
        let output = writer.write(&[], &texts);

        assert!(output.contains("TEXT"));
        assert!(output.contains("5\n100")); // Handle
        assert!(output.contains("100\nAcDbText"));
        assert!(output.contains("1\nHello World"));
    }

    #[test]
    fn test_write_text_with_builder() {
        let writer = DxfWriter::new();
        let texts = vec![DxfText::new(10.0, 20.0, "Styled")
            .height(2.5)
            .rotation(45.0)
            .color(3)
            .align_h(HorizontalAlignment::Center)
            .align_v(VerticalAlignment::Middle)
            .layer("TextLayer")];
        let output = writer.write(&[], &texts);

        assert!(output.contains("TEXT"));
        assert!(output.contains("8\nTextLayer"));
        assert!(output.contains("62\n3"));
        assert!(output.contains("40\n2.5"));
        assert!(output.contains("50\n45"));
        assert!(output.contains("72\n1"));
        assert!(output.contains("73\n2"));
    }

    #[test]
    fn test_write_text_default_alignment_no_second_point() {
        let writer = DxfWriter::new();
        let texts = vec![DxfText::new(30.0, 40.0, "Default")];
        let output = writer.write(&[], &texts);

        assert!(output.contains("TEXT"));
        // Should NOT contain second alignment point for default alignment
        let text_section_start = output.find("1\nDefault").unwrap();
        let after_text = &output[text_section_start..];
        assert!(!after_text.contains("11\n30"));
    }

    #[test]
    fn test_write_text_vertical_alignment_has_second_point() {
        let writer = DxfWriter::new();
        let texts = vec![DxfText::new(50.0, 60.0, "Vertical")
            .align_h(HorizontalAlignment::Left)
            .align_v(VerticalAlignment::Middle)];
        let output = writer.write(&[], &texts);

        assert!(output.contains("11\n50"));
        assert!(output.contains("21\n60"));
    }

    #[test]
    fn test_write_multiple_entities() {
        let writer = DxfWriter::new();
        let lines = vec![
            DxfLine::new(0.0, 0.0, 10.0, 10.0),
            DxfLine::new(10.0, 10.0, 20.0, 20.0),
        ];
        let texts = vec![
            DxfText::new(5.0, 5.0, "Point 1"),
            DxfText::new(15.0, 15.0, "Point 2"),
        ];
        let output = writer.write(&lines, &texts);

        let line_count = output.matches("0\nLINE\n").count();
        assert_eq!(line_count, 2);

        let text_count = output.matches("0\nTEXT\n").count();
        assert_eq!(text_count, 2);
    }

    #[test]
    fn test_header_content() {
        let writer = DxfWriter::new();
        let output = writer.write(&[], &[]);

        assert!(output.starts_with("0\nSECTION\n2\nHEADER\n"));
        assert!(output.contains("9\n$ACADVER\n1\nAC1015\n"));
        assert!(output.contains("9\n$INSUNITS\n70\n4\n"));
    }

    #[test]
    fn test_default_implementation() {
        let writer = DxfWriter::default();
        let output = writer.write(&[], &[]);
        assert!(output.contains("EOF"));
    }

    #[test]
    fn test_write_single_circle() {
        let mut writer = DxfWriter::new();
        let circles = vec![DxfCircle::new(50.0, 50.0, 10.0)];
        let output = writer.write_all(&[], &[], &circles, &[]);

        assert!(output.contains("CIRCLE"));
        assert!(output.contains("5\n100")); // Handle
        assert!(output.contains("100\nAcDbCircle"));
        assert!(output.contains("40\n10")); // Radius
    }

    #[test]
    fn test_write_circle_with_style() {
        let mut writer = DxfWriter::new();
        let circles = vec![DxfCircle::new(100.0, 200.0, 25.0)
            .color(3)
            .layer("CircleLayer")];
        let output = writer.write_all(&[], &[], &circles, &[]);

        assert!(output.contains("CIRCLE"));
        assert!(output.contains("8\nCircleLayer"));
        assert!(output.contains("62\n3"));
    }

    #[test]
    fn test_write_lwpolyline_open() {
        let mut writer = DxfWriter::new();
        let polylines = vec![DxfLwPolyline::new(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
        ])];
        let output = writer.write_all(&[], &[], &[], &polylines);

        assert!(output.contains("LWPOLYLINE"));
        assert!(output.contains("100\nAcDbPolyline"));
        assert!(output.contains("90\n3")); // 3 vertices
        assert!(output.contains("70\n0")); // Open
    }

    #[test]
    fn test_write_lwpolyline_closed() {
        let mut writer = DxfWriter::new();
        let polylines = vec![DxfLwPolyline::closed(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
        ])];
        let output = writer.write_all(&[], &[], &[], &polylines);

        assert!(output.contains("LWPOLYLINE"));
        assert!(output.contains("90\n4")); // 4 vertices
        assert!(output.contains("70\n1")); // Closed
    }

    #[test]
    fn test_write_lwpolyline_with_style() {
        let mut writer = DxfWriter::new();
        let polylines = vec![DxfLwPolyline::new(vec![(0.0, 0.0), (100.0, 100.0)])
            .color(5)
            .layer("OutlineLayer")];
        let output = writer.write_all(&[], &[], &[], &polylines);

        assert!(output.contains("8\nOutlineLayer"));
        assert!(output.contains("62\n5"));
    }

    #[test]
    fn test_write_lwpolyline_empty_skipped() {
        let mut writer = DxfWriter::new();
        let polylines = vec![DxfLwPolyline::new(vec![])];
        let output = writer.write_all(&[], &[], &[], &polylines);

        assert!(!output.contains("LWPOLYLINE"));
    }

    #[test]
    fn test_write_all_mixed_entities() {
        let mut writer = DxfWriter::new();
        let lines = vec![DxfLine::new(0.0, 0.0, 10.0, 10.0)];
        let texts = vec![DxfText::new(5.0, 5.0, "Label")];
        let circles = vec![DxfCircle::new(20.0, 20.0, 5.0)];
        let polylines = vec![DxfLwPolyline::closed(vec![
            (30.0, 30.0),
            (40.0, 30.0),
            (40.0, 40.0),
        ])];

        let output = writer.write_all(&lines, &texts, &circles, &polylines);

        assert_eq!(output.matches("0\nLINE\n").count(), 1);
        assert_eq!(output.matches("0\nTEXT\n").count(), 1);
        assert_eq!(output.matches("0\nCIRCLE\n").count(), 1);
        assert_eq!(output.matches("0\nLWPOLYLINE\n").count(), 1);
    }

    #[test]
    fn test_handles_are_unique() {
        let mut writer = DxfWriter::new();
        let lines = vec![
            DxfLine::new(0.0, 0.0, 10.0, 10.0),
            DxfLine::new(10.0, 10.0, 20.0, 20.0),
            DxfLine::new(20.0, 20.0, 30.0, 30.0),
        ];
        let output = writer.write_all(&lines, &[], &[], &[]);

        // Check handles are sequential and unique
        assert!(output.contains("5\n100"));
        assert!(output.contains("5\n101"));
        assert!(output.contains("5\n102"));
    }

    #[test]
    fn test_all_entities_have_owner_reference() {
        let mut writer = DxfWriter::new();
        let lines = vec![DxfLine::new(0.0, 0.0, 10.0, 10.0)];
        let texts = vec![DxfText::new(5.0, 5.0, "Test")];
        let circles = vec![DxfCircle::new(20.0, 20.0, 5.0)];
        let polylines = vec![DxfLwPolyline::new(vec![(0.0, 0.0), (10.0, 10.0)])];

        let output = writer.write_all(&lines, &texts, &circles, &polylines);

        // All entities should have owner reference (330)
        let owner_count = output.matches("330\n1F").count();
        assert_eq!(owner_count, 4);
    }

    #[test]
    fn test_reset_handles() {
        let mut writer = DxfWriter::new();
        let lines = vec![DxfLine::new(0.0, 0.0, 10.0, 10.0)];

        let output1 = writer.write_all(&lines, &[], &[], &[]);
        assert!(output1.contains("5\n100"));

        // Without reset, next handle continues
        let output2 = writer.write_all(&lines, &[], &[], &[]);
        assert!(output2.contains("5\n101"));

        // After reset, handles start over
        writer.reset();
        let output3 = writer.write_all(&lines, &[], &[], &[]);
        assert!(output3.contains("5\n100"));
    }
}
