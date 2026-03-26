//! DXF entity → egui shape renderer
//!
//! Converts RoadSectionGeometry (and raw DXF entities) into egui painter calls.
//! Handles coordinate transformation: DXF Y-up → screen Y-down.

use egui::{Color32, Pos2, Stroke, Vec2};
use road_section::RoadSectionGeometry;

/// Viewport transformation state
pub struct Viewport {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub origin: Pos2,
}

impl Viewport {
    /// Compute viewport from geometry bounding box and available canvas size.
    pub fn from_geometry(geometry: &RoadSectionGeometry, canvas_size: Vec2, origin: Pos2) -> Option<Self> {
        if geometry.lines.is_empty() {
            return None;
        }

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for seg in &geometry.lines {
            min_x = min_x.min(seg.x1).min(seg.x2);
            max_x = max_x.max(seg.x1).max(seg.x2);
            min_y = min_y.min(seg.y1).min(seg.y2);
            max_y = max_y.max(seg.y1).max(seg.y2);
        }

        let data_w = (max_x - min_x).max(1.0);
        let data_h = (max_y - min_y).max(1.0);

        let scale = (canvas_size.x / data_w as f32).min(canvas_size.y / data_h as f32) * 0.9;
        let offset_x = (canvas_size.x - data_w as f32 * scale) / 2.0;
        let offset_y = (canvas_size.y - data_h as f32 * scale) / 2.0;

        Some(Self {
            min_x, max_x, min_y, max_y,
            scale, offset_x, offset_y, origin,
        })
    }

    /// Transform DXF coordinate (Y-up) to screen coordinate (Y-down).
    pub fn to_screen(&self, x: f64, y: f64) -> Pos2 {
        Pos2::new(
            self.origin.x + self.offset_x + (x - self.min_x) as f32 * self.scale,
            self.origin.y + self.offset_y + (self.max_y - y) as f32 * self.scale,
        )
    }
}

/// Render road section geometry onto an egui painter.
pub fn render_road_section(
    painter: &egui::Painter,
    geometry: &RoadSectionGeometry,
    viewport: &Viewport,
) {
    // Draw line segments
    for seg in &geometry.lines {
        let color = dxf_color_to_egui(seg.color);
        painter.line_segment(
            [viewport.to_screen(seg.x1, seg.y1), viewport.to_screen(seg.x2, seg.y2)],
            Stroke::new(1.0, color),
        );
    }

    // Draw dimension texts (station names in blue)
    for dim in &geometry.texts {
        if dim.color == 5 {
            let pos = viewport.to_screen(dim.x, dim.y);
            painter.text(
                pos,
                egui::Align2::CENTER_BOTTOM,
                &dim.text,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(0, 128, 255),
            );
        }
    }
}

/// Map DXF color index to egui Color32.
pub fn dxf_color_to_egui(color: i32) -> Color32 {
    match color {
        1 => Color32::RED,
        2 => Color32::YELLOW,
        3 => Color32::GREEN,
        4 => Color32::from_rgb(0, 255, 255),
        5 => Color32::from_rgb(0, 128, 255),
        6 => Color32::from_rgb(255, 0, 255),
        7 => Color32::WHITE,
        _ => Color32::LIGHT_GRAY,
    }
}
