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

/// Build a path from centerline DxfLine entities.
/// Deduplicates shared endpoints between consecutive segments.
pub fn build_centerline_path(lines: &[DxfLine]) -> Vec<PathPoint> {
    if lines.is_empty() {
        return vec![];
    }
    let mut path = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            path.push(PathPoint { x: line.x1, y: line.y1 });
        }
        path.push(PathPoint { x: line.x2, y: line.y2 });
    }
    path
}

/// Get a point at a given distance along the path via linear interpolation.
pub fn point_at_distance(path: &[PathPoint], distance: f64) -> Option<PathPoint> {
    if path.len() < 2 {
        return None;
    }
    let mut remaining = distance;
    for i in 0..path.len() - 1 {
        let dx = path[i + 1].x - path[i].x;
        let dy = path[i + 1].y - path[i].y;
        let seg_len = (dx * dx + dy * dy).sqrt();
        if remaining <= seg_len + 1e-9 {
            let ratio = if seg_len > 1e-12 { remaining / seg_len } else { 0.0 };
            return Some(PathPoint {
                x: path[i].x + dx * ratio,
                y: path[i].y + dy * ratio,
            });
        }
        remaining -= seg_len;
    }
    // Beyond path end — return last point
    path.last().copied()
}

/// Get road direction angle at a given distance along the path.
fn road_angle_at(path: &[PathPoint], distance: f64) -> f64 {
    if path.len() < 2 {
        return 0.0;
    }
    let mut remaining = distance;
    for i in 0..path.len() - 1 {
        let dx = path[i + 1].x - path[i].x;
        let dy = path[i + 1].y - path[i].y;
        let seg_len = (dx * dx + dy * dy).sqrt();
        if remaining <= seg_len + 1e-9 {
            return dy.atan2(dx);
        }
        remaining -= seg_len;
    }
    // Last segment direction
    let n = path.len();
    let dx = path[n - 1].x - path[n - 2].x;
    let dy = path[n - 1].y - path[n - 2].y;
    dy.atan2(dx)
}

/// Generate a single stripe rectangle centered at `center` with given dimensions and angles.
/// Returns 4 DxfLines forming the rectangle.
fn generate_stripe_rect(
    center: PathPoint,
    stripe_length: f64,
    stripe_width: f64,
    road_angle: f64,
    layer: &str,
) -> Vec<DxfLine> {
    let half_len = stripe_length / 2.0;
    let half_wid = stripe_width / 2.0;

    // Road direction unit vectors
    let cos_r = road_angle.cos();
    let sin_r = road_angle.sin();

    // Perpendicular direction (crossing the road)
    let cos_p = -sin_r;
    let sin_p = cos_r;

    // 4 corners: road_direction ± half_wid, perp_direction ± half_len
    let corners = [
        PathPoint {
            x: center.x + half_wid * cos_r + half_len * cos_p,
            y: center.y + half_wid * sin_r + half_len * sin_p,
        },
        PathPoint {
            x: center.x + half_wid * cos_r - half_len * cos_p,
            y: center.y + half_wid * sin_r - half_len * sin_p,
        },
        PathPoint {
            x: center.x - half_wid * cos_r - half_len * cos_p,
            y: center.y - half_wid * sin_r - half_len * sin_p,
        },
        PathPoint {
            x: center.x - half_wid * cos_r + half_len * cos_p,
            y: center.y - half_wid * sin_r + half_len * sin_p,
        },
    ];

    vec![
        DxfLine::with_style(corners[0].x, corners[0].y, corners[1].x, corners[1].y, 7, layer),
        DxfLine::with_style(corners[1].x, corners[1].y, corners[2].x, corners[2].y, 7, layer),
        DxfLine::with_style(corners[2].x, corners[2].y, corners[3].x, corners[3].y, 7, layer),
        DxfLine::with_style(corners[3].x, corners[3].y, corners[0].x, corners[0].y, 7, layer),
    ]
}

/// Generate crosswalk stripes from centerline.
/// Stripes are centered symmetrically around `start_offset` along the road direction.
/// Returns DXF lines forming stripe rectangles (4 lines per stripe).
pub fn generate_crosswalk(centerlines: &[DxfLine], config: &CrosswalkConfig) -> Vec<DxfLine> {
    if centerlines.is_empty() || config.stripe_count == 0 {
        return vec![];
    }

    let path = build_centerline_path(centerlines);
    if path.len() < 2 {
        return vec![];
    }

    let road_angle = road_angle_at(&path, config.start_offset);

    // Reference point on centerline at start_offset
    let ref_point = match point_at_distance(&path, config.start_offset) {
        Some(p) => p,
        None => return vec![],
    };

    // Total width of all stripes + gaps
    let n = config.stripe_count as f64;
    let total_width = n * config.stripe_width + (n - 1.0) * config.stripe_spacing;

    // Stripes are placed centered around ref_point along road direction
    let cos_r = road_angle.cos();
    let sin_r = road_angle.sin();

    let mut result = Vec::new();
    for i in 0..config.stripe_count {
        // Offset from center of the group
        let stripe_center_offset = (i as f64) * (config.stripe_width + config.stripe_spacing)
            + config.stripe_width / 2.0
            - total_width / 2.0;

        let center = PathPoint {
            x: ref_point.x + stripe_center_offset * cos_r,
            y: ref_point.y + stripe_center_offset * sin_r,
        };

        result.extend(generate_stripe_rect(
            center,
            config.stripe_length,
            config.stripe_width,
            road_angle,
            &config.layer,
        ));
    }

    result
}

/// Filter lines by layer name pattern (case-insensitive contains).
pub fn filter_by_layer(lines: &[DxfLine], pattern: &str) -> Vec<DxfLine> {
    let pattern_lower = pattern.to_lowercase();
    lines
        .iter()
        .filter(|l| l.layer.to_lowercase().contains(&pattern_lower))
        .cloned()
        .collect()
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

    // ================================================================
    // Zero stripes
    // ================================================================

    #[test]
    fn test_crosswalk_zero_stripes() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            stripe_count: 0,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert!(result.is_empty(), "0 stripes must produce 0 lines");
    }

    // ================================================================
    // 1 stripe — centered on ref point
    // ================================================================

    #[test]
    fn test_crosswalk_one_stripe_centered() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 5000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 1,
            stripe_spacing: 0.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 4, "1 stripe = 4 lines");

        // The stripe center should be at x=5000, y=0 (on centerline)
        let all_x: Vec<f64> = result.iter().flat_map(|l| vec![l.x1, l.x2]).collect();
        let avg_x: f64 = all_x.iter().sum::<f64>() / all_x.len() as f64;
        assert!((avg_x - 5000.0).abs() < 1.0,
            "Single stripe center X should be at offset: avg={}", avg_x);

        let all_y: Vec<f64> = result.iter().flat_map(|l| vec![l.y1, l.y2]).collect();
        let avg_y: f64 = all_y.iter().sum::<f64>() / all_y.len() as f64;
        assert!(avg_y.abs() < 1.0,
            "Single stripe should be centered on centerline Y: avg={}", avg_y);
    }

    // ================================================================
    // Even stripe count
    // ================================================================

    #[test]
    fn test_crosswalk_even_stripe_count() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_count: 4,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 16, "4 stripes × 4 lines = 16");

        // Still symmetric around centerline
        let all_y: Vec<f64> = result.iter().flat_map(|l| vec![l.y1, l.y2]).collect();
        let min_y = all_y.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = all_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!((min_y + max_y).abs() < 1.0,
            "Even stripe count must still be symmetric: min={}, max={}", min_y, max_y);
    }

    // ================================================================
    // Odd stripe count (non-default)
    // ================================================================

    #[test]
    fn test_crosswalk_odd_stripe_count_3() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_count: 3,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 12, "3 stripes × 4 lines = 12");
    }

    // ================================================================
    // Negative start_offset — returns last point (clamped)
    // ================================================================

    #[test]
    fn test_crosswalk_negative_offset() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: -1000.0,
            stripe_count: 3,
            ..CrosswalkConfig::default()
        };
        // point_at_distance with negative distance: remaining starts negative,
        // first segment check: remaining <= seg_len + 1e-9 → true (negative <= positive)
        // ratio will be negative/seg_len → negative ratio, point before path start
        let result = generate_crosswalk(&centerlines, &config);
        // Should still produce geometry (extrapolated before path start)
        assert_eq!(result.len(), 12, "Should still generate 3 stripes even with negative offset");
    }

    // ================================================================
    // Offset beyond centerline end — clamps to last point
    // ================================================================

    #[test]
    fn test_crosswalk_offset_beyond_centerline() {
        let centerlines = vec![DxfLine::new(0.0, 0.0, 5000.0, 0.0)];
        let config = CrosswalkConfig {
            start_offset: 10000.0, // far beyond the 5000mm centerline
            stripe_count: 3,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        // point_at_distance returns last point when beyond path end
        assert_eq!(result.len(), 12, "Should generate stripes at path end");

        // All stripes should cluster near x=5000 (the end of centerline)
        let all_x: Vec<f64> = result.iter().flat_map(|l| vec![l.x1, l.x2]).collect();
        let avg_x: f64 = all_x.iter().sum::<f64>() / all_x.len() as f64;
        assert!((avg_x - 5000.0).abs() < 5000.0,
            "Stripes should be near path end: avg_x={}", avg_x);
    }

    // ================================================================
    // Angled centerline: 45 degrees
    // ================================================================

    #[test]
    fn test_crosswalk_angled_45deg() {
        // 45 degree centerline: (0,0) → (10000, 10000)
        let centerlines = vec![DxfLine::new(0.0, 0.0, 10000.0, 10000.0)];
        let config = CrosswalkConfig {
            start_offset: 5000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 1,
            stripe_spacing: 0.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 4, "1 stripe = 4 lines on angled centerline");

        // The stripe should be perpendicular to the 45° road direction
        // Road angle = 45° → stripes extend at 135° (perpendicular)
        // Check that the stripe is not axis-aligned
        let dx = (result[0].x2 - result[0].x1).abs();
        let dy = (result[0].y2 - result[0].y1).abs();
        assert!(dx > 1.0 && dy > 1.0,
            "On 45° road, stripe edges should have both X and Y components: dx={}, dy={}", dx, dy);
    }

    // ================================================================
    // Angled centerline: 90 degrees (vertical road)
    // ================================================================

    #[test]
    fn test_crosswalk_angled_90deg() {
        // Vertical centerline: (0,0) → (0, 20000)
        let centerlines = vec![DxfLine::new(0.0, 0.0, 0.0, 20000.0)];
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 3,
            stripe_spacing: 450.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 12, "3 stripes on vertical road");

        // On a vertical road, stripes should extend in X direction (perpendicular)
        let all_x: Vec<f64> = result.iter().flat_map(|l| vec![l.x1, l.x2]).collect();
        let min_x = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_x = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(min_x < 0.0 && max_x > 0.0,
            "On vertical road, stripes should span both sides of X=0: min={}, max={}", min_x, max_x);
    }

    // ================================================================
    // Multi-segment centerline path
    // ================================================================

    #[test]
    fn test_crosswalk_multi_segment_centerline() {
        // L-shaped path: horizontal then vertical
        let centerlines = vec![
            DxfLine::new(0.0, 0.0, 10000.0, 0.0),
            DxfLine::new(10000.0, 0.0, 10000.0, 10000.0),
        ];
        let config = CrosswalkConfig {
            start_offset: 15000.0, // 10000 horizontal + 5000 vertical
            stripe_count: 1,
            stripe_length: 2000.0,
            stripe_width: 450.0,
            stripe_spacing: 0.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 4);

        // At distance 15000, we're on the vertical segment at (10000, 5000)
        let all_x: Vec<f64> = result.iter().flat_map(|l| vec![l.x1, l.x2]).collect();
        let avg_x: f64 = all_x.iter().sum::<f64>() / all_x.len() as f64;
        assert!((avg_x - 10000.0).abs() < 500.0,
            "Stripe should be near x=10000 on vertical segment: avg={}", avg_x);
    }

    // ================================================================
    // build_centerline_path edge cases
    // ================================================================

    #[test]
    fn test_build_path_empty() {
        let path = build_centerline_path(&[]);
        assert!(path.is_empty());
    }

    #[test]
    fn test_build_path_single_line() {
        let lines = vec![DxfLine::new(10.0, 20.0, 30.0, 40.0)];
        let path = build_centerline_path(&lines);
        assert_eq!(path.len(), 2);
        assert!((path[0].x - 10.0).abs() < 1e-9);
        assert!((path[0].y - 20.0).abs() < 1e-9);
        assert!((path[1].x - 30.0).abs() < 1e-9);
        assert!((path[1].y - 40.0).abs() < 1e-9);
    }

    // ================================================================
    // point_at_distance edge cases
    // ================================================================

    #[test]
    fn test_point_at_distance_single_point() {
        let path = vec![PathPoint { x: 5.0, y: 5.0 }];
        assert!(point_at_distance(&path, 0.0).is_none(), "< 2 points returns None");
    }

    #[test]
    fn test_point_at_distance_empty() {
        assert!(point_at_distance(&[], 0.0).is_none());
    }

    #[test]
    fn test_point_at_distance_zero() {
        let path = vec![
            PathPoint { x: 100.0, y: 200.0 },
            PathPoint { x: 300.0, y: 200.0 },
        ];
        let p = point_at_distance(&path, 0.0).unwrap();
        assert!((p.x - 100.0).abs() < 1e-9);
        assert!((p.y - 200.0).abs() < 1e-9);
    }

    #[test]
    fn test_point_at_distance_exact_end() {
        let path = vec![
            PathPoint { x: 0.0, y: 0.0 },
            PathPoint { x: 100.0, y: 0.0 },
        ];
        let p = point_at_distance(&path, 100.0).unwrap();
        assert!((p.x - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_point_at_distance_beyond_end() {
        let path = vec![
            PathPoint { x: 0.0, y: 0.0 },
            PathPoint { x: 100.0, y: 0.0 },
        ];
        let p = point_at_distance(&path, 500.0).unwrap();
        // Returns last point
        assert!((p.x - 100.0).abs() < 1e-6);
    }

    // ================================================================
    // filter_by_layer edge cases
    // ================================================================

    #[test]
    fn test_filter_by_layer_empty_lines() {
        let filtered = filter_by_layer(&[], "中心");
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_by_layer_no_match() {
        let lines = vec![DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "外壁")];
        let filtered = filter_by_layer(&lines, "中心");
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_by_layer_empty_pattern() {
        let lines = vec![
            DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "中心線"),
            DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, ""),
        ];
        // Empty pattern matches everything (every string contains "")
        let filtered = filter_by_layer(&lines, "");
        assert_eq!(filtered.len(), 2);
    }

    // ================================================================
    // Stripe dimensions accuracy
    // ================================================================

    #[test]
    fn test_stripe_dimensions_correct() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 1,
            stripe_spacing: 0.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 4);

        // On horizontal road: stripe extends in Y (perpendicular), width in X (road dir)
        // Line 0: top edge (y1=+2000, y2=-2000, x constant at x_center + half_width)
        // Measure Y extent = stripe_length = 4000
        let all_y: Vec<f64> = result.iter().flat_map(|l| vec![l.y1, l.y2]).collect();
        let min_y = all_y.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = all_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let y_extent = max_y - min_y;
        assert!((y_extent - 4000.0).abs() < 1.0,
            "Y extent should equal stripe_length 4000: got {}", y_extent);

        // Measure X extent = stripe_width = 450
        let all_x: Vec<f64> = result.iter().flat_map(|l| vec![l.x1, l.x2]).collect();
        let min_x = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_x = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let x_extent = max_x - min_x;
        assert!((x_extent - 450.0).abs() < 1.0,
            "X extent should equal stripe_width 450: got {}", x_extent);
    }

    // ================================================================
    // Default config values
    // ================================================================

    #[test]
    fn test_crosswalk_config_default() {
        let config = CrosswalkConfig::default();
        assert_eq!(config.start_offset, 11000.0);
        assert_eq!(config.stripe_length, 4000.0);
        assert_eq!(config.stripe_width, 450.0);
        assert_eq!(config.stripe_count, 7);
        assert_eq!(config.stripe_spacing, 450.0);
        assert_eq!(config.layer, "横断歩道");
    }

    // ================================================================
    // Zero-length centerline (degenerate)
    // ================================================================

    #[test]
    fn test_crosswalk_zero_length_centerline() {
        let centerlines = vec![DxfLine::new(5000.0, 5000.0, 5000.0, 5000.0)];
        let config = CrosswalkConfig {
            start_offset: 0.0,
            stripe_count: 1,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        // Path has 2 points but segment length is 0 → road_angle = atan2(0,0) = 0
        // Should still generate geometry
        assert_eq!(result.len(), 4);
    }

    // ================================================================
    // Large stripe count
    // ================================================================

    #[test]
    fn test_crosswalk_large_stripe_count() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_count: 100,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 400, "100 stripes × 4 lines = 400");
    }

    // ================================================================
    // Angled centerline: 30 degrees
    // ================================================================

    #[test]
    fn test_crosswalk_angled_30deg() {
        // 30° road: tan(30°)=1/√3, so rise/run ≈ 0.577
        let run = 10000.0_f64;
        let rise = run * (std::f64::consts::PI / 6.0).tan(); // ≈ 5773.5
        let centerlines = vec![DxfLine::new(0.0, 0.0, run, rise)];
        let config = CrosswalkConfig {
            start_offset: 5000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 3,
            stripe_spacing: 450.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 12, "3 stripes × 4 lines on 30° road");

        // Stripes should be perpendicular to 30° direction (i.e. at 120°)
        // Verify they span both sides of the centerline path
        let ref_pt = point_at_distance(
            &build_centerline_path(&centerlines), 5000.0,
        ).unwrap();

        // Check that stripe corners are distributed around the ref point
        let all_x: Vec<f64> = result.iter().flat_map(|l| vec![l.x1, l.x2]).collect();
        let all_y: Vec<f64> = result.iter().flat_map(|l| vec![l.y1, l.y2]).collect();
        let min_x = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_x = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_y = all_y.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = all_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // The stripe group should straddle the reference point
        assert!(min_x < ref_pt.x && max_x > ref_pt.x,
            "Stripes should straddle ref X={}: min={}, max={}", ref_pt.x, min_x, max_x);
        assert!(min_y < ref_pt.y && max_y > ref_pt.y,
            "Stripes should straddle ref Y={}: min={}, max={}", ref_pt.y, min_y, max_y);
    }

    // ================================================================
    // Angled centerline: 60 degrees
    // ================================================================

    #[test]
    fn test_crosswalk_angled_60deg() {
        let run = 10000.0_f64;
        let rise = run * (std::f64::consts::PI / 3.0).tan(); // ≈ 17320.5
        let centerlines = vec![DxfLine::new(0.0, 0.0, run, rise)];
        let config = CrosswalkConfig {
            start_offset: 5000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 3,
            stripe_spacing: 450.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 12, "3 stripes × 4 lines on 60° road");

        // Perpendicular to 60° road = 150° direction
        // On a steep road, stripes extend more in X than Y
        let ref_pt = point_at_distance(
            &build_centerline_path(&centerlines), 5000.0,
        ).unwrap();

        // Stripe length = 4000, so perpendicular extent = ±2000 from center
        // At 60° road, perpendicular is at -30° from X axis
        // Perpendicular X component = cos(150°) ≈ -0.866, so X extent ≈ 2*2000*0.866 = 3464
        let all_x: Vec<f64> = result.iter().flat_map(|l| vec![l.x1, l.x2]).collect();
        let x_extent = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            - all_x.iter().cloned().fold(f64::INFINITY, f64::min);
        // X extent should be significant (not near-zero like it would be for a horizontal road)
        assert!(x_extent > 3000.0,
            "60° road stripes should have large X extent: got {}", x_extent);
    }

    // ================================================================
    // Angled centerline: exact 90 degrees — stripe symmetry check
    // ================================================================

    #[test]
    fn test_crosswalk_angled_90deg_symmetry() {
        // Vertical road (0,0)→(0,20000)
        let centerlines = vec![DxfLine::new(0.0, 0.0, 0.0, 20000.0)];
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_length: 4000.0,
            stripe_width: 450.0,
            stripe_count: 7,
            stripe_spacing: 450.0,
            layer: "横断歩道".to_string(),
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 28, "7 stripes on vertical road");

        // On vertical road: stripes extend in X (perpendicular), spread in Y (road dir)
        // X should be symmetric around 0
        let all_x: Vec<f64> = result.iter().flat_map(|l| vec![l.x1, l.x2]).collect();
        let min_x = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_x = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!((min_x + max_x).abs() < 1.0,
            "Stripes on vertical road must be X-symmetric: min={}, max={}", min_x, max_x);

        // Y should be symmetric around offset=10000
        let all_y: Vec<f64> = result.iter().flat_map(|l| vec![l.y1, l.y2]).collect();
        let avg_y: f64 = all_y.iter().sum::<f64>() / all_y.len() as f64;
        assert!((avg_y - 10000.0).abs() < 1.0,
            "Stripes should center around Y=10000: avg={}", avg_y);
    }

    // ================================================================
    // Generate + DxfLinter roundtrip
    // ================================================================

    #[test]
    fn test_crosswalk_generate_lint_roundtrip() {
        use dxf_engine::{DxfWriter, DxfLinter};

        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig::default();
        let lines = generate_crosswalk(&centerlines, &config);
        assert!(!lines.is_empty());

        // Write to DXF string and validate with linter
        let writer = DxfWriter::new();
        let dxf_content = writer.write(&lines, &[]);
        assert!(DxfLinter::is_valid(&dxf_content),
            "Generated crosswalk DXF must pass linter validation");
    }

    #[test]
    fn test_crosswalk_angled_generate_lint_roundtrip() {
        use dxf_engine::{DxfWriter, DxfLinter};

        // 30° road
        let run = 10000.0_f64;
        let rise = run * (std::f64::consts::PI / 6.0).tan();
        let centerlines = vec![DxfLine::new(0.0, 0.0, run, rise)];
        let config = CrosswalkConfig {
            start_offset: 3000.0,
            stripe_count: 5,
            ..CrosswalkConfig::default()
        };
        let lines = generate_crosswalk(&centerlines, &config);

        let writer = DxfWriter::new();
        let dxf_content = writer.write(&lines, &[]);
        assert!(DxfLinter::is_valid(&dxf_content),
            "Angled crosswalk DXF must pass linter validation");
    }

    #[test]
    fn test_crosswalk_vertical_generate_lint_roundtrip() {
        use dxf_engine::{DxfWriter, DxfLinter};

        let centerlines = vec![DxfLine::new(0.0, 0.0, 0.0, 20000.0)];
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_count: 3,
            ..CrosswalkConfig::default()
        };
        let lines = generate_crosswalk(&centerlines, &config);

        let writer = DxfWriter::new();
        let dxf_content = writer.write(&lines, &[]);
        assert!(DxfLinter::is_valid(&dxf_content),
            "Vertical road crosswalk DXF must pass linter validation");
    }

    // ================================================================
    // Zero/extreme stripe parameter edge cases
    // ================================================================

    #[test]
    fn test_crosswalk_zero_stripe_width() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_width: 0.0,
            stripe_count: 3,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        // Zero-width stripes should still generate 3×4=12 lines (degenerate rectangles)
        assert_eq!(result.len(), 12, "zero-width stripes should still generate geometry");
    }

    #[test]
    fn test_crosswalk_zero_stripe_spacing() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_spacing: 0.0,
            stripe_count: 3,
            stripe_width: 450.0,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 12, "3 stripes × 4 lines = 12 with zero spacing");
    }

    #[test]
    fn test_crosswalk_zero_stripe_length() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_length: 0.0,
            stripe_count: 1,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 4, "zero-length stripe should still generate 4 degenerate lines");
    }

    #[test]
    fn test_crosswalk_negative_stripe_length() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_length: -4000.0,
            stripe_count: 1,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        // Negative length inverts corners but still produces valid geometry
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_crosswalk_very_large_stripe_count_1000() {
        let centerlines = horizontal_centerline();
        let config = CrosswalkConfig {
            start_offset: 10000.0,
            stripe_count: 1000,
            stripe_width: 10.0,
            stripe_spacing: 5.0,
            ..CrosswalkConfig::default()
        };
        let result = generate_crosswalk(&centerlines, &config);
        assert_eq!(result.len(), 4000, "1000 stripes × 4 lines = 4000");
    }

    #[test]
    fn test_point_at_distance_negative() {
        let path = vec![
            PathPoint { x: 0.0, y: 0.0 },
            PathPoint { x: 100.0, y: 0.0 },
        ];
        let pt = point_at_distance(&path, -10.0);
        assert!(pt.is_some(), "negative distance should return a point (fallback)");
    }

    #[test]
    fn test_build_centerline_path_single_line() {
        let lines = vec![DxfLine::new(0.0, 0.0, 100.0, 0.0)];
        let path = build_centerline_path(&lines);
        assert_eq!(path.len(), 2);
        assert!((path[0].x - 0.0).abs() < 0.001);
        assert!((path[1].x - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_filter_by_layer_empty_pattern() {
        let lines = vec![
            DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "中心線"),
            DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "路肩"),
        ];
        let filtered = filter_by_layer(&lines, "");
        // Empty pattern matches everything (contains "")
        assert_eq!(filtered.len(), 2);
    }
}
