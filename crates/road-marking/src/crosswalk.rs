//! Crosswalk (横断歩道) stripe generation
//!
//! Generates crosswalk stripes perpendicular to a centerline path.
//! Stripes are placed symmetrically around the centerline.
//!
//! From Kotlin CrosswalkGenerator.kt:
//! - Stripes extend perpendicular to road direction (crossing the road)
//! - Centered on the centerline axis (left-right symmetric)
//! - 7 stripes = 3 left + 1 center + 3 right
//!
//! Default dimensions (mm):
//! - stripeLength: 4000 (crossing direction)
//! - stripeWidth: 450 (road direction)
//! - stripeSpacing: 450 (between stripes)

use dxf_engine::DxfLine;

/// Centerline path point
#[derive(Clone, Copy, Debug)]
pub struct PathPoint {
    pub x: f64,
    pub y: f64,
}

/// Crosswalk configuration
#[derive(Clone, Debug)]
pub struct CrosswalkConfig {
    pub start_offset: f64,     // Distance from centerline start (mm)
    pub stripe_length: f64,    // Length crossing the road (mm)
    pub stripe_width: f64,     // Width along road direction (mm)
    pub stripe_count: usize,   // Number of stripes
    pub stripe_spacing: f64,   // Gap between stripes (mm)
    pub layer: String,         // DXF layer name
}

impl Default for CrosswalkConfig {
    fn default() -> Self {
        Self {
            start_offset: 11000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 7,
            stripe_spacing: 450.0,
            layer: "横断歩道".to_string(),
        }
    }
}

/// Build a path from centerline DxfLine entities
pub fn build_centerline_path(lines: &[DxfLine]) -> Vec<PathPoint> {
    todo!("Implement: extract path points from centerline lines")
}

/// Get a point at a given distance along the path
pub fn point_at_distance(path: &[PathPoint], distance: f64) -> Option<PathPoint> {
    todo!("Implement: interpolate point at cumulative distance along path")
}

/// Generate crosswalk stripes from centerline
/// Returns DXF lines forming stripe rectangles (4 lines per stripe)
pub fn generate_crosswalk(centerlines: &[DxfLine], config: &CrosswalkConfig) -> Vec<DxfLine> {
    todo!("Implement: generate stripe rectangles along centerline")
}

/// Filter lines by layer name pattern (case-insensitive contains)
pub fn filter_by_layer(lines: &[DxfLine], pattern: &str) -> Vec<DxfLine> {
    todo!("Implement: filter lines where layer contains pattern")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxf_engine::DxfLine;

    fn horizontal_centerline() -> Vec<DxfLine> {
        vec![DxfLine::new(0.0, 0.0, 20000.0, 0.0)]
    }

    // ================================================================
    // Stripe count and structure
    // From Kotlin: "1本のストライプは4本の線で構成される"
    // ================================================================

    #[test]
    fn test_single_stripe_generates_4_lines() {
        let centerlines = vec![DxfLine::new(0.0, 0.0, 10000.0, 0.0)];
        let config = CrosswalkConfig {
            start_offset: 1000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 1,
            stripe_spacing: 0.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 4, "1 stripe = 4 lines (rectangle)");
    }

    #[test]
    fn test_seven_stripes_generates_28_lines() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 11000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 7,
            stripe_spacing: 450.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 28, "7 stripes × 4 lines = 28");
    }

    #[test]
    fn test_empty_centerlines_returns_empty() {
        let config = CrosswalkConfig::default();
        let result = generate_crosswalk(&[], &config);
        assert!(result.is_empty());
    }

    // ================================================================
    // Symmetric placement around centerline
    // From Kotlin: "ストライプはセンターラインを軸に左右対称配置"
    // ================================================================

    #[test]
    fn test_stripes_centered_on_centerline() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 7,
            stripe_spacing: 450.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);

        // Y coordinates should be symmetric around Y=0 (centerline)
        let all_y: Vec<f64> = result.iter()
            .flat_map(|l| vec![l.y1, l.y2])
            .collect();
        let min_y = all_y.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = all_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Stripes extend ±2000mm from centerline (stripe_length/2)
        assert!(min_y < 0.0 && max_y > 0.0,
            "Stripes must span both sides of centerline: min_y={}, max_y={}", min_y, max_y);
        assert!((min_y + max_y).abs() < 1.0,
            "Stripes must be symmetric: |min_y + max_y| = {}", (min_y + max_y).abs());
    }

    #[test]
    fn test_stripes_along_road_direction() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 7,
            stripe_spacing: 450.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);

        // X range should span the total crosswalk width
        let all_x: Vec<f64> = result.iter()
            .flat_map(|l| vec![l.x1, l.x2])
            .collect();
        let min_x = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_x = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Total road-direction width = 7*450 + 6*450 = 5850mm
        let total_width = 7.0 * 450.0 + 6.0 * 450.0;
        let half_width = total_width / 2.0;
        let center = 10000.0;

        assert!(min_x >= center - half_width - 250.0, "min_x={} too far left", min_x);
        assert!(max_x <= center + half_width + 250.0, "max_x={} too far right", max_x);
    }

    // ================================================================
    // Layer assignment
    // ================================================================

    #[test]
    fn test_all_stripes_have_correct_layer() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 5000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 3,
            stripe_spacing: 450.0,
            layer: "テスト層".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert!(result.iter().all(|l| l.layer == "テスト層"),
            "All lines must have the specified layer");
    }

    // ================================================================
    // Centerline path building
    // ================================================================

    #[test]
    fn test_build_path_from_lines() {
        let lines = vec![
            DxfLine::new(0.0, 0.0, 100.0, 0.0),
            DxfLine::new(100.0, 0.0, 200.0, 50.0),
        ];
        let path = build_centerline_path(&lines);
        assert_eq!(path.len(), 3); // start + end of each segment (deduplicated)
        assert!((path[0].x - 0.0).abs() < 0.001);
        assert!((path[1].x - 100.0).abs() < 0.001);
        assert!((path[2].x - 200.0).abs() < 0.001);
        assert!((path[2].y - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_point_at_distance_interpolation() {
        let path = vec![
            PathPoint { x: 0.0, y: 0.0 },
            PathPoint { x: 100.0, y: 0.0 },
            PathPoint { x: 200.0, y: 0.0 },
        ];
        let p = point_at_distance(&path, 50.0).unwrap();
        assert!((p.x - 50.0).abs() < 0.001);
        assert!((p.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_point_at_distance_second_segment() {
        let path = vec![
            PathPoint { x: 0.0, y: 0.0 },
            PathPoint { x: 100.0, y: 0.0 },
            PathPoint { x: 200.0, y: 100.0 },
        ];
        // Distance 150 = 100 (first segment) + 50 of second segment
        // Second segment length = sqrt(100^2 + 100^2) ≈ 141.42
        // ratio = 50 / 141.42 ≈ 0.3536
        let p = point_at_distance(&path, 150.0).unwrap();
        assert!(p.x > 100.0 && p.x < 200.0, "x should be on second segment: {}", p.x);
    }

    // ================================================================
    // Layer filtering
    // ================================================================

    #[test]
    fn test_filter_by_layer_case_insensitive() {
        let lines = vec![
            DxfLine::with_style(0.0, 0.0, 100.0, 0.0, 7, "中心線"),
            DxfLine::with_style(0.0, 0.0, 100.0, 0.0, 7, "中心-道路"),
            DxfLine::with_style(0.0, 0.0, 100.0, 0.0, 7, "ガードレール"),
        ];
        let filtered = filter_by_layer(&lines, "中心");
        assert_eq!(filtered.len(), 2);
    }
}
