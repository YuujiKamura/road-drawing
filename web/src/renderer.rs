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

#[cfg(test)]
mod tests {
    use super::*;
    use road_section::{LineSegment, DimensionText, RoadSectionGeometry, StationData, RoadSectionConfig};

    fn make_geometry(stations: &[StationData]) -> RoadSectionGeometry {
        let config = RoadSectionConfig::default();
        road_section::calculate_road_section(stations, &config)
    }

    // ================================================================
    // Viewport::from_geometry
    // ================================================================

    #[test]
    fn test_viewport_from_empty_geometry_returns_none() {
        let geom = RoadSectionGeometry::default();
        let vp = Viewport::from_geometry(&geom, Vec2::new(800.0, 600.0), Pos2::ZERO);
        assert!(vp.is_none());
    }

    #[test]
    fn test_viewport_from_single_station() {
        let stations = vec![StationData::new("No.0", 0.0, 2.5, 2.5)];
        let geom = make_geometry(&stations);
        let vp = Viewport::from_geometry(&geom, Vec2::new(800.0, 600.0), Pos2::ZERO);
        assert!(vp.is_some(), "Single station geometry should produce viewport");
    }

    #[test]
    fn test_viewport_from_two_stations_bounds() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.0, 3.0),
            StationData::new("No.1", 10.0, 2.0, 3.0),
        ];
        let geom = make_geometry(&stations);
        let vp = Viewport::from_geometry(&geom, Vec2::new(800.0, 600.0), Pos2::ZERO).unwrap();

        // x range: 0 to 10000 (10m * 1000 scale)
        assert!((vp.min_x - 0.0).abs() < 0.01);
        assert!((vp.max_x - 10000.0).abs() < 0.01);
        // y range: -3000 (right width) to 2000 (left width)
        assert!(vp.min_y < 0.0, "min_y should be negative (right width)");
        assert!(vp.max_y > 0.0, "max_y should be positive (left width)");
    }

    #[test]
    fn test_viewport_scale_positive() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.5, 2.5),
            StationData::new("No.1", 20.0, 2.5, 2.5),
        ];
        let geom = make_geometry(&stations);
        let vp = Viewport::from_geometry(&geom, Vec2::new(800.0, 600.0), Pos2::ZERO).unwrap();
        assert!(vp.scale > 0.0, "Scale must be positive, got {}", vp.scale);
    }

    #[test]
    fn test_viewport_from_geometry_with_custom_origin() {
        let stations = vec![
            StationData::new("No.0", 0.0, 1.0, 1.0),
            StationData::new("No.1", 5.0, 1.0, 1.0),
        ];
        let geom = make_geometry(&stations);
        let origin = Pos2::new(100.0, 50.0);
        let vp = Viewport::from_geometry(&geom, Vec2::new(400.0, 300.0), origin).unwrap();
        assert_eq!(vp.origin, origin);
    }

    // ================================================================
    // Viewport::to_screen — coordinate transformation
    // ================================================================

    #[test]
    fn test_to_screen_y_flip() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.0, 2.0),
            StationData::new("No.1", 10.0, 2.0, 2.0),
        ];
        let geom = make_geometry(&stations);
        let vp = Viewport::from_geometry(&geom, Vec2::new(800.0, 600.0), Pos2::ZERO).unwrap();

        let top_dxf = vp.to_screen(0.0, vp.max_y);    // DXF top → screen top (small Y)
        let bot_dxf = vp.to_screen(0.0, vp.min_y);    // DXF bottom → screen bottom (large Y)
        assert!(top_dxf.y < bot_dxf.y,
            "DXF max_y should map to smaller screen Y: top={}, bot={}", top_dxf.y, bot_dxf.y);
    }

    #[test]
    fn test_to_screen_x_increases_right() {
        let stations = vec![
            StationData::new("No.0", 0.0, 1.0, 1.0),
            StationData::new("No.1", 10.0, 1.0, 1.0),
        ];
        let geom = make_geometry(&stations);
        let vp = Viewport::from_geometry(&geom, Vec2::new(800.0, 600.0), Pos2::ZERO).unwrap();

        let left = vp.to_screen(vp.min_x, 0.0);
        let right = vp.to_screen(vp.max_x, 0.0);
        assert!(right.x > left.x,
            "Larger DXF X should map to larger screen X: left={}, right={}", left.x, right.x);
    }

    #[test]
    fn test_to_screen_min_maps_to_offset() {
        let vp = Viewport {
            min_x: 0.0, max_x: 100.0, min_y: 0.0, max_y: 100.0,
            scale: 1.0, offset_x: 10.0, offset_y: 20.0,
            origin: Pos2::new(5.0, 5.0),
        };
        let p = vp.to_screen(0.0, 100.0); // min_x, max_y → top-left
        assert!((p.x - (5.0 + 10.0)).abs() < 0.01, "x = origin.x + offset_x");
        assert!((p.y - (5.0 + 20.0)).abs() < 0.01, "y = origin.y + offset_y (max_y - max_y = 0)");
    }

    #[test]
    fn test_to_screen_max_maps_correctly() {
        let vp = Viewport {
            min_x: 0.0, max_x: 200.0, min_y: 0.0, max_y: 100.0,
            scale: 2.0, offset_x: 0.0, offset_y: 0.0,
            origin: Pos2::ZERO,
        };
        let p = vp.to_screen(200.0, 0.0); // max_x, min_y → bottom-right
        assert!((p.x - 400.0).abs() < 0.01, "x = (200-0)*2 = 400");
        assert!((p.y - 200.0).abs() < 0.01, "y = (100-0)*2 = 200");
    }

    // ================================================================
    // dxf_color_to_egui — all color mappings
    // ================================================================

    #[test]
    fn test_dxf_color_red() {
        assert_eq!(dxf_color_to_egui(1), Color32::RED);
    }

    #[test]
    fn test_dxf_color_yellow() {
        assert_eq!(dxf_color_to_egui(2), Color32::YELLOW);
    }

    #[test]
    fn test_dxf_color_green() {
        assert_eq!(dxf_color_to_egui(3), Color32::GREEN);
    }

    #[test]
    fn test_dxf_color_cyan() {
        assert_eq!(dxf_color_to_egui(4), Color32::from_rgb(0, 255, 255));
    }

    #[test]
    fn test_dxf_color_blue() {
        assert_eq!(dxf_color_to_egui(5), Color32::from_rgb(0, 128, 255));
    }

    #[test]
    fn test_dxf_color_magenta() {
        assert_eq!(dxf_color_to_egui(6), Color32::from_rgb(255, 0, 255));
    }

    #[test]
    fn test_dxf_color_white() {
        assert_eq!(dxf_color_to_egui(7), Color32::WHITE);
    }

    #[test]
    fn test_dxf_color_unknown_defaults_to_light_gray() {
        assert_eq!(dxf_color_to_egui(0), Color32::LIGHT_GRAY);
        assert_eq!(dxf_color_to_egui(8), Color32::LIGHT_GRAY);
        assert_eq!(dxf_color_to_egui(-1), Color32::LIGHT_GRAY);
        assert_eq!(dxf_color_to_egui(255), Color32::LIGHT_GRAY);
    }

    // ================================================================
    // Geometry shape counts — verify expected line/text counts
    // ================================================================

    #[test]
    fn test_geometry_shape_count_single_station() {
        let stations = vec![StationData::new("No.0", 0.0, 2.5, 2.5)];
        let geom = make_geometry(&stations);
        // Single station: 2 width lines (center→left, center→right)
        assert_eq!(geom.lines.len(), 2, "Single station should have 2 width lines");
        // Texts: left width dim + right width dim + station name = 3
        let station_names: Vec<_> = geom.texts.iter().filter(|t| t.color == 5).collect();
        assert_eq!(station_names.len(), 1, "Single station should have 1 name label");
    }

    #[test]
    fn test_geometry_shape_count_two_stations() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.5, 2.5),
            StationData::new("No.1", 20.0, 2.5, 2.5),
        ];
        let geom = make_geometry(&stations);
        // 2 stations: 2+2 width lines + 3 connecting lines (center, top, bottom) = 7
        assert_eq!(geom.lines.len(), 7,
            "Two stations should have 4 width + 3 connecting = 7 lines");
        let station_names: Vec<_> = geom.texts.iter().filter(|t| t.color == 5).collect();
        assert_eq!(station_names.len(), 2, "Two stations should have 2 name labels");
    }

    #[test]
    fn test_geometry_shape_count_three_stations() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.5, 2.5),
            StationData::new("No.1", 20.0, 2.5, 2.5),
            StationData::new("No.2", 40.0, 2.5, 2.5),
        ];
        let geom = make_geometry(&stations);
        // 3 stations: 3×2 width + 2×3 connecting = 12
        assert_eq!(geom.lines.len(), 12,
            "Three stations should have 6 width + 6 connecting = 12 lines");
        let station_names: Vec<_> = geom.texts.iter().filter(|t| t.color == 5).collect();
        assert_eq!(station_names.len(), 3);
    }

    #[test]
    fn test_geometry_shape_count_zero_width_no_outline() {
        let stations = vec![
            StationData::new("No.0", 0.0, 0.0, 0.0),
            StationData::new("No.1", 20.0, 0.0, 0.0),
        ];
        let geom = make_geometry(&stations);
        // Width = 0: width lines still drawn (center→center = degenerate)
        // But top/bottom outlines skipped (wl=0, wr=0)
        // 2+2 width lines + 1 center line = 5
        assert_eq!(geom.lines.len(), 5,
            "Zero width: 4 width + 1 center connecting = 5 lines (no top/bottom outline)");
    }

    #[test]
    fn test_geometry_texts_include_distance_dimensions() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.5, 2.5),
            StationData::new("No.1", 15.5, 2.5, 2.5),
        ];
        let geom = make_geometry(&stations);
        let dist_text = geom.texts.iter().find(|t| t.text == "15.50");
        assert!(dist_text.is_some(), "Should have distance dimension '15.50'");
    }

    #[test]
    fn test_geometry_texts_width_dimensions_rotated() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.5, 3.0),
        ];
        let geom = make_geometry(&stations);
        let width_texts: Vec<_> = geom.texts.iter()
            .filter(|t| t.text == "2.50" || t.text == "3.00")
            .collect();
        assert_eq!(width_texts.len(), 2, "Should have left (2.50) and right (3.00) width dims");
        for t in &width_texts {
            assert_eq!(t.rotation, -90.0, "Width dimensions should be rotated -90°");
        }
    }

    // ================================================================
    // Viewport edge cases
    // ================================================================

    #[test]
    fn test_viewport_from_geometry_single_point() {
        // All lines collapse to a single point → data_w and data_h clamped to 1.0
        let mut geom = RoadSectionGeometry::default();
        geom.lines.push(LineSegment::new(5.0, 5.0, 5.0, 5.0));
        let vp = Viewport::from_geometry(&geom, Vec2::new(800.0, 600.0), Pos2::ZERO).unwrap();
        assert!(vp.scale > 0.0, "Scale should still be positive for single-point geometry");
    }

    #[test]
    fn test_viewport_narrow_canvas() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.0, 2.0),
            StationData::new("No.1", 100.0, 2.0, 2.0),
        ];
        let geom = make_geometry(&stations);
        // Very narrow canvas: width constrains the scale
        let vp = Viewport::from_geometry(&geom, Vec2::new(100.0, 10000.0), Pos2::ZERO).unwrap();
        let data_w = vp.max_x - vp.min_x;
        let data_h = vp.max_y - vp.min_y;
        let expected_scale = (100.0 / data_w as f32).min(10000.0 / data_h as f32) * 0.9;
        assert!((vp.scale - expected_scale).abs() < 0.001);
    }

    // ================================================================
    // Viewport: screen coordinates stay within canvas bounds
    // ================================================================

    #[test]
    fn test_to_screen_stays_within_canvas() {
        let stations = vec![
            StationData::new("No.0", 0.0, 5.0, 5.0),
            StationData::new("No.1", 50.0, 5.0, 5.0),
        ];
        let geom = make_geometry(&stations);
        let canvas = Vec2::new(800.0, 600.0);
        let origin = Pos2::ZERO;
        let vp = Viewport::from_geometry(&geom, canvas, origin).unwrap();

        // All geometry points should map within canvas
        for seg in &geom.lines {
            for (x, y) in [(seg.x1, seg.y1), (seg.x2, seg.y2)] {
                let p = vp.to_screen(x, y);
                assert!(p.x >= origin.x && p.x <= origin.x + canvas.x,
                    "Screen x={} out of canvas [0, {}]", p.x, canvas.x);
                assert!(p.y >= origin.y && p.y <= origin.y + canvas.y,
                    "Screen y={} out of canvas [0, {}]", p.y, canvas.y);
            }
        }
    }

    // ================================================================
    // Viewport: scale adapts to aspect ratio
    // ================================================================

    #[test]
    fn test_viewport_wide_data_on_square_canvas() {
        // Wide data (100m road, 4m width) on square canvas → scale limited by width
        let stations = vec![
            StationData::new("No.0", 0.0, 2.0, 2.0),
            StationData::new("No.5", 100.0, 2.0, 2.0),
        ];
        let geom = make_geometry(&stations);
        let vp = Viewport::from_geometry(&geom, Vec2::new(500.0, 500.0), Pos2::ZERO).unwrap();
        let data_w = (vp.max_x - vp.min_x) as f32;
        let data_h = (vp.max_y - vp.min_y) as f32;
        // Wide data: scale determined by canvas_w / data_w
        let scale_by_w = 500.0 / data_w * 0.9;
        let scale_by_h = 500.0 / data_h * 0.9;
        assert!(scale_by_w < scale_by_h,
            "Wide data should be width-constrained: w_scale={} < h_scale={}", scale_by_w, scale_by_h);
        assert!((vp.scale - scale_by_w).abs() < 0.001);
    }

    // ================================================================
    // Geometry → DXF export → re-parse → shape count consistency
    // ================================================================

    #[test]
    fn test_geometry_to_dxf_export_shape_count() {
        let stations = vec![
            StationData::new("No.0", 0.0, 3.0, 3.0),
            StationData::new("No.1", 20.0, 3.0, 3.0),
            StationData::new("No.2", 40.0, 3.0, 3.0),
        ];
        let geom = make_geometry(&stations);
        let dxf_str = crate::renderer::tests::export_geometry_to_dxf(&geom);
        let doc = dxf_engine::parse_dxf(&dxf_str).unwrap();

        // Lines should match geometry
        assert_eq!(doc.lines.len(), geom.lines.len());
        // Station name texts (color=5) count should match
        let geom_names = geom.texts.iter().filter(|t| t.color == 5).count();
        let dxf_names = doc.texts.iter().filter(|t| t.color == 5).count();
        assert_eq!(dxf_names, geom_names,
            "Station name count should survive roundtrip: geom={}, dxf={}", geom_names, dxf_names);
    }

    /// Helper: export geometry to DXF string (same as dxf_export module)
    fn export_geometry_to_dxf(geometry: &RoadSectionGeometry) -> String {
        let (lines, texts) = road_section::geometry_to_dxf(geometry);
        let writer = dxf_engine::DxfWriter::new();
        writer.write(&lines, &texts)
    }

    // ================================================================
    // Asymmetric width rendering
    // ================================================================

    #[test]
    fn test_geometry_asymmetric_widths_both_sides_rendered() {
        let stations = vec![
            StationData::new("No.0", 0.0, 5.0, 0.0),  // left only
            StationData::new("No.1", 20.0, 0.0, 4.0),  // right only
        ];
        let geom = make_geometry(&stations);
        // Each station: 2 width lines. Connection: center + one outline (partial)
        assert!(geom.lines.len() >= 5, "Asymmetric widths should still generate lines");

        // Width dimension texts
        let has_left = geom.texts.iter().any(|t| t.text == "5.00");
        let has_right = geom.texts.iter().any(|t| t.text == "4.00");
        assert!(has_left, "Should have left width dim 5.00");
        assert!(has_right, "Should have right width dim 4.00");
    }
}
