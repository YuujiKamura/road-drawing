//! DXF entity data structures
//!
//! This module defines the core data structures for DXF entities:
//! - `DxfLine`: LINE entity with coordinates, color, and layer
//! - `DxfText`: TEXT entity with position, content, styling, and alignment

/// Horizontal text alignment
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(i32)]
pub enum HorizontalAlignment {
    #[default]
    Left = 0,
    Center = 1,
    Right = 2,
}

/// Vertical text alignment
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(i32)]
pub enum VerticalAlignment {
    #[default]
    Baseline = 0,
    Bottom = 1,
    Middle = 2,
    Top = 3,
}

/// DXF LINE entity
///
/// Represents a line segment with start and end coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct DxfLine {
    /// X coordinate of the start point
    pub x1: f64,
    /// Y coordinate of the start point
    pub y1: f64,
    /// X coordinate of the end point
    pub x2: f64,
    /// Y coordinate of the end point
    pub y2: f64,
    /// Color number (default: 7 = white)
    pub color: i32,
    /// Layer name (default: "0")
    pub layer: String,
}

impl Default for DxfLine {
    fn default() -> Self {
        Self {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            color: 7,
            layer: String::from("0"),
        }
    }
}

impl DxfLine {
    /// Creates a new DxfLine with the specified coordinates
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            ..Default::default()
        }
    }

    /// Creates a new DxfLine with all parameters
    pub fn with_style(x1: f64, y1: f64, x2: f64, y2: f64, color: i32, layer: &str) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            color,
            layer: layer.to_string(),
        }
    }

    /// Builder: set color
    pub fn color(mut self, color: i32) -> Self {
        self.color = color;
        self
    }

    /// Builder: set layer
    pub fn layer(mut self, layer: &str) -> Self {
        self.layer = layer.to_string();
        self
    }
}

/// DXF TEXT entity
///
/// Represents a text element with position, content, and styling.
#[derive(Clone, Debug, PartialEq)]
pub struct DxfText {
    /// X coordinate of the text position
    pub x: f64,
    /// Y coordinate of the text position
    pub y: f64,
    /// Text content
    pub text: String,
    /// Text height (default: 1.0)
    pub height: f64,
    /// Rotation angle in degrees (default: 0.0)
    pub rotation: f64,
    /// Color number (default: 7 = white)
    pub color: i32,
    /// Horizontal alignment
    pub align_h: HorizontalAlignment,
    /// Vertical alignment
    pub align_v: VerticalAlignment,
    /// Layer name (default: "0")
    pub layer: String,
}

impl Default for DxfText {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            text: String::new(),
            height: 1.0,
            rotation: 0.0,
            color: 7,
            align_h: HorizontalAlignment::Left,
            align_v: VerticalAlignment::Baseline,
            layer: String::from("0"),
        }
    }
}

impl DxfText {
    /// Creates a new DxfText with position and content
    pub fn new(x: f64, y: f64, text: &str) -> Self {
        Self {
            x,
            y,
            text: text.to_string(),
            ..Default::default()
        }
    }

    /// Builder: set height
    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// Builder: set rotation
    pub fn rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// Builder: set color
    pub fn color(mut self, color: i32) -> Self {
        self.color = color;
        self
    }

    /// Builder: set horizontal alignment
    pub fn align_h(mut self, align_h: HorizontalAlignment) -> Self {
        self.align_h = align_h;
        self
    }

    /// Builder: set vertical alignment
    pub fn align_v(mut self, align_v: VerticalAlignment) -> Self {
        self.align_v = align_v;
        self
    }

    /// Builder: set layer
    pub fn layer(mut self, layer: &str) -> Self {
        self.layer = layer.to_string();
        self
    }
}

/// DXF CIRCLE entity
///
/// Represents a circle with center point and radius.
#[derive(Clone, Debug, PartialEq)]
pub struct DxfCircle {
    /// X coordinate of the center
    pub x: f64,
    /// Y coordinate of the center
    pub y: f64,
    /// Radius
    pub radius: f64,
    /// Color number (default: 7 = white)
    pub color: i32,
    /// Layer name (default: "0")
    pub layer: String,
}

impl Default for DxfCircle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
            color: 7,
            layer: String::from("0"),
        }
    }
}

impl DxfCircle {
    /// Creates a new DxfCircle with center and radius
    pub fn new(x: f64, y: f64, radius: f64) -> Self {
        Self {
            x,
            y,
            radius,
            ..Default::default()
        }
    }

    /// Builder: set color
    pub fn color(mut self, color: i32) -> Self {
        self.color = color;
        self
    }

    /// Builder: set layer
    pub fn layer(mut self, layer: &str) -> Self {
        self.layer = layer.to_string();
        self
    }
}

/// DXF LWPOLYLINE entity
///
/// Represents a lightweight polyline (2D polyline with vertices).
#[derive(Clone, Debug, PartialEq)]
pub struct DxfLwPolyline {
    /// List of vertices as (x, y) pairs
    pub vertices: Vec<(f64, f64)>,
    /// Whether the polyline is closed
    pub closed: bool,
    /// Color number (default: 7 = white)
    pub color: i32,
    /// Layer name (default: "0")
    pub layer: String,
}

impl Default for DxfLwPolyline {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            closed: false,
            color: 7,
            layer: String::from("0"),
        }
    }
}

impl DxfLwPolyline {
    /// Creates a new DxfLwPolyline with vertices
    pub fn new(vertices: Vec<(f64, f64)>) -> Self {
        Self {
            vertices,
            ..Default::default()
        }
    }

    /// Creates a new closed DxfLwPolyline with vertices
    pub fn closed(vertices: Vec<(f64, f64)>) -> Self {
        Self {
            vertices,
            closed: true,
            ..Default::default()
        }
    }

    /// Builder: set closed flag
    pub fn set_closed(mut self, closed: bool) -> Self {
        self.closed = closed;
        self
    }

    /// Builder: set color
    pub fn color(mut self, color: i32) -> Self {
        self.color = color;
        self
    }

    /// Builder: set layer
    pub fn layer(mut self, layer: &str) -> Self {
        self.layer = layer.to_string();
        self
    }

    /// Add a vertex
    pub fn add_vertex(mut self, x: f64, y: f64) -> Self {
        self.vertices.push((x, y));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dxf_line_default() {
        let line = DxfLine::default();
        assert_eq!(line.x1, 0.0);
        assert_eq!(line.y1, 0.0);
        assert_eq!(line.x2, 0.0);
        assert_eq!(line.y2, 0.0);
        assert_eq!(line.color, 7);
        assert_eq!(line.layer, "0");
    }

    #[test]
    fn test_dxf_line_new() {
        let line = DxfLine::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(line.x1, 1.0);
        assert_eq!(line.y1, 2.0);
        assert_eq!(line.x2, 3.0);
        assert_eq!(line.y2, 4.0);
        assert_eq!(line.color, 7);
        assert_eq!(line.layer, "0");
    }

    #[test]
    fn test_dxf_line_with_style() {
        let line = DxfLine::with_style(1.0, 2.0, 3.0, 4.0, 5, "Layer1");
        assert_eq!(line.x1, 1.0);
        assert_eq!(line.y1, 2.0);
        assert_eq!(line.x2, 3.0);
        assert_eq!(line.y2, 4.0);
        assert_eq!(line.color, 5);
        assert_eq!(line.layer, "Layer1");
    }

    #[test]
    fn test_dxf_line_builder() {
        let line = DxfLine::new(1.0, 2.0, 3.0, 4.0)
            .color(5)
            .layer("Layer1");
        assert_eq!(line.color, 5);
        assert_eq!(line.layer, "Layer1");
    }

    #[test]
    fn test_dxf_text_default() {
        let text = DxfText::default();
        assert_eq!(text.x, 0.0);
        assert_eq!(text.y, 0.0);
        assert_eq!(text.text, "");
        assert_eq!(text.height, 1.0);
        assert_eq!(text.rotation, 0.0);
        assert_eq!(text.color, 7);
        assert_eq!(text.align_h, HorizontalAlignment::Left);
        assert_eq!(text.align_v, VerticalAlignment::Baseline);
        assert_eq!(text.layer, "0");
    }

    #[test]
    fn test_dxf_text_new() {
        let text = DxfText::new(10.0, 20.0, "Hello");
        assert_eq!(text.x, 10.0);
        assert_eq!(text.y, 20.0);
        assert_eq!(text.text, "Hello");
        assert_eq!(text.height, 1.0);
    }

    #[test]
    fn test_dxf_text_builder() {
        let text = DxfText::new(10.0, 20.0, "Hello")
            .height(2.5)
            .rotation(45.0)
            .color(3)
            .align_h(HorizontalAlignment::Center)
            .align_v(VerticalAlignment::Middle)
            .layer("TextLayer");
        assert_eq!(text.height, 2.5);
        assert_eq!(text.rotation, 45.0);
        assert_eq!(text.color, 3);
        assert_eq!(text.align_h, HorizontalAlignment::Center);
        assert_eq!(text.align_v, VerticalAlignment::Middle);
        assert_eq!(text.layer, "TextLayer");
    }

    #[test]
    fn test_alignment_values() {
        assert_eq!(HorizontalAlignment::Left as i32, 0);
        assert_eq!(HorizontalAlignment::Center as i32, 1);
        assert_eq!(HorizontalAlignment::Right as i32, 2);
        assert_eq!(VerticalAlignment::Baseline as i32, 0);
        assert_eq!(VerticalAlignment::Bottom as i32, 1);
        assert_eq!(VerticalAlignment::Middle as i32, 2);
        assert_eq!(VerticalAlignment::Top as i32, 3);
    }

    #[test]
    fn test_dxf_line_clone() {
        let line = DxfLine::new(1.0, 2.0, 3.0, 4.0);
        let cloned = line.clone();
        assert_eq!(line, cloned);
    }

    #[test]
    fn test_dxf_text_clone() {
        let text = DxfText::new(10.0, 20.0, "Test");
        let cloned = text.clone();
        assert_eq!(text, cloned);
    }

    #[test]
    fn test_dxf_circle_default() {
        let circle = DxfCircle::default();
        assert_eq!(circle.x, 0.0);
        assert_eq!(circle.y, 0.0);
        assert_eq!(circle.radius, 1.0);
        assert_eq!(circle.color, 7);
        assert_eq!(circle.layer, "0");
    }

    #[test]
    fn test_dxf_circle_new() {
        let circle = DxfCircle::new(10.0, 20.0, 5.0);
        assert_eq!(circle.x, 10.0);
        assert_eq!(circle.y, 20.0);
        assert_eq!(circle.radius, 5.0);
        assert_eq!(circle.color, 7);
        assert_eq!(circle.layer, "0");
    }

    #[test]
    fn test_dxf_circle_builder() {
        let circle = DxfCircle::new(10.0, 20.0, 5.0)
            .color(3)
            .layer("CircleLayer");
        assert_eq!(circle.color, 3);
        assert_eq!(circle.layer, "CircleLayer");
    }

    #[test]
    fn test_dxf_circle_clone() {
        let circle = DxfCircle::new(10.0, 20.0, 5.0);
        let cloned = circle.clone();
        assert_eq!(circle, cloned);
    }

    #[test]
    fn test_dxf_lwpolyline_default() {
        let poly = DxfLwPolyline::default();
        assert!(poly.vertices.is_empty());
        assert!(!poly.closed);
        assert_eq!(poly.color, 7);
        assert_eq!(poly.layer, "0");
    }

    #[test]
    fn test_dxf_lwpolyline_new() {
        let vertices = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
        let poly = DxfLwPolyline::new(vertices.clone());
        assert_eq!(poly.vertices, vertices);
        assert!(!poly.closed);
    }

    #[test]
    fn test_dxf_lwpolyline_closed() {
        let vertices = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
        let poly = DxfLwPolyline::closed(vertices.clone());
        assert_eq!(poly.vertices, vertices);
        assert!(poly.closed);
    }

    #[test]
    fn test_dxf_lwpolyline_builder() {
        let poly = DxfLwPolyline::new(vec![(0.0, 0.0)])
            .add_vertex(10.0, 0.0)
            .add_vertex(10.0, 10.0)
            .set_closed(true)
            .color(5)
            .layer("OutlineLayer");
        assert_eq!(poly.vertices.len(), 3);
        assert!(poly.closed);
        assert_eq!(poly.color, 5);
        assert_eq!(poly.layer, "OutlineLayer");
    }

    #[test]
    fn test_dxf_lwpolyline_clone() {
        let poly = DxfLwPolyline::closed(vec![(0.0, 0.0), (10.0, 10.0)]);
        let cloned = poly.clone();
        assert_eq!(poly, cloned);
    }

    // ================================================================
    // Boundary values
    // ================================================================

    #[test]
    fn test_dxf_line_zero_length() {
        let line = DxfLine::new(5.0, 5.0, 5.0, 5.0);
        assert_eq!(line.x1, line.x2);
        assert_eq!(line.y1, line.y2);
    }

    #[test]
    fn test_dxf_line_negative_coords() {
        let line = DxfLine::new(-100.0, -200.0, -300.0, -400.0);
        assert_eq!(line.x1, -100.0);
        assert_eq!(line.y2, -400.0);
    }

    #[test]
    fn test_dxf_line_large_coords() {
        let line = DxfLine::new(1e12, 1e12, -1e12, -1e12);
        assert_eq!(line.x1, 1e12);
        assert_eq!(line.x2, -1e12);
    }

    #[test]
    fn test_dxf_line_color_zero() {
        let line = DxfLine::new(0.0, 0.0, 1.0, 1.0).color(0);
        assert_eq!(line.color, 0);
    }

    #[test]
    fn test_dxf_line_color_negative() {
        let line = DxfLine::new(0.0, 0.0, 1.0, 1.0).color(-1);
        assert_eq!(line.color, -1);
    }

    #[test]
    fn test_dxf_line_empty_layer() {
        let line = DxfLine::new(0.0, 0.0, 1.0, 1.0).layer("");
        assert_eq!(line.layer, "");
    }

    #[test]
    fn test_dxf_line_builder_chain_overwrite() {
        let line = DxfLine::new(0.0, 0.0, 1.0, 1.0)
            .color(3)
            .color(5)
            .layer("A")
            .layer("B");
        assert_eq!(line.color, 5);
        assert_eq!(line.layer, "B");
    }

    #[test]
    fn test_dxf_line_partial_eq_different() {
        let a = DxfLine::new(0.0, 0.0, 1.0, 1.0);
        let b = DxfLine::new(0.0, 0.0, 1.0, 2.0);
        assert_ne!(a, b);
    }

    #[test]
    fn test_dxf_line_partial_eq_layer_differs() {
        let a = DxfLine::new(0.0, 0.0, 1.0, 1.0).layer("A");
        let b = DxfLine::new(0.0, 0.0, 1.0, 1.0).layer("B");
        assert_ne!(a, b);
    }

    #[test]
    fn test_dxf_text_empty_string() {
        let text = DxfText::new(0.0, 0.0, "");
        assert_eq!(text.text, "");
    }

    #[test]
    fn test_dxf_text_multiline_content() {
        let text = DxfText::new(0.0, 0.0, "line1\nline2");
        assert_eq!(text.text, "line1\nline2");
    }

    #[test]
    fn test_dxf_text_height_zero() {
        let text = DxfText::new(0.0, 0.0, "x").height(0.0);
        assert_eq!(text.height, 0.0);
    }

    #[test]
    fn test_dxf_text_negative_rotation() {
        let text = DxfText::new(0.0, 0.0, "x").rotation(-90.0);
        assert_eq!(text.rotation, -90.0);
    }

    #[test]
    fn test_dxf_text_rotation_360() {
        let text = DxfText::new(0.0, 0.0, "x").rotation(360.0);
        assert_eq!(text.rotation, 360.0);
    }

    #[test]
    fn test_dxf_text_all_alignments() {
        for h in [HorizontalAlignment::Left, HorizontalAlignment::Center, HorizontalAlignment::Right] {
            for v in [VerticalAlignment::Baseline, VerticalAlignment::Bottom, VerticalAlignment::Middle, VerticalAlignment::Top] {
                let text = DxfText::new(0.0, 0.0, "x").align_h(h).align_v(v);
                assert_eq!(text.align_h, h);
                assert_eq!(text.align_v, v);
            }
        }
    }

    #[test]
    fn test_dxf_text_partial_eq_different_content() {
        let a = DxfText::new(0.0, 0.0, "hello");
        let b = DxfText::new(0.0, 0.0, "world");
        assert_ne!(a, b);
    }

    #[test]
    fn test_dxf_circle_zero_radius() {
        let c = DxfCircle::new(0.0, 0.0, 0.0);
        assert_eq!(c.radius, 0.0);
    }

    #[test]
    fn test_dxf_circle_large_radius() {
        let c = DxfCircle::new(0.0, 0.0, 1e9);
        assert_eq!(c.radius, 1e9);
    }

    #[test]
    fn test_dxf_lwpolyline_single_vertex() {
        let poly = DxfLwPolyline::new(vec![(5.0, 5.0)]);
        assert_eq!(poly.vertices.len(), 1);
    }

    #[test]
    fn test_dxf_lwpolyline_empty_vertices() {
        let poly = DxfLwPolyline::new(vec![]);
        assert!(poly.vertices.is_empty());
        assert!(!poly.closed);
    }

    #[test]
    fn test_dxf_lwpolyline_add_vertex_chain() {
        let poly = DxfLwPolyline::default()
            .add_vertex(0.0, 0.0)
            .add_vertex(1.0, 0.0)
            .add_vertex(1.0, 1.0)
            .add_vertex(0.0, 1.0);
        assert_eq!(poly.vertices.len(), 4);
        assert_eq!(poly.vertices[3], (0.0, 1.0));
    }

    #[test]
    fn test_dxf_lwpolyline_set_closed_toggle() {
        let poly = DxfLwPolyline::new(vec![(0.0, 0.0)])
            .set_closed(true)
            .set_closed(false);
        assert!(!poly.closed);
    }

    #[test]
    fn test_dxf_line_debug_format() {
        let line = DxfLine::new(1.0, 2.0, 3.0, 4.0);
        let debug = format!("{:?}", line);
        assert!(debug.contains("DxfLine"));
    }

    #[test]
    fn test_dxf_text_debug_format() {
        let text = DxfText::new(0.0, 0.0, "test");
        let debug = format!("{:?}", text);
        assert!(debug.contains("DxfText"));
    }

    #[test]
    fn test_dxf_circle_debug_format() {
        let c = DxfCircle::new(0.0, 0.0, 1.0);
        let debug = format!("{:?}", c);
        assert!(debug.contains("DxfCircle"));
    }

    #[test]
    fn test_dxf_lwpolyline_debug_format() {
        let poly = DxfLwPolyline::new(vec![(0.0, 0.0)]);
        let debug = format!("{:?}", poly);
        assert!(debug.contains("DxfLwPolyline"));
    }

    #[test]
    fn test_dxf_line_unicode_layer() {
        let line = DxfLine::new(0.0, 0.0, 1.0, 1.0).layer("中心線レイヤー");
        assert_eq!(line.layer, "中心線レイヤー");
    }

    #[test]
    fn test_dxf_text_unicode_content() {
        let text = DxfText::new(0.0, 0.0, "横断歩道 No.1+5");
        assert_eq!(text.text, "横断歩道 No.1+5");
    }
}
