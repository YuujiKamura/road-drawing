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

/// Build a connected triangle list from parsed CSV rows
/// Returns triangles with computed vertices based on parent-child connections
pub fn build_connected_list(
    rows: &[(f64, f64, f64, i32, i32)], // (a, b, c, parent_number, connection_type)
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

            if (a - parent_edge).abs() > EPSILON {
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
}
