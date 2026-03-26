//! Property-based tests for triangle-core
//!
//! Uses a seeded PRNG for reproducibility (no external deps).
//! All test functions prefixed with test_prop_.

use triangle_core::triangle::Triangle;
use triangle_core::connection::{build_connected_list, verify_connection};

// ================================================================
// Simple seeded PRNG (xorshift64) for reproducibility
// ================================================================

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self { Self(seed) }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Random f64 in [lo, hi)
    fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        let t = (self.next_u64() % 1_000_000) as f64 / 1_000_000.0;
        lo + t * (hi - lo)
    }
}

/// Generate a valid triangle (satisfying triangle inequality) with sides in [min_side, max_side]
fn random_valid_triangle(rng: &mut Rng, min_side: f64, max_side: f64) -> (f64, f64, f64) {
    loop {
        let a = rng.range_f64(min_side, max_side);
        let b = rng.range_f64(min_side, max_side);
        let c = rng.range_f64(min_side, max_side);
        if a + b > c && b + c > a && c + a > b {
            return (a, b, c);
        }
    }
}

const NUM_TRIALS: usize = 1000;
const SEED: u64 = 20260326;

// ================================================================
// Property: valid triangle always has positive area
// ================================================================

#[test]
fn test_prop_valid_triangle_positive_area() {
    let mut rng = Rng::new(SEED);
    for i in 0..NUM_TRIALS {
        let (a, b, c) = random_valid_triangle(&mut rng, 0.1, 100.0);
        let t = Triangle::new(a, b, c);
        assert!(t.is_valid(), "Trial {}: ({}, {}, {}) should be valid", i, a, b, c);
        assert!(t.area() > 0.0, "Trial {}: ({}, {}, {}) area={} should be > 0", i, a, b, c, t.area());
    }
}

// ================================================================
// Property: vertex distances match input side lengths
// ================================================================

#[test]
fn test_prop_vertex_distances_match_sides() {
    let mut rng = Rng::new(SEED + 1);
    for i in 0..NUM_TRIALS {
        let (a, b, c) = random_valid_triangle(&mut rng, 0.1, 100.0);
        let t = Triangle::new(a, b, c);

        let ca_ab = t.point_ca().distance_to(t.point_ab());
        let ab_bc = t.point_ab().distance_to(t.point_bc());
        let bc_ca = t.point_bc().distance_to(t.point_ca());

        assert!((ca_ab - a).abs() < 0.001,
            "Trial {}: CA→AB={} vs a={}, err={}", i, ca_ab, a, (ca_ab - a).abs());
        assert!((ab_bc - b).abs() < 0.001,
            "Trial {}: AB→BC={} vs b={}, err={}", i, ab_bc, b, (ab_bc - b).abs());
        assert!((bc_ca - c).abs() < 0.001,
            "Trial {}: BC→CA={} vs c={}, err={}", i, bc_ca, c, (bc_ca - c).abs());
    }
}

// ================================================================
// Property: angle sum always equals 180°
// ================================================================

#[test]
fn test_prop_angle_sum_180() {
    let mut rng = Rng::new(SEED + 2);
    for i in 0..NUM_TRIALS {
        let (a, b, c) = random_valid_triangle(&mut rng, 0.1, 100.0);
        let t = Triangle::new(a, b, c);
        let sum = t.angle_a() + t.angle_b() + t.angle_c();
        assert!((sum - 180.0).abs() < 0.01,
            "Trial {}: ({}, {}, {}) angle sum={}", i, a, b, c, sum);
    }
}

// ================================================================
// Property: area is invariant to base angle
// ================================================================

#[test]
fn test_prop_area_invariant_to_angle() {
    let mut rng = Rng::new(SEED + 3);
    for i in 0..500 {
        let (a, b, c) = random_valid_triangle(&mut rng, 0.5, 50.0);
        let angle1 = rng.range_f64(0.0, 360.0);
        let angle2 = rng.range_f64(0.0, 360.0);

        let t1 = Triangle::with_angle(a, b, c, triangle_core::triangle::Point::new(0.0, 0.0), angle1);
        let t2 = Triangle::with_angle(a, b, c, triangle_core::triangle::Point::new(0.0, 0.0), angle2);

        assert!((t1.area() - t2.area()).abs() < 0.01,
            "Trial {}: area should be same at angle {} and {}, got {} vs {}",
            i, angle1, angle2, t1.area(), t2.area());
    }
}

// ================================================================
// Property: area is invariant to origin position
// ================================================================

#[test]
fn test_prop_area_invariant_to_origin() {
    let mut rng = Rng::new(SEED + 4);
    for i in 0..500 {
        let (a, b, c) = random_valid_triangle(&mut rng, 0.5, 50.0);
        let ox = rng.range_f64(-1000.0, 1000.0);
        let oy = rng.range_f64(-1000.0, 1000.0);

        let t1 = Triangle::new(a, b, c);
        let t2 = Triangle::with_angle(a, b, c, triangle_core::triangle::Point::new(ox, oy), 180.0);

        assert!((t1.area() - t2.area()).abs() < 0.01,
            "Trial {}: area at origin vs ({},{}) should match: {} vs {}",
            i, ox, oy, t1.area(), t2.area());
    }
}

// ================================================================
// Property: vertex distances match sides regardless of angle/origin
// ================================================================

#[test]
fn test_prop_vertex_distances_any_placement() {
    let mut rng = Rng::new(SEED + 5);
    for i in 0..500 {
        let (a, b, c) = random_valid_triangle(&mut rng, 0.5, 50.0);
        let ox = rng.range_f64(-500.0, 500.0);
        let oy = rng.range_f64(-500.0, 500.0);
        let angle = rng.range_f64(0.0, 360.0);

        let t = Triangle::with_angle(a, b, c,
            triangle_core::triangle::Point::new(ox, oy), angle);

        let ca_ab = t.point_ca().distance_to(t.point_ab());
        let ab_bc = t.point_ab().distance_to(t.point_bc());
        let bc_ca = t.point_bc().distance_to(t.point_ca());

        assert!((ca_ab - a).abs() < 0.001,
            "Trial {}: CA→AB={} vs a={}", i, ca_ab, a);
        assert!((ab_bc - b).abs() < 0.001,
            "Trial {}: AB→BC={} vs b={}", i, ab_bc, b);
        assert!((bc_ca - c).abs() < 0.001,
            "Trial {}: BC→CA={} vs c={}", i, bc_ca, c);
    }
}

// ================================================================
// Property: connected child vertex distances match after build
// ================================================================

#[test]
fn test_prop_connected_pair_vertex_precision() {
    let mut rng = Rng::new(SEED + 6);
    for i in 0..500 {
        let (a1, b1, c1) = random_valid_triangle(&mut rng, 1.0, 20.0);

        // Child connects to parent's B-edge (type=1): child.A = parent.B
        let child_a = b1;
        let (child_b, child_c) = loop {
            let b = rng.range_f64(0.5, 20.0);
            let c = rng.range_f64(0.5, 20.0);
            if child_a + b > c && b + c > child_a && c + child_a > b {
                break (b, c);
            }
        };

        let rows = vec![
            (a1, b1, c1, -1, -1),
            (child_a, child_b, child_c, 1, 1),
        ];
        let list = build_connected_list(&rows).unwrap();

        // Verify child vertex distances
        let child = &list[1];
        let ca_ab = child.point_ca().distance_to(child.point_ab());
        let ab_bc = child.point_ab().distance_to(child.point_bc());
        let bc_ca = child.point_bc().distance_to(child.point_ca());

        assert!((ca_ab - child_a).abs() < 0.01,
            "Trial {}: child CA→AB={} vs {}", i, ca_ab, child_a);
        assert!((ab_bc - child_b).abs() < 0.01,
            "Trial {}: child AB→BC={} vs {}", i, ab_bc, child_b);
        assert!((bc_ca - child_c).abs() < 0.01,
            "Trial {}: child BC→CA={} vs {}", i, bc_ca, child_c);

        // Verify connection alignment
        assert!(verify_connection(&list[0], &list[1], 1),
            "Trial {}: connection should verify", i);
    }
}

// ================================================================
// Property: Heron area matches cross-product area (independent check)
// ================================================================

#[test]
fn test_prop_heron_matches_cross_product() {
    let mut rng = Rng::new(SEED + 7);
    for i in 0..NUM_TRIALS {
        let (a, b, c) = random_valid_triangle(&mut rng, 0.1, 100.0);
        let t = Triangle::new(a, b, c);

        // Cross-product area from vertices: |CA→AB × CA→BC| / 2
        let p0 = t.point_ca();
        let p1 = t.point_ab();
        let p2 = t.point_bc();
        let cross = (p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x);
        let cross_area = cross.abs() / 2.0;
        let cross_area_rounded = (cross_area * 100.0).round() / 100.0;

        assert!((t.area() - cross_area_rounded).abs() < 0.01,
            "Trial {}: Heron={} vs cross-product={}", i, t.area(), cross_area_rounded);
    }
}
