//! Triangle geometry: area, vertices, angles
//!
//! Spec (from trianglelist CLAUDE.md):
//! - A edge: connection edge (shared with parent)
//! - B edge: free edge
//! - C edge: free edge
//! - Vertices: point[0]=CA (origin), pointAB (x-axis at dist=c), pointBC (cosine rule)
//! - Area: Heron's formula, round to 2 decimal places

/// 2D point
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Triangle with 3 side lengths and computed vertices
#[derive(Clone, Debug)]
pub struct Triangle {
    /// Side lengths: [A, B, C]
    pub lengths: [f64; 3],
    /// Vertices: [CA, AB, BC]
    pub points: [Point; 3],
    /// Base rotation angle in degrees
    pub angle: f64,
    /// Parent triangle number (-1 = independent)
    pub parent_number: i32,
    /// Connection type: -1=none, 1=parent's B edge, 2=parent's C edge
    pub connection_type: i32,
}

impl Triangle {
    /// Create an independent triangle at origin with given base angle
    /// Side A = lengths[0], B = lengths[1], C = lengths[2]
    /// Point CA at origin, Point AB at distance C along base angle direction
    /// Point BC computed via cosine rule
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        Self::with_angle(a, b, c, Point::new(0.0, 0.0), 180.0)
    }

    /// Create triangle with explicit base point and angle
    pub fn with_angle(a: f64, b: f64, c: f64, origin: Point, angle: f64) -> Self {
        let rad = angle.to_radians();

        // CA at origin
        let point_ca = origin;

        // AB at distance A from CA along angle direction
        let point_ab = Point::new(
            origin.x + a * rad.cos(),
            origin.y + a * rad.sin(),
        );

        // BC computed via cosine rule
        // In triangle: CA→AB = A, AB→BC = B, BC→CA = C
        // Angle at CA between sides A and C:
        //   cos(angle_at_CA) = (A² + C² - B²) / (2·A·C)
        let cos_angle_at_ca = (a * a + c * c - b * b) / (2.0 * a * c);
        let angle_at_ca = cos_angle_at_ca.clamp(-1.0, 1.0).acos();

        // BC is at distance C from CA, rotated by angle_at_ca from the CA→AB direction
        let bc_angle = rad + angle_at_ca;
        let point_bc = Point::new(
            origin.x + c * bc_angle.cos(),
            origin.y + c * bc_angle.sin(),
        );

        Self {
            lengths: [a, b, c],
            points: [point_ca, point_ab, point_bc],
            angle,
            parent_number: -1,
            connection_type: -1,
        }
    }

    /// Check if side lengths form a valid triangle (triangle inequality)
    pub fn is_valid(&self) -> bool {
        let [a, b, c] = self.lengths;
        a + b > c && b + c > a && c + a > b
    }

    /// Calculate area using Heron's formula, rounded to 2 decimal places
    pub fn area(&self) -> f64 {
        let [a, b, c] = self.lengths;
        let s = (a + b + c) / 2.0;
        let area = (s * (s - a) * (s - b) * (s - c)).sqrt();
        (area * 100.0).round() / 100.0
    }

    /// Internal angle at vertex CA (opposite to side A) in degrees
    /// Opposite side A: between sides C and... wait.
    /// angle_a = angle opposite to side A
    /// Side A connects CA→AB. Opposite vertex is BC.
    /// So angle at BC, between sides B and C:
    ///   cos(angle) = (B² + C² - A²) / (2·B·C)
    pub fn angle_a(&self) -> f64 {
        let [a, b, c] = self.lengths;
        let cos_val = (b * b + c * c - a * a) / (2.0 * b * c);
        cos_val.clamp(-1.0, 1.0).acos().to_degrees()
    }

    /// Internal angle at vertex AB (opposite to side B) in degrees
    /// Side B connects AB→BC. Opposite vertex is CA.
    ///   cos(angle) = (A² + C² - B²) / (2·A·C)
    pub fn angle_b(&self) -> f64 {
        let [a, b, c] = self.lengths;
        let cos_val = (a * a + c * c - b * b) / (2.0 * a * c);
        cos_val.clamp(-1.0, 1.0).acos().to_degrees()
    }

    /// Internal angle at vertex BC (opposite to side C) in degrees
    /// Side C connects BC→CA. Opposite vertex is AB.
    ///   cos(angle) = (A² + B² - C²) / (2·A·B)
    pub fn angle_c(&self) -> f64 {
        let [a, b, c] = self.lengths;
        let cos_val = (a * a + b * b - c * c) / (2.0 * a * b);
        cos_val.clamp(-1.0, 1.0).acos().to_degrees()
    }

    /// Point CA (vertex 0, origin)
    pub fn point_ca(&self) -> &Point {
        &self.points[0]
    }

    /// Point AB (vertex 1, on x-axis)
    pub fn point_ab(&self) -> &Point {
        &self.points[1]
    }

    /// Point BC (vertex 2, computed)
    pub fn point_bc(&self) -> &Point {
        &self.points[2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    // Area calculation (Heron's formula)
    // Expected values from Kotlin TriangleTest.testGetArea()
    // ================================================================

    #[test]
    fn test_area_heron_basic() {
        // Kotlin: Triangle(5.71f, 9.1f, 6.59f) → area = 18.74
        let t = Triangle::new(5.71, 9.1, 6.59);
        assert!((t.area() - 18.74).abs() < 0.01);
    }

    #[test]
    fn test_area_right_triangle() {
        // 3-4-5 right triangle → area = 6.0
        let t = Triangle::new(3.0, 4.0, 5.0);
        assert!((t.area() - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_area_equilateral() {
        // Equilateral 5-5-5 → area = sqrt(3)/4 * 25 ≈ 10.83
        let t = Triangle::new(5.0, 5.0, 5.0);
        assert!((t.area() - 10.83).abs() < 0.01);
    }

    #[test]
    fn test_area_rounded_to_2_decimals() {
        // Area must be rounded to exactly 2 decimal places
        let t = Triangle::new(3.0, 4.0, 5.0);
        let area_str = format!("{:.2}", t.area());
        assert_eq!(area_str, "6.00");
    }

    // ================================================================
    // Triangle validity (triangle inequality)
    // ================================================================

    #[test]
    fn test_valid_triangle() {
        let t = Triangle::new(3.0, 4.0, 5.0);
        assert!(t.is_valid());
    }

    #[test]
    fn test_invalid_triangle_degenerate() {
        // a = b + c → degenerate (not a valid triangle)
        // This should still create but is_valid() returns false
        let t = Triangle {
            lengths: [10.0, 3.0, 7.0],
            points: [Point::new(0.0, 0.0); 3],
            angle: 180.0,
            parent_number: -1,
            connection_type: -1,
        };
        assert!(!t.is_valid());
    }

    // ================================================================
    // Vertex placement from side lengths
    // Expected values from Kotlin TriangleTest.testTrianglePoint()
    // Triangle(3,4,5) at origin with angle=180:
    //   point[0] (CA) = (0, 0)
    //   pointAB = (-3, 0)  [note: angle=180 flips along x-axis]
    // ================================================================

    #[test]
    fn test_vertex_placement_345_angle180() {
        // Kotlin: Triangle(3.0f, 4.0f, 5.0f, PointXY(0,0), 180.0f)
        //   point[0] = (0, 0)
        //   pointAB = (-3, 0)
        let t = Triangle::with_angle(3.0, 4.0, 5.0, Point::new(0.0, 0.0), 180.0);
        assert!((t.point_ca().x - 0.0).abs() < 0.001);
        assert!((t.point_ca().y - 0.0).abs() < 0.001);
        assert!((t.point_ab().x - (-3.0)).abs() < 0.001);
        assert!((t.point_ab().y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_vertex_bc_position() {
        // For Triangle(3,4,5) at origin with angle=180:
        // A=3 (CA→AB), B=4, C=5
        // CA at origin, AB at (-3, 0)
        // BC computed by cosine rule
        // In the Kotlin convention: lengths = [A, B, C]
        //   side C connects CA to AB (length = C = 5... wait)
        //
        // Actually Kotlin: length[0]=A=side opposite BC or the connection side
        // Let's verify: Triangle(a=3, b=4, c=5)
        //   pointCA at origin, pointAB at distance C=5 along angle
        //   ... but Kotlin test shows pointAB.x = -3.0 for angle=180
        //   So it seems C (lengths[2]) = distance from CA to AB? No.
        //   From Kotlin code: length_c = CA→AB distance
        //   So with angle=180: pointAB = origin + (-c, 0) = (-5, 0)?
        //   But test says -3.0...
        //
        // Reading more carefully: Triangle(3f, 4f, 5f) means
        //   length[0]=3=A, length[1]=4=B, length[2]=5=C
        //   Kotlin: point[1] (AB) is at distance lengths[0] (=A=3) from point[0]
        //   So AB = CA + A*direction = (0,0) + 3*(cos180, sin180) = (-3, 0) ✓
        //
        // So: CA→AB distance = lengths[0] = A (NOT C)
        // And BC is computed from A, B, C using cosine rule
        let t = Triangle::with_angle(3.0, 4.0, 5.0, Point::new(0.0, 0.0), 180.0);

        // CA→AB = 3.0 (side A), along angle 180° → AB = (-3, 0)
        assert!((t.point_ab().x - (-3.0)).abs() < 0.001);

        // BC should be computable and form valid triangle
        let ca_ab = t.point_ca().distance_to(t.point_ab());
        let ab_bc = t.point_ab().distance_to(t.point_bc());
        let bc_ca = t.point_bc().distance_to(t.point_ca());

        // Verify side lengths match (within float precision)
        // CA→AB = A = 3
        assert!((ca_ab - 3.0).abs() < 0.01, "CA→AB should be 3.0, got {}", ca_ab);
        // AB→BC = B = 4
        assert!((ab_bc - 4.0).abs() < 0.01, "AB→BC should be 4.0, got {}", ab_bc);
        // BC→CA = C = 5
        assert!((bc_ca - 5.0).abs() < 0.01, "BC→CA should be 5.0, got {}", bc_ca);
    }

    #[test]
    fn test_equilateral_vertices() {
        // Equilateral triangle 5-5-5 should have all sides equal
        let t = Triangle::new(5.0, 5.0, 5.0);
        let ca_ab = t.point_ca().distance_to(t.point_ab());
        let ab_bc = t.point_ab().distance_to(t.point_bc());
        let bc_ca = t.point_bc().distance_to(t.point_ca());

        assert!((ca_ab - 5.0).abs() < 0.01);
        assert!((ab_bc - 5.0).abs() < 0.01);
        assert!((bc_ca - 5.0).abs() < 0.01);
    }

    // ================================================================
    // Angle calculations
    // ================================================================

    #[test]
    fn test_angles_equilateral() {
        // Equilateral: all angles = 60°
        let t = Triangle::new(5.0, 5.0, 5.0);
        assert!((t.angle_a() - 60.0).abs() < 0.01);
        assert!((t.angle_b() - 60.0).abs() < 0.01);
        assert!((t.angle_c() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_angles_right_triangle() {
        // 3-4-5: angle opposite to C=5 (hypotenuse) should be 90°
        // A=3, B=4, C=5
        // angle_a = opposite to A=3 → ~36.87°
        // angle_b = opposite to B=4 → ~53.13°
        // angle_c = opposite to C=5 → 90°
        let t = Triangle::new(3.0, 4.0, 5.0);
        assert!((t.angle_a() - 36.87).abs() < 0.01);
        assert!((t.angle_b() - 53.13).abs() < 0.01);
        assert!((t.angle_c() - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_angles_sum_180() {
        let t = Triangle::new(5.71, 9.1, 6.59);
        let sum = t.angle_a() + t.angle_b() + t.angle_c();
        assert!((sum - 180.0).abs() < 0.01, "Angle sum should be 180, got {}", sum);
    }

    // ================================================================
    // Point
    // ================================================================

    #[test]
    fn test_point_distance_to_same() {
        let p = Point::new(3.0, 4.0);
        assert!((p.distance_to(&p) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_distance_to_origin() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_distance_symmetric() {
        let a = Point::new(1.0, 2.0);
        let b = Point::new(4.0, 6.0);
        assert!((a.distance_to(&b) - b.distance_to(&a)).abs() < 1e-10);
    }

    #[test]
    fn test_point_distance_negative_coords() {
        let a = Point::new(-3.0, -4.0);
        let b = Point::new(0.0, 0.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-10);
    }

    // ================================================================
    // Degenerate and invalid triangles
    // ================================================================

    #[test]
    fn test_degenerate_collinear_triangle() {
        // a + c == b → collinear, zero area
        let t = Triangle {
            lengths: [3.0, 7.0, 4.0],
            points: [Point::new(0.0, 0.0); 3],
            angle: 180.0,
            parent_number: -1,
            connection_type: -1,
        };
        assert!(!t.is_valid());
        assert!((t.area() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_degenerate_collinear_a_plus_b_eq_c() {
        // a + b == c exactly → collinear, zero area, invalid triangle
        let t = Triangle {
            lengths: [2.0, 3.0, 5.0],
            points: [Point::new(0.0, 0.0); 3],
            angle: 180.0,
            parent_number: -1,
            connection_type: -1,
        };
        assert!(!t.is_valid()); // 2+3 > 5 is false
        assert!((t.area() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_degenerate_collinear_b_plus_c_eq_a() {
        // b + c == a exactly
        let t = Triangle {
            lengths: [9.0, 4.0, 5.0],
            points: [Point::new(0.0, 0.0); 3],
            angle: 180.0,
            parent_number: -1,
            connection_type: -1,
        };
        assert!(!t.is_valid()); // 4+5 > 9 is false
        assert!((t.area() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_degenerate_collinear_constructed() {
        // Construct via with_angle: a+b==c → cosine rule yields degenerate vertex
        let t = Triangle::with_angle(2.0, 3.0, 5.0, Point::new(0.0, 0.0), 0.0);
        assert!(!t.is_valid());
        // Heron: s=5, s-a=3, s-b=2, s-c=0 → area=0
        assert!((t.area() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_degenerate_near_zero_sides() {
        // All sides = 0.001 → valid but near-zero area
        let t = Triangle::new(0.001, 0.001, 0.001);
        assert!(t.is_valid());
        // area = sqrt(3)/4 * 0.001^2 ≈ 4.33e-7 → rounds to 0.00
        assert!((t.area() - 0.0).abs() < 0.01);
        // Angles should still be 60° (equilateral)
        assert!((t.angle_a() - 60.0).abs() < 0.01);
        assert!((t.angle_b() - 60.0).abs() < 0.01);
        assert!((t.angle_c() - 60.0).abs() < 0.01);
        // Vertex distances should match side lengths
        let ca_ab = t.point_ca().distance_to(t.point_ab());
        let ab_bc = t.point_ab().distance_to(t.point_bc());
        let bc_ca = t.point_bc().distance_to(t.point_ca());
        assert!((ca_ab - 0.001).abs() < 1e-6, "CA→AB: {}", ca_ab);
        assert!((ab_bc - 0.001).abs() < 1e-6, "AB→BC: {}", ab_bc);
        assert!((bc_ca - 0.001).abs() < 1e-6, "BC→CA: {}", bc_ca);
    }

    #[test]
    fn test_degenerate_zero_side_a() {
        let t = Triangle {
            lengths: [0.0, 3.0, 3.0],
            points: [Point::new(0.0, 0.0); 3],
            angle: 180.0,
            parent_number: -1,
            connection_type: -1,
        };
        // 0 + 3 > 3 is false → invalid
        assert!(!t.is_valid());
    }

    #[test]
    fn test_degenerate_zero_all_sides() {
        let t = Triangle {
            lengths: [0.0, 0.0, 0.0],
            points: [Point::new(0.0, 0.0); 3],
            angle: 180.0,
            parent_number: -1,
            connection_type: -1,
        };
        assert!(!t.is_valid());
        assert!(t.area().is_nan() || t.area() == 0.0);
    }

    #[test]
    fn test_invalid_triangle_inequality_violation() {
        // a > b + c
        let t = Triangle {
            lengths: [100.0, 1.0, 1.0],
            points: [Point::new(0.0, 0.0); 3],
            angle: 180.0,
            parent_number: -1,
            connection_type: -1,
        };
        assert!(!t.is_valid());
    }

    #[test]
    fn test_invalid_negative_side() {
        let t = Triangle {
            lengths: [-3.0, 4.0, 5.0],
            points: [Point::new(0.0, 0.0); 3],
            angle: 180.0,
            parent_number: -1,
            connection_type: -1,
        };
        // Negative side → can't form valid triangle
        assert!(!t.is_valid());
    }

    // ================================================================
    // Area edge cases
    // ================================================================

    #[test]
    fn test_area_very_small_triangle() {
        // Micro triangle: sides 0.01, 0.01, 0.01
        let t = Triangle::new(0.01, 0.01, 0.01);
        assert!(t.is_valid());
        // area = sqrt(3)/4 * 0.01^2 ≈ 0.0000433
        // Rounded to 2 decimal → 0.00
        assert!((t.area() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_area_very_large_triangle() {
        let t = Triangle::new(1000.0, 1000.0, 1000.0);
        assert!(t.is_valid());
        // area = sqrt(3)/4 * 1e6 ≈ 433012.70
        let expected = (3.0_f64).sqrt() / 4.0 * 1_000_000.0;
        assert!((t.area() - (expected * 100.0).round() / 100.0).abs() < 0.01);
    }

    #[test]
    fn test_area_isosceles() {
        // Isosceles: 5, 5, 6
        let t = Triangle::new(5.0, 5.0, 6.0);
        assert!(t.is_valid());
        // height = sqrt(5^2 - 3^2) = 4, area = 0.5 * 6 * 4 = 12.0
        // But heron: s=8, sqrt(8*3*3*2)=sqrt(144)=12.0
        assert!((t.area() - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_area_obtuse_triangle() {
        // Obtuse: 2, 3, 4 (angle opposite 4 > 90°)
        let t = Triangle::new(2.0, 3.0, 4.0);
        assert!(t.is_valid());
        let s: f64 = (2.0 + 3.0 + 4.0) / 2.0;
        let expected = (s * (s - 2.0) * (s - 3.0) * (s - 4.0)).sqrt();
        assert!((t.area() - (expected * 100.0).round() / 100.0).abs() < 0.01);
    }

    // ================================================================
    // Angle edge cases
    // ================================================================

    #[test]
    fn test_angle_isosceles() {
        // Isosceles 5, 5, 6: two angles equal
        let t = Triangle::new(5.0, 5.0, 6.0);
        // Sides A=5, B=5 are equal → angle_a == angle_b (opposite equal sides)
        assert!((t.angle_a() - t.angle_b()).abs() < 0.01);
    }

    #[test]
    fn test_angle_obtuse() {
        // 2, 3, 4: angle opposite to C=4 should be > 90°
        let t = Triangle::new(2.0, 3.0, 4.0);
        assert!(t.angle_c() > 90.0, "Obtuse angle should be > 90°, got {}", t.angle_c());
    }

    #[test]
    fn test_angle_near_flat() {
        // Nearly flat: a=1, b=100, c=100 → angle_a ≈ 0.57°
        let t = Triangle::new(1.0, 100.0, 100.0);
        assert!(t.angle_a() < 1.0, "Very narrow angle, got {}", t.angle_a());
    }

    #[test]
    fn test_angles_sum_180_various() {
        let cases = [
            (3.0, 4.0, 5.0),
            (10.0, 10.0, 10.0),
            (1.0, 1.0, 1.5),
            (7.0, 8.0, 9.0),
            (0.5, 0.6, 0.7),
        ];
        for (a, b, c) in cases {
            let t = Triangle::new(a, b, c);
            let sum = t.angle_a() + t.angle_b() + t.angle_c();
            assert!((sum - 180.0).abs() < 0.01,
                "Triangle({},{},{}): angle sum = {}", a, b, c, sum);
        }
    }

    // ================================================================
    // Vertex placement edge cases
    // ================================================================

    #[test]
    fn test_vertex_placement_angle_0() {
        let t = Triangle::with_angle(3.0, 4.0, 5.0, Point::new(0.0, 0.0), 0.0);
        // angle=0: AB along +x
        assert!((t.point_ab().x - 3.0).abs() < 0.001);
        assert!((t.point_ab().y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_vertex_placement_angle_90() {
        let t = Triangle::with_angle(3.0, 4.0, 5.0, Point::new(0.0, 0.0), 90.0);
        // angle=90: AB along +y
        assert!((t.point_ab().x - 0.0).abs() < 0.001);
        assert!((t.point_ab().y - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_vertex_placement_nonzero_origin() {
        let origin = Point::new(10.0, 20.0);
        let t = Triangle::with_angle(3.0, 4.0, 5.0, origin, 0.0);
        assert!((t.point_ca().x - 10.0).abs() < 0.001);
        assert!((t.point_ca().y - 20.0).abs() < 0.001);
        assert!((t.point_ab().x - 13.0).abs() < 0.001);
        assert!((t.point_ab().y - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_vertex_distances_match_side_lengths() {
        // For various triangles, verify computed vertex distances match input side lengths
        let cases = [
            (3.0, 4.0, 5.0),
            (5.0, 5.0, 5.0),
            (7.0, 8.0, 9.0),
            (5.71, 9.1, 6.59),
        ];
        for (a, b, c) in cases {
            let t = Triangle::new(a, b, c);
            let ca_ab = t.point_ca().distance_to(t.point_ab());
            let ab_bc = t.point_ab().distance_to(t.point_bc());
            let bc_ca = t.point_bc().distance_to(t.point_ca());
            assert!((ca_ab - a).abs() < 0.01, "({},{},{}): CA→AB={} expected {}", a, b, c, ca_ab, a);
            assert!((ab_bc - b).abs() < 0.01, "({},{},{}): AB→BC={} expected {}", a, b, c, ab_bc, b);
            assert!((bc_ca - c).abs() < 0.01, "({},{},{}): BC→CA={} expected {}", a, b, c, bc_ca, c);
        }
    }

    #[test]
    fn test_new_uses_angle_180() {
        let t1 = Triangle::new(3.0, 4.0, 5.0);
        let t2 = Triangle::with_angle(3.0, 4.0, 5.0, Point::new(0.0, 0.0), 180.0);
        assert!((t1.point_ab().x - t2.point_ab().x).abs() < 0.001);
        assert!((t1.point_ab().y - t2.point_ab().y).abs() < 0.001);
        assert!((t1.point_bc().x - t2.point_bc().x).abs() < 0.001);
        assert!((t1.point_bc().y - t2.point_bc().y).abs() < 0.001);
    }
}
