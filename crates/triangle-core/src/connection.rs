//! Parent-child triangle connection and coordinate verification
//!
//! Connection types:
//!   -1 = independent (no parent)
//!    1 = child's A-edge matches parent's B-edge
//!    2 = child's A-edge matches parent's C-edge
//!
//! Constraint: child.length_a == parent.length_{b|c}
//! Coordinate verification: endpoint distance < 0.01

use crate::triangle::Triangle;

const EPSILON: f64 = 0.01;

/// Build a connected triangle list from parsed CSV rows (strict edge validation).
/// Returns error if child's A-edge doesn't match parent's connection edge.
pub fn build_connected_list(
    rows: &[(f64, f64, f64, i32, i32)],
) -> Result<Vec<Triangle>, ConnectionError> {
    build_connected_list_inner(rows, true)
}

/// Build a connected triangle list, skipping edge length mismatch validation.
/// Used for FULL format CSVs where edge lengths may not match exactly.
pub fn build_connected_list_lenient(
    rows: &[(f64, f64, f64, i32, i32)],
) -> Result<Vec<Triangle>, ConnectionError> {
    build_connected_list_inner(rows, false)
}

fn build_connected_list_inner(
    rows: &[(f64, f64, f64, i32, i32)],
    strict_edge_check: bool,
) -> Result<Vec<Triangle>, ConnectionError> {
    let mut triangles: Vec<Triangle> = Vec::with_capacity(rows.len());

    for (i, &(a, b, c, parent_num, conn_type)) in rows.iter().enumerate() {
        let child_num = (i + 1) as i32;

        if parent_num == -1 || conn_type == -1 {
            // Independent triangle
            let mut t = Triangle::new(a, b, c);
            t.parent_number = parent_num;
            t.connection_type = conn_type;
            triangles.push(t);
        } else {
            // Find parent by 1-based number
            let parent_idx = (parent_num - 1) as usize;
            if parent_idx >= triangles.len() {
                return Err(ConnectionError::ParentNotFound {
                    child: child_num,
                    parent: parent_num,
                });
            }
            let parent = &triangles[parent_idx];

            // Validate edge length match
            let parent_edge = match conn_type {
                1 => parent.lengths[1], // parent's B
                2 => parent.lengths[2], // parent's C
                _ => return Err(ConnectionError::InvalidConnectionType {
                    child: child_num,
                    connection_type: conn_type,
                }),
            };

            if strict_edge_check && (a - parent_edge).abs() > EPSILON {
                return Err(ConnectionError::EdgeLengthMismatch {
                    child: child_num,
                    child_a: a,
                    parent: parent_num,
                    parent_edge,
                    connection_type: conn_type,
                });
            }

            // Calculate child's origin and angle from parent's edge
            let (origin, base_angle) = match conn_type {
                1 => {
                    // Child on parent's B-edge: AB→BC
                    let p1 = parent.point_ab();
                    let p2 = parent.point_bc();
                    let angle = (p2.y - p1.y).atan2(p2.x - p1.x).to_degrees();
                    (*p1, angle)
                }
                2 => {
                    // Child on parent's C-edge: BC→CA
                    let p1 = parent.point_bc();
                    let p2 = parent.point_ca();
                    let angle = (p2.y - p1.y).atan2(p2.x - p1.x).to_degrees();
                    (*p1, angle)
                }
                _ => unreachable!(),
            };

            let mut t = Triangle::with_angle(a, b, c, origin, base_angle);
            t.parent_number = parent_num;
            t.connection_type = conn_type;
            triangles.push(t);
        }
    }

    Ok(triangles)
}

/// Verify that child's A-edge endpoints match parent's connection edge endpoints
/// Returns true if distance between corresponding endpoints < epsilon
pub fn verify_connection(parent: &Triangle, child: &Triangle, connection_type: i32) -> bool {
    let (parent_p1, parent_p2) = match connection_type {
        1 => (parent.point_ab(), parent.point_bc()),
        2 => (parent.point_bc(), parent.point_ca()),
        _ => return false,
    };

    let child_ca = child.point_ca();
    let child_ab = child.point_ab();

    // Check both possible alignments (forward and reverse)
    let d_forward = child_ca.distance_to(parent_p1) + child_ab.distance_to(parent_p2);
    let d_reverse = child_ca.distance_to(parent_p2) + child_ab.distance_to(parent_p1);

    d_forward.min(d_reverse) < EPSILON
}

#[derive(Debug)]
pub enum ConnectionError {
    /// Parent triangle not found
    ParentNotFound { child: i32, parent: i32 },
    /// Child's A-edge doesn't match parent's connection edge length
    EdgeLengthMismatch {
        child: i32,
        child_a: f64,
        parent: i32,
        parent_edge: f64,
        connection_type: i32,
    },
    /// Invalid connection type
    InvalidConnectionType { child: i32, connection_type: i32 },
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::ParentNotFound { child, parent } =>
                write!(f, "Triangle {}: parent {} not found", child, parent),
            ConnectionError::EdgeLengthMismatch { child, child_a, parent, parent_edge, connection_type } =>
                write!(f, "Triangle {}: A={} doesn't match parent {} edge (type={}) length {}",
                    child, child_a, parent, connection_type, parent_edge),
            ConnectionError::InvalidConnectionType { child, connection_type } =>
                write!(f, "Triangle {}: invalid connection type {}", child, connection_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 0.01;

    // ================================================================
    // Connection type 1: child's A-edge on parent's B-edge
    // From connected.csv:
    //   Parent 1: A=6, B=5, C=4
    //   Child 2:  A=5, B=4, C=3, parent=1, type=1 (parent's B)
    //   → child.A (5.0) == parent.B (5.0) ✓
    // ================================================================

    #[test]
    fn test_connection_type1_basic() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),  // triangle 1: independent
            (5.0, 4.0, 3.0, 1, 1),    // triangle 2: parent 1, B-edge
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 2);

        // Child's A-edge endpoints should align with parent's B-edge endpoints
        assert!(verify_connection(&list[0], &list[1], 1));
    }

    #[test]
    fn test_connection_type1_vertex_alignment() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.0, 4.0, 3.0, 1, 1),
        ];
        let list = build_connected_list(&rows).unwrap();

        // Parent's B-edge goes from AB to BC
        // Child's A-edge (CA→AB) should match parent's B-edge endpoints
        let parent = &list[0];
        let child = &list[1];

        // One of child's A-edge endpoints should be close to parent's AB
        // Other should be close to parent's BC
        let child_ca = child.point_ca();
        let child_ab = child.point_ab();
        let parent_ab = parent.point_ab();
        let parent_bc = parent.point_bc();

        let d1 = child_ca.distance_to(parent_ab) + child_ab.distance_to(parent_bc);
        let d2 = child_ca.distance_to(parent_bc) + child_ab.distance_to(parent_ab);
        let min_d = d1.min(d2);

        assert!(min_d < EPSILON,
            "Child A-edge should align with parent B-edge, min distance = {}", min_d);
    }

    // ================================================================
    // Connection type 2: child's A-edge on parent's C-edge
    // From connected.csv:
    //   Parent 1: A=6, B=5, C=4
    //   Child 3:  A=4, B=3.5, C=3, parent=1, type=2 (parent's C)
    //   → child.A (4.0) == parent.C (4.0) ✓
    // ================================================================

    #[test]
    fn test_connection_type2_basic() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),  // triangle 1: independent
            (4.0, 3.5, 3.0, 1, 2),    // triangle 3: parent 1, C-edge
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 2);

        assert!(verify_connection(&list[0], &list[1], 2));
    }

    #[test]
    fn test_connection_type2_vertex_alignment() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (4.0, 3.5, 3.0, 1, 2),
        ];
        let list = build_connected_list(&rows).unwrap();

        let parent = &list[0];
        let child = &list[1];

        // Parent's C-edge goes from BC to CA
        let child_ca = child.point_ca();
        let child_ab = child.point_ab();
        let parent_bc = parent.point_bc();
        let parent_ca = parent.point_ca();

        let d1 = child_ca.distance_to(parent_bc) + child_ab.distance_to(parent_ca);
        let d2 = child_ca.distance_to(parent_ca) + child_ab.distance_to(parent_bc);
        let min_d = d1.min(d2);

        assert!(min_d < EPSILON,
            "Child A-edge should align with parent C-edge, min distance = {}", min_d);
    }

    // ================================================================
    // Multi-level connections from connected.csv
    // ================================================================

    #[test]
    fn test_full_connected_csv() {
        // All 7 triangles from connected.csv
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),  // 1: root
            (5.0, 4.0, 3.0,  1,  1),  // 2: parent 1, B
            (4.0, 3.5, 3.0,  1,  2),  // 3: parent 1, C
            (4.0, 3.5, 3.0,  2,  1),  // 4: parent 2, B
            (3.0, 2.5, 2.0,  2,  2),  // 5: parent 2, C
            (3.5, 3.0, 2.5,  3,  1),  // 6: parent 3, B
            (3.0, 2.5, 2.0,  3,  2),  // 7: parent 3, C
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 7);

        // Verify all connections
        assert!(verify_connection(&list[0], &list[1], 1), "2→1 B-edge");
        assert!(verify_connection(&list[0], &list[2], 2), "3→1 C-edge");
        assert!(verify_connection(&list[1], &list[3], 1), "4→2 B-edge");
        assert!(verify_connection(&list[1], &list[4], 2), "5→2 C-edge");
        assert!(verify_connection(&list[2], &list[5], 1), "6→3 B-edge");
        assert!(verify_connection(&list[2], &list[6], 2), "7→3 C-edge");
    }

    // ================================================================
    // Error cases
    // ================================================================

    #[test]
    fn test_parent_not_found() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.0, 4.0, 3.0, 99, 1),  // parent 99 doesn't exist
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_length_mismatch() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (3.0, 4.0, 3.0, 1, 1),  // A=3, but parent B=5 → mismatch
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_err());
    }

    // ================================================================
    // 3+ chain connections with cumulative error check
    // ================================================================

    #[test]
    fn test_chain_3_levels_type1() {
        // Chain: 1 → 2 (B) → 3 (B) — all type1
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),  // 1
            (5.0, 4.0, 3.0,  1,  1),  // 2: parent 1's B=5
            (4.0, 3.0, 2.5,  2,  1),  // 3: parent 2's B=4
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 3);
        assert!(verify_connection(&list[0], &list[1], 1));
        assert!(verify_connection(&list[1], &list[2], 1));
    }

    #[test]
    fn test_chain_3_levels_type2() {
        // Chain: 1 → 2 (C) → 3 (C) — all type2
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),  // 1
            (4.0, 3.5, 3.0,  1,  2),  // 2: parent 1's C=4
            (3.0, 2.5, 2.0,  2,  2),  // 3: parent 2's C=3
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 3);
        assert!(verify_connection(&list[0], &list[1], 2));
        assert!(verify_connection(&list[1], &list[2], 2));
    }

    #[test]
    fn test_chain_cumulative_error() {
        // Verify that vertex positions remain accurate after 3-level chain
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.0, 4.0, 3.0,  1,  1),
            (4.0, 3.0, 2.5,  2,  1),
        ];
        let list = build_connected_list(&rows).unwrap();

        // Check that vertex distances match side lengths for the deepest triangle
        let t = &list[2];
        let ca_ab = t.point_ca().distance_to(t.point_ab());
        let ab_bc = t.point_ab().distance_to(t.point_bc());
        let bc_ca = t.point_bc().distance_to(t.point_ca());

        assert!((ca_ab - 4.0).abs() < 0.01, "Level 3 CA→AB: {} vs 4.0", ca_ab);
        assert!((ab_bc - 3.0).abs() < 0.01, "Level 3 AB→BC: {} vs 3.0", ab_bc);
        assert!((bc_ca - 2.5).abs() < 0.01, "Level 3 BC→CA: {} vs 2.5", bc_ca);
    }

    #[test]
    fn test_chain_mixed_types() {
        // Chain: 1 → 2 (B) → 3 (C)
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.0, 4.0, 3.0,  1,  1),  // parent 1's B=5
            (3.0, 2.5, 2.0,  2,  2),  // parent 2's C=3
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 3);
        assert!(verify_connection(&list[0], &list[1], 1));
        assert!(verify_connection(&list[1], &list[2], 2));
    }

    // ================================================================
    // 5+ triangle chain: cumulative floating-point error < 0.01
    // ================================================================

    /// Helper: check that a triangle's vertex distances match its side lengths within tolerance
    fn assert_side_lengths_match(t: &Triangle, label: &str, tol: f64) {
        let [a, b, c] = t.lengths;
        let ca_ab = t.point_ca().distance_to(t.point_ab());
        let ab_bc = t.point_ab().distance_to(t.point_bc());
        let bc_ca = t.point_bc().distance_to(t.point_ca());
        assert!((ca_ab - a).abs() < tol, "{} CA→AB: {} vs {}, err={}", label, ca_ab, a, (ca_ab - a).abs());
        assert!((ab_bc - b).abs() < tol, "{} AB→BC: {} vs {}, err={}", label, ab_bc, b, (ab_bc - b).abs());
        assert!((bc_ca - c).abs() < tol, "{} BC→CA: {} vs {}, err={}", label, bc_ca, c, (bc_ca - c).abs());
    }

    #[test]
    fn test_chain_5_triangles_type1_cumulative_error() {
        // Linear chain of 5 triangles via B-edge (type1)
        // Each child's A == parent's B
        let rows = vec![
            (10.0, 8.0, 7.0, -1, -1), // 1: root
            ( 8.0, 6.0, 5.0,  1,  1), // 2: A=8 == parent1.B=8
            ( 6.0, 5.0, 4.0,  2,  1), // 3: A=6 == parent2.B=6
            ( 5.0, 4.0, 3.5,  3,  1), // 4: A=5 == parent3.B=5
            ( 4.0, 3.0, 2.5,  4,  1), // 5: A=4 == parent4.B=4
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 5);

        // Verify all connections
        for i in 0..4 {
            assert!(verify_connection(&list[i], &list[i + 1], 1),
                "Connection {}->{} failed", i + 1, i + 2);
        }

        // Verify cumulative error on deepest triangle stays < 0.01
        assert_side_lengths_match(&list[4], "chain5[4]", EPSILON);
    }

    #[test]
    fn test_chain_5_triangles_type2_cumulative_error() {
        // Linear chain of 5 triangles via C-edge (type2)
        // Each child's A == parent's C
        let rows = vec![
            (10.0, 8.0, 7.0, -1, -1), // 1: root
            ( 7.0, 6.0, 5.0,  1,  2), // 2: A=7 == parent1.C=7
            ( 5.0, 4.5, 4.0,  2,  2), // 3: A=5 == parent2.C=5
            ( 4.0, 3.5, 3.0,  3,  2), // 4: A=4 == parent3.C=4
            ( 3.0, 2.5, 2.0,  4,  2), // 5: A=3 == parent4.C=3
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 5);

        for i in 0..4 {
            assert!(verify_connection(&list[i], &list[i + 1], 2),
                "Connection {}->{} failed", i + 1, i + 2);
        }

        assert_side_lengths_match(&list[4], "chain5_type2[4]", EPSILON);
    }

    #[test]
    fn test_chain_5_triangles_alternating_types_cumulative_error() {
        // Chain alternating B→C→B→C connections
        let rows = vec![
            (10.0, 8.0, 7.0, -1, -1), // 1
            ( 8.0, 6.0, 5.0,  1,  1), // 2: A=8 == parent1.B=8 (type1)
            ( 5.0, 4.0, 3.5,  2,  2), // 3: A=5 == parent2.C=5 (type2)
            ( 4.0, 3.5, 3.0,  3,  1), // 4: A=4 == parent3.B=4 (type1)  (fixed: parent3.B=4)
            ( 3.0, 2.5, 2.0,  4,  2), // 5: A=3 == parent4.C=3 (type2)  (fixed: parent4.C=3)
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 5);

        let expected_types = [1, 2, 1, 2];
        for i in 0..4 {
            assert!(verify_connection(&list[i], &list[i + 1], expected_types[i]),
                "Connection {}->{} type{} failed", i + 1, i + 2, expected_types[i]);
        }

        // Cumulative error check on every triangle in the chain
        for (idx, t) in list.iter().enumerate() {
            assert_side_lengths_match(t, &format!("alt_chain[{}]", idx), EPSILON);
        }
    }

    #[test]
    fn test_chain_7_triangles_deep_cumulative_error() {
        // 7-deep linear chain via B-edges: realistic road survey scenario
        let rows = vec![
            (12.0, 10.0, 9.0, -1, -1), // 1
            (10.0,  8.0, 7.0,  1,  1),  // 2
            ( 8.0,  7.0, 6.0,  2,  1),  // 3
            ( 7.0,  6.0, 5.0,  3,  1),  // 4
            ( 6.0,  5.0, 4.5,  4,  1),  // 5
            ( 5.0,  4.5, 4.0,  5,  1),  // 6
            ( 4.5,  4.0, 3.5,  6,  1),  // 7
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 7);

        // Verify all connections
        for i in 0..6 {
            assert!(verify_connection(&list[i], &list[i + 1], 1),
                "Connection {}->{} failed", i + 1, i + 2);
        }

        // Cumulative error on deepest (7th) triangle must stay < 0.01
        assert_side_lengths_match(&list[6], "deep7[6]", EPSILON);

        // Also verify every intermediate triangle
        for (idx, t) in list.iter().enumerate() {
            assert_side_lengths_match(t, &format!("deep7[{}]", idx), EPSILON);
        }
    }

    #[test]
    fn test_chain_5_tree_branching_cumulative_error() {
        // Tree structure: root with 2 children, each with 1 grandchild = 5 triangles
        //       1
        //      / \
        //     2   3
        //     |   |
        //     4   5
        let rows = vec![
            (10.0, 8.0, 7.0, -1, -1), // 1: root
            ( 8.0, 6.0, 5.0,  1,  1), // 2: parent 1's B=8
            ( 7.0, 5.5, 4.5,  1,  2), // 3: parent 1's C=7
            ( 6.0, 4.5, 4.0,  2,  1), // 4: parent 2's B=6
            ( 5.5, 4.0, 3.5,  3,  1), // 5: parent 3's B=5.5
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 5);

        assert!(verify_connection(&list[0], &list[1], 1), "2→1 B");
        assert!(verify_connection(&list[0], &list[2], 2), "3→1 C");
        assert!(verify_connection(&list[1], &list[3], 1), "4→2 B");
        assert!(verify_connection(&list[2], &list[4], 1), "5→3 B");

        // Cumulative error on both leaf triangles
        assert_side_lengths_match(&list[3], "tree_leaf[3]", EPSILON);
        assert_side_lengths_match(&list[4], "tree_leaf[4]", EPSILON);
    }

    #[test]
    fn test_chain_10_triangles_stress_cumulative_error() {
        // 10-deep chain: stress test for floating-point accumulation
        // Each triangle: A=parent.B, shrinking by 0.5 per level
        let rows = vec![
            (15.0, 14.0, 13.0, -1, -1), // 1
            (14.0, 13.0, 12.0,  1,  1),  // 2
            (13.0, 12.0, 11.0,  2,  1),  // 3
            (12.0, 11.0, 10.0,  3,  1),  // 4
            (11.0, 10.0,  9.0,  4,  1),  // 5
            (10.0,  9.0,  8.0,  5,  1),  // 6
            ( 9.0,  8.0,  7.0,  6,  1),  // 7
            ( 8.0,  7.0,  6.0,  7,  1),  // 8
            ( 7.0,  6.0,  5.0,  8,  1),  // 9
            ( 6.0,  5.0,  4.0,  9,  1),  // 10
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 10);

        // Verify all 9 connections
        for i in 0..9 {
            assert!(verify_connection(&list[i], &list[i + 1], 1),
                "Connection {}->{} failed", i + 1, i + 2);
        }

        // Cumulative error on ALL triangles must stay < 0.01
        for (idx, t) in list.iter().enumerate() {
            assert_side_lengths_match(t, &format!("stress10[{}]", idx), EPSILON);
        }
    }

    // ================================================================
    // Self-reference
    // ================================================================

    #[test]
    fn test_self_reference() {
        // Triangle 1 references itself as parent
        let rows = vec![
            (6.0, 5.0, 4.0, 1, 1),  // parent=1 but index 0 hasn't been pushed yet
        ];
        // parent_idx = 0, triangles.len() = 0 → ParentNotFound
        let result = build_connected_list(&rows);
        assert!(result.is_err());
    }

    #[test]
    fn test_self_reference_second_triangle() {
        // Triangle 2 (index 1) references itself as parent
        // parent_num=2, i=1 → parent_idx=1, triangles.len()=1 → ParentNotFound
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),  // 1: independent
            (5.0, 4.0, 3.0,  2,  1),  // 2: parent=2 (self)
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_err());
    }

    #[test]
    fn test_connection_to_index_0_itself() {
        // Triangle at index 0 tries to connect to itself (parent_num=1, which is index 0)
        // At processing time, triangles vec is empty → parent_idx=0 >= len=0 → error
        let rows = vec![
            (5.0, 4.0, 3.0, 1, 1),  // index 0, parent_num=1 → parent_idx=0
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_err());
        // Confirm it's ParentNotFound (not a panic from out-of-bounds)
        match result.unwrap_err() {
            ConnectionError::ParentNotFound { child: 1, parent: 1 } => {},
            other => panic!("Expected ParentNotFound{{child:1, parent:1}}, got {:?}", other),
        }
    }

    // ================================================================
    // Forward reference (parent defined after child)
    // ================================================================

    #[test]
    fn test_forward_reference_parent() {
        // Triangle 1 references parent 2 which hasn't been processed yet
        let rows = vec![
            (5.0, 4.0, 3.0, 2, 1),   // references triangle 2
            (6.0, 5.0, 4.0, -1, -1),  // triangle 2
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_err());
    }

    // ================================================================
    // Invalid connection types
    // ================================================================

    #[test]
    fn test_invalid_connection_type_0() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.0, 4.0, 3.0, 1, 0),  // connection_type 0 is invalid
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_connection_type_3() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.0, 4.0, 3.0, 1, 3),  // connection_type 3 is invalid
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_connection_type_negative() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.0, 4.0, 3.0, 1, -2), // -2 is invalid
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_err());
    }

    // ================================================================
    // Edge length mismatch tolerance
    // ================================================================

    #[test]
    fn test_edge_length_within_epsilon() {
        // Mismatch just under EPSILON (0.01) should pass
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.005, 4.0, 3.0, 1, 1),  // diff = 0.005 < 0.01
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_length_at_epsilon_boundary() {
        // Mismatch exactly at EPSILON boundary
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.011, 4.0, 3.0, 1, 1),  // diff = 0.011 > 0.01 → fail
        ];
        let result = build_connected_list(&rows);
        assert!(result.is_err());
    }

    // ================================================================
    // verify_connection edge cases
    // ================================================================

    #[test]
    fn test_verify_connection_invalid_type() {
        let t1 = Triangle::new(3.0, 4.0, 5.0);
        let t2 = Triangle::new(4.0, 3.0, 2.0);
        assert!(!verify_connection(&t1, &t2, 0));
        assert!(!verify_connection(&t1, &t2, 3));
        assert!(!verify_connection(&t1, &t2, -1));
    }

    // ================================================================
    // Single / empty
    // ================================================================

    #[test]
    fn test_single_independent_triangle() {
        let rows = vec![(6.0, 5.0, 4.0, -1, -1)];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].parent_number, -1);
    }

    #[test]
    fn test_empty_rows() {
        let rows: Vec<(f64, f64, f64, i32, i32)> = vec![];
        let list = build_connected_list(&rows).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_multiple_independent_triangles() {
        let rows = vec![
            (3.0, 4.0, 5.0, -1, -1),
            (5.0, 5.0, 5.0, -1, -1),
            (7.0, 8.0, 9.0, -1, -1),
        ];
        let list = build_connected_list(&rows).unwrap();
        assert_eq!(list.len(), 3);
        for t in &list {
            assert_eq!(t.parent_number, -1);
            assert_eq!(t.connection_type, -1);
        }
    }

    // ================================================================
    // ConnectionError Display
    // ================================================================

    #[test]
    fn test_error_display_parent_not_found() {
        let err = ConnectionError::ParentNotFound { child: 2, parent: 99 };
        let msg = format!("{}", err);
        assert!(msg.contains("99"));
        assert!(msg.contains("2"));
    }

    #[test]
    fn test_error_display_edge_mismatch() {
        let err = ConnectionError::EdgeLengthMismatch {
            child: 2, child_a: 3.0, parent: 1, parent_edge: 5.0, connection_type: 1,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("3"));
        assert!(msg.contains("5"));
    }

    #[test]
    fn test_error_display_invalid_type() {
        let err = ConnectionError::InvalidConnectionType { child: 2, connection_type: 0 };
        let msg = format!("{}", err);
        assert!(msg.contains("0"));
    }

    // ================================================================
    // Parent number edge: parent=0 (1-based, so 0 is underflow)
    // ================================================================

    #[test]
    fn test_parent_number_zero() {
        // parent_num=0 → parent_idx = -1 as usize → huge number → out of bounds
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.0, 4.0, 3.0, 0, 1),  // parent=0
        ];
        let result = build_connected_list(&rows);
        // 0 - 1 = underflow → parent_idx huge → ParentNotFound
        assert!(result.is_err());
    }

    // ================================================================
    // build_connected_list_lenient tests
    // ================================================================

    #[test]
    fn test_conn_lenient_single_independent() {
        let rows = vec![(3.0, 4.0, 5.0, -1, -1)];
        let result = build_connected_list_lenient(&rows);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_conn_lenient_skips_edge_mismatch() {
        // child.a=3.0 but parent.b=4.0 — strict would fail, lenient should succeed
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),  // triangle 1: a=6, b=5, c=4
            (3.0, 4.0, 3.0, 1, 1),     // triangle 2: a=3 != parent.b=5, conn_type=1
        ];
        let strict = build_connected_list(&rows);
        let lenient = build_connected_list_lenient(&rows);
        assert!(strict.is_err(), "Strict should reject edge mismatch");
        assert!(lenient.is_ok(), "Lenient should accept edge mismatch");
        assert_eq!(lenient.unwrap().len(), 2);
    }

    #[test]
    fn test_conn_lenient_type2_skips_edge_mismatch() {
        // conn_type=2 → child.a should match parent.c, but doesn't
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),  // triangle 1: a=6, b=5, c=4
            (99.0, 3.0, 3.0, 1, 2),    // triangle 2: a=99 != parent.c=4, conn_type=2
        ];
        let strict = build_connected_list(&rows);
        let lenient = build_connected_list_lenient(&rows);
        assert!(strict.is_err());
        assert!(lenient.is_ok());
    }

    #[test]
    fn test_conn_lenient_matching_edges_also_works() {
        // When edges DO match, lenient should work identically to strict
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (5.0, 4.0, 3.0, 1, 1),  // a=5 matches parent.b=5
        ];
        let strict = build_connected_list(&rows).unwrap();
        let lenient = build_connected_list_lenient(&rows).unwrap();
        assert_eq!(strict.len(), lenient.len());
    }

    #[test]
    fn test_conn_lenient_still_rejects_invalid_parent() {
        // Lenient skips edge checks but not parent existence
        let rows = vec![
            (3.0, 4.0, 5.0, -1, -1),
            (3.0, 4.0, 5.0, 99, 1),  // parent 99 doesn't exist
        ];
        let result = build_connected_list_lenient(&rows);
        assert!(result.is_err());
    }

    #[test]
    fn test_conn_lenient_still_rejects_invalid_conn_type() {
        let rows = vec![
            (3.0, 4.0, 5.0, -1, -1),
            (3.0, 4.0, 5.0, 1, 0),  // conn_type=0 is invalid
        ];
        let result = build_connected_list_lenient(&rows);
        assert!(result.is_err());
    }

    #[test]
    fn test_conn_lenient_chain_three_triangles() {
        let rows = vec![
            (6.0, 5.0, 4.0, -1, -1),
            (3.0, 4.0, 3.0, 1, 1),   // mismatch a=3 vs parent.b=5
            (2.0, 3.0, 2.0, 2, 2),   // mismatch a=2 vs parent.c=3
        ];
        let result = build_connected_list_lenient(&rows);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_conn_lenient_empty_input() {
        let rows: Vec<(f64, f64, f64, i32, i32)> = vec![];
        let result = build_connected_list_lenient(&rows);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
