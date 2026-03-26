//! Road section (面積展開図) generation module
//!
//! Generates road section diagrams from station data (centerline distance + widths).
//! Ported from csv_to_dxf's dxf_draw_tenkaiz.py

use dxf_engine::{DxfLine, DxfText, HorizontalAlignment, VerticalAlignment};

/// Station data for road section
#[derive(Clone, Debug, PartialEq)]
pub struct StationData {
    /// Station name (測点名)
    pub name: String,
    /// Cumulative distance along centerline (累積延長) in meters
    pub x: f64,
    /// Left width from centerline (左幅員) in meters
    pub wl: f64,
    /// Right width from centerline (右幅員) in meters
    pub wr: f64,
}

impl StationData {
    pub fn new(name: &str, x: f64, wl: f64, wr: f64) -> Self {
        Self {
            name: name.to_string(),
            x,
            wl,
            wr,
        }
    }
}

/// Geometry output from road section calculation
#[derive(Clone, Debug, Default)]
pub struct RoadSectionGeometry {
    /// Line segments
    pub lines: Vec<LineSegment>,
    /// Dimension texts
    pub texts: Vec<DimensionText>,
}

/// A line segment with optional color
#[derive(Clone, Debug)]
pub struct LineSegment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub color: i32,
}

impl LineSegment {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2, color: 7 }
    }

    pub fn with_color(x1: f64, y1: f64, x2: f64, y2: f64, color: i32) -> Self {
        Self { x1, y1, x2, y2, color }
    }
}

/// A dimension/label text with position and rotation
#[derive(Clone, Debug)]
pub struct DimensionText {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
    pub height: f64,
    pub color: i32,
    pub align_h: HorizontalAlignment,
    pub align_v: VerticalAlignment,
}

impl DimensionText {
    pub fn new(text: &str, x: f64, y: f64) -> Self {
        Self {
            text: text.to_string(),
            x,
            y,
            rotation: 0.0,
            height: 350.0,
            color: 7,
            align_h: HorizontalAlignment::Center,
            align_v: VerticalAlignment::Middle,
        }
    }

    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_color(mut self, color: i32) -> Self {
        self.color = color;
        self
    }

    pub fn with_alignment(mut self, h: HorizontalAlignment, v: VerticalAlignment) -> Self {
        self.align_h = h;
        self.align_v = v;
        self
    }
}

/// Configuration for road section generation
#[derive(Clone, Debug)]
pub struct RoadSectionConfig {
    /// Scale factor (default: 1000.0 for mm output)
    pub scale: f64,
    /// Text height for dimensions
    pub text_height: f64,
    /// Offset from width line end for dimension text
    pub text_offset: f64,
}

impl Default for RoadSectionConfig {
    fn default() -> Self {
        Self {
            scale: 1000.0,
            text_height: 350.0,
            text_offset: 500.0,
        }
    }
}

/// Calculate road section geometry from station data
pub fn calculate_road_section(
    stations: &[StationData],
    config: &RoadSectionConfig,
) -> RoadSectionGeometry {
    let mut geometry = RoadSectionGeometry::default();

    if stations.is_empty() {
        return geometry;
    }

    let scale = config.scale;
    let text_offset = config.text_offset;

    let mut prev_points: Option<((f64, f64), (f64, f64), (f64, f64))> = None;
    let mut prev_x_unscaled: f64 = 0.0;

    for station in stations {
        let x_scaled = station.x * scale;
        let wl_scaled = station.wl * scale;
        let wr_scaled = station.wr * scale;

        let pt_left = (x_scaled, wl_scaled);
        let pt_center = (x_scaled, 0.0);
        let pt_right = (x_scaled, -wr_scaled);

        // Width lines
        geometry.lines.push(LineSegment::new(
            pt_center.0, pt_center.1, pt_left.0, pt_left.1,
        ));
        geometry.lines.push(LineSegment::new(
            pt_center.0, pt_center.1, pt_right.0, pt_right.1,
        ));

        // Connect to previous station
        if let Some((prev_left, prev_center, prev_right)) = prev_points {
            let distance = x_scaled - prev_center.0;

            if distance > 0.0 {
                // Center line
                geometry.lines.push(LineSegment::new(
                    prev_center.0, prev_center.1, pt_center.0, pt_center.1,
                ));

                // Top outline
                if pt_left.1 > 0.0 || prev_left.1 > 0.0 {
                    geometry.lines.push(LineSegment::new(
                        prev_left.0, prev_left.1, pt_left.0, pt_left.1,
                    ));
                }

                // Bottom outline
                if pt_right.1 < 0.0 || prev_right.1 < 0.0 {
                    geometry.lines.push(LineSegment::new(
                        prev_right.0, prev_right.1, pt_right.0, pt_right.1,
                    ));
                }

                // Distance dimension
                let distance_unscaled = station.x - prev_x_unscaled;
                let mid_x = (prev_center.0 + pt_center.0) * 0.5;
                let v_align = if distance < 1000.0 {
                    VerticalAlignment::Bottom
                } else {
                    VerticalAlignment::Top
                };
                geometry.texts.push(
                    DimensionText::new(&format!("{:.2}", distance_unscaled), mid_x, 0.0)
                        .with_alignment(HorizontalAlignment::Center, v_align)
                );
            }
        }

        let tankyori = x_scaled - prev_points.map(|p| p.1.0).unwrap_or(0.0);
        let v_align = if tankyori < 1000.0 {
            VerticalAlignment::Bottom
        } else {
            VerticalAlignment::Top
        };

        // Left width dimension
        if station.wl > 0.0 {
            let text_y = wl_scaled + text_offset;
            geometry.texts.push(
                DimensionText::new(&format!("{:.2}", station.wl), x_scaled, text_y)
                    .with_rotation(-90.0)
                    .with_alignment(HorizontalAlignment::Center, v_align)
            );
        }

        // Right width dimension
        if station.wr > 0.0 {
            let text_y = -wr_scaled - text_offset;
            geometry.texts.push(
                DimensionText::new(&format!("{:.2}", station.wr), x_scaled, text_y)
                    .with_rotation(-90.0)
                    .with_alignment(HorizontalAlignment::Center, v_align)
            );
        }

        // Station name label (blue)
        let label_y = if station.wl > 0.0 {
            wl_scaled + 2000.0
        } else {
            2000.0
        };
        geometry.texts.push(
            DimensionText::new(&station.name, x_scaled, label_y)
                .with_rotation(-90.0)
                .with_color(5)
                .with_alignment(HorizontalAlignment::Center, VerticalAlignment::Bottom)
        );

        prev_points = Some((pt_left, pt_center, pt_right));
        prev_x_unscaled = station.x;
    }

    geometry
}

/// Convert geometry to DXF entities
pub fn geometry_to_dxf(geometry: &RoadSectionGeometry) -> (Vec<DxfLine>, Vec<DxfText>) {
    let lines: Vec<DxfLine> = geometry.lines.iter()
        .map(|seg| DxfLine::new(seg.x1, seg.y1, seg.x2, seg.y2).color(seg.color))
        .collect();

    let texts: Vec<DxfText> = geometry.texts.iter()
        .map(|dim| {
            DxfText::new(dim.x, dim.y, &dim.text)
                .height(dim.height)
                .rotation(dim.rotation)
                .color(dim.color)
                .align_h(dim.align_h)
                .align_v(dim.align_v)
        })
        .collect();

    (lines, texts)
}

/// Parse road section CSV data
pub fn parse_road_section_csv(content: &str) -> Result<Vec<StationData>, String> {
    let mut stations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Err("Empty CSV".to_string());
    }

    let mut start_row = 0;
    let mut name_col = 0;
    let mut x_col = 1;
    let mut wl_col = 2;
    let mut wr_col = 3;

    let first_line = lines[0];
    let first_parts: Vec<&str> = first_line.split(',').map(|s| s.trim()).collect();

    let is_header = first_parts.iter().any(|p| {
        p.contains("測点") || p.contains("延長") || p.contains("幅員") ||
        p.to_lowercase().contains("name") || p.to_lowercase().contains("station")
    });

    if is_header {
        start_row = 1;
        for (i, part) in first_parts.iter().enumerate() {
            let lower = part.to_lowercase();
            if lower.contains("測点") || lower.contains("name") || lower.contains("station") {
                name_col = i;
            } else if lower.contains("延長") || lower.contains("距離") || lower == "x" {
                x_col = i;
            } else if lower.contains("左") || lower == "wl" {
                wl_col = i;
            } else if lower.contains("右") || lower == "wr" {
                wr_col = i;
            }
        }
    }

    for (line_num, line) in lines.iter().enumerate().skip(start_row) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 4 {
            continue;
        }

        let name = parts.get(name_col).unwrap_or(&"").to_string();
        let x: f64 = parts.get(x_col)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("Line {}: invalid x value", line_num + 1))?;
        let wl: f64 = parts.get(wl_col)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let wr: f64 = parts.get(wr_col)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        stations.push(StationData::new(&name, x, wl, wr));
    }

    if stations.is_empty() {
        return Err("No valid station data found".to_string());
    }

    Ok(stations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_station_data() {
        let station = StationData::new("No.1", 0.0, 2.5, 2.5);
        assert_eq!(station.name, "No.1");
        assert_eq!(station.x, 0.0);
        assert_eq!(station.wl, 2.5);
        assert_eq!(station.wr, 2.5);
    }

    #[test]
    fn test_parse_road_section_csv() {
        let csv = "測点名,累積延長,左幅員,右幅員\nNo.1,0.0,2.5,2.5\nNo.1+10,10.0,2.5,3.0\nNo.2,20.0,2.5,2.5\n";
        let result = parse_road_section_csv(csv);
        assert!(result.is_ok());
        let stations = result.unwrap();
        assert_eq!(stations.len(), 3);
        assert_eq!(stations[0].name, "No.1");
        assert_eq!(stations[1].x, 10.0);
        assert_eq!(stations[2].wr, 2.5);
    }

    #[test]
    fn test_calculate_road_section() {
        let stations = vec![
            StationData::new("No.1", 0.0, 2.5, 2.5),
            StationData::new("No.2", 10.0, 2.5, 2.5),
        ];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        assert!(!geometry.lines.is_empty());
        assert!(!geometry.texts.is_empty());
    }

    #[test]
    fn test_geometry_to_dxf() {
        let stations = vec![
            StationData::new("No.1", 0.0, 2.5, 2.5),
            StationData::new("No.2", 10.0, 3.0, 2.0),
        ];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let (lines, texts) = geometry_to_dxf(&geometry);
        assert!(!lines.is_empty());
        assert!(!texts.is_empty());

        let rotated_texts: Vec<_> = texts.iter().filter(|t| t.rotation != 0.0).collect();
        assert!(!rotated_texts.is_empty());
        for text in &rotated_texts {
            assert_eq!(text.rotation, -90.0);
        }
    }

    #[test]
    fn test_station_name_color() {
        let stations = vec![
            StationData::new("No.1", 0.0, 2.5, 2.5),
            StationData::new("No.2", 10.0, 3.0, 2.0),
        ];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let (_, texts) = geometry_to_dxf(&geometry);

        let station_name_texts: Vec<_> = texts.iter()
            .filter(|t| t.text.starts_with("No."))
            .collect();
        for text in &station_name_texts {
            assert_eq!(text.color, 5);
        }
    }

    // ================================================================
    // Empty stations
    // ================================================================

    #[test]
    fn test_calculate_empty_stations() {
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&[], &config);
        assert!(geometry.lines.is_empty());
        assert!(geometry.texts.is_empty());
    }

    // ================================================================
    // Single station — no connecting lines
    // ================================================================

    #[test]
    fn test_calculate_single_station() {
        let stations = vec![StationData::new("No.0", 0.0, 3.0, 2.5)];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);

        // Single station: 2 width lines (left + right), station name, width dims
        assert_eq!(geometry.lines.len(), 2, "Single station = 2 width lines");
        // Texts: left width dim + right width dim + station name = 3
        assert!(geometry.texts.len() >= 2, "Should have width dims and station name");
    }

    // ================================================================
    // Many stations
    // ================================================================

    #[test]
    fn test_calculate_many_stations() {
        let stations: Vec<StationData> = (0..10)
            .map(|i| StationData::new(&format!("No.{}", i), i as f64 * 10.0, 2.5, 2.5))
            .collect();
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        assert!(!geometry.lines.is_empty());
        // Each pair of consecutive stations adds: center, top, bottom connecting lines
        // Plus 2 width lines per station
    }

    // ================================================================
    // Zero width (one side only)
    // ================================================================

    #[test]
    fn test_calculate_zero_left_width() {
        let stations = vec![
            StationData::new("No.0", 0.0, 0.0, 3.0),
            StationData::new("No.1", 10.0, 0.0, 3.0),
        ];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        // Left width is 0 → left width line still drawn (center to center)
        // No left width dimension text (wl == 0)
        let left_dims: Vec<_> = geometry.texts.iter()
            .filter(|t| t.text.parse::<f64>().is_ok() && t.y > 0.0)
            .collect();
        // When wl=0, the left dimension text should NOT be generated
        assert!(left_dims.is_empty(), "Zero left width should produce no left dim text");
    }

    #[test]
    fn test_calculate_zero_right_width() {
        let stations = vec![
            StationData::new("No.0", 0.0, 3.0, 0.0),
            StationData::new("No.1", 10.0, 3.0, 0.0),
        ];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let right_dims: Vec<_> = geometry.texts.iter()
            .filter(|t| t.text.parse::<f64>().is_ok() && t.y < 0.0)
            .collect();
        assert!(right_dims.is_empty(), "Zero right width should produce no right dim text");
    }

    // ================================================================
    // Asymmetric widths
    // ================================================================

    #[test]
    fn test_calculate_asymmetric_widths() {
        let stations = vec![
            StationData::new("No.0", 0.0, 5.0, 1.0),
            StationData::new("No.1", 10.0, 1.0, 5.0),
        ];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let (lines, _) = geometry_to_dxf(&geometry);
        assert!(!lines.is_empty());
    }

    // ================================================================
    // Scale conversion accuracy
    // ================================================================

    #[test]
    fn test_scale_conversion() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.0, 3.0),
            StationData::new("No.1", 10.0, 2.0, 3.0),
        ];
        let config = RoadSectionConfig { scale: 1000.0, ..Default::default() };
        let geometry = calculate_road_section(&stations, &config);

        // Station at x=10.0m should have x_scaled = 10000.0mm
        let max_x = geometry.lines.iter()
            .flat_map(|l| vec![l.x1, l.x2])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((max_x - 10000.0).abs() < 1.0,
            "10m * 1000 scale = 10000mm, got {}", max_x);

        // Left width 2.0m → y = 2000mm
        let max_y = geometry.lines.iter()
            .flat_map(|l| vec![l.y1, l.y2])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((max_y - 2000.0).abs() < 1.0,
            "2m left width * 1000 = 2000mm, got {}", max_y);
    }

    // ================================================================
    // geometry_to_dxf text properties
    // ================================================================

    #[test]
    fn test_geometry_to_dxf_text_rotation() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.5, 2.5),
            StationData::new("No.1", 10.0, 2.5, 2.5),
        ];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let (_, texts) = geometry_to_dxf(&geometry);

        // Width dimension texts should be rotated -90°
        let width_texts: Vec<_> = texts.iter()
            .filter(|t| t.rotation != 0.0)
            .collect();
        for text in &width_texts {
            assert_eq!(text.rotation, -90.0);
        }
    }

    #[test]
    fn test_geometry_to_dxf_station_name_blue() {
        let stations = vec![StationData::new("ABC", 0.0, 2.5, 2.5)];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let (_, texts) = geometry_to_dxf(&geometry);

        let name_texts: Vec<_> = texts.iter()
            .filter(|t| t.text == "ABC")
            .collect();
        assert_eq!(name_texts.len(), 1);
        assert_eq!(name_texts[0].color, 5, "Station name must be blue (color 5)");
    }

    // ================================================================
    // CSV parsing edge cases
    // ================================================================

    #[test]
    fn test_parse_csv_empty() {
        let result = parse_road_section_csv("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_csv_header_only() {
        let result = parse_road_section_csv("測点名,累積延長,左幅員,右幅員\n");
        assert!(result.is_err(), "Header-only CSV should error");
    }

    #[test]
    fn test_parse_csv_comment_lines_skipped() {
        let csv = "name,x,wl,wr\n# this is a comment\nNo.0,0.0,2.5,2.5\n";
        let result = parse_road_section_csv(csv).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_parse_csv_empty_lines_skipped() {
        let csv = "name,x,wl,wr\n\nNo.0,0.0,2.5,2.5\n\nNo.1,10.0,3.0,3.0\n\n";
        let result = parse_road_section_csv(csv).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_csv_insufficient_columns() {
        // Lines with < 4 columns should be skipped
        let csv = "name,x,wl,wr\nNo.0,0.0,2.5,2.5\nshort,1.0\nNo.1,10.0,3.0,3.0\n";
        let result = parse_road_section_csv(csv).unwrap();
        assert_eq!(result.len(), 2, "Short lines should be skipped");
    }

    #[test]
    fn test_parse_csv_japanese_headers() {
        let csv = "測点名,累積延長,左幅員,右幅員\nNo.0,0.0,2.5,2.5\n";
        let result = parse_road_section_csv(csv).unwrap();
        assert_eq!(result[0].name, "No.0");
        assert_eq!(result[0].wl, 2.5);
        assert_eq!(result[0].wr, 2.5);
    }

    #[test]
    fn test_parse_csv_no_header() {
        let csv = "No.0,0.0,2.5,3.0\nNo.1,10.0,2.5,3.0\n";
        let result = parse_road_section_csv(csv).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "No.0");
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[0].wl, 2.5);
        assert_eq!(result[0].wr, 3.0);
    }

    #[test]
    fn test_parse_csv_wl_wr_default_to_zero() {
        // wl/wr columns have non-numeric values → default to 0.0
        let csv = "name,x,wl,wr\nNo.0,0.0,abc,def\n";
        let result = parse_road_section_csv(csv).unwrap();
        assert_eq!(result[0].wl, 0.0);
        assert_eq!(result[0].wr, 0.0);
    }

    // ================================================================
    // DimensionText builder
    // ================================================================

    #[test]
    fn test_dimension_text_builder() {
        let dt = DimensionText::new("test", 100.0, 200.0)
            .with_rotation(-90.0)
            .with_color(5)
            .with_alignment(HorizontalAlignment::Left, VerticalAlignment::Top);
        assert_eq!(dt.text, "test");
        assert_eq!(dt.x, 100.0);
        assert_eq!(dt.y, 200.0);
        assert_eq!(dt.rotation, -90.0);
        assert_eq!(dt.color, 5);
    }

    // ================================================================
    // LineSegment constructors
    // ================================================================

    #[test]
    fn test_line_segment_default_color() {
        let seg = LineSegment::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(seg.color, 7, "Default color should be 7 (white)");
    }

    #[test]
    fn test_line_segment_with_color() {
        let seg = LineSegment::with_color(0.0, 0.0, 100.0, 100.0, 3);
        assert_eq!(seg.color, 3);
    }

    // ================================================================
    // Distance dimension text
    // ================================================================

    #[test]
    fn test_distance_dimension_text() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.5, 2.5),
            StationData::new("No.1", 15.5, 2.5, 2.5),
        ];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);

        // There should be a dimension text showing "15.50" (the distance)
        let dist_text = geometry.texts.iter()
            .find(|t| t.text == "15.50");
        assert!(dist_text.is_some(), "Should have distance dimension text '15.50'");
    }
}
