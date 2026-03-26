# triangle-core

Triangle list geometry engine for area measurement drawings. Computes triangle area via Heron's formula (rounded to 2 decimals), vertex positions via the cosine rule, and internal angles. Supports parent-child connections where a child's A-edge attaches to a parent's B or C edge, building a connected mesh from CSV data in MIN (4-column), CONN (6-column), or FULL (28-column) formats.

## Usage

```rust
use triangle_core::triangle::Triangle;
use triangle_core::csv_loader::parse_csv;
use triangle_core::connection::build_connected_list;

// Parse CSV
let csv = "koujiname, Test\nrosenname, Route\ngyousyaname, Builder\nzumennum, 1\n\
           1, 6.0, 5.0, 4.0, -1, -1\n\
           2, 5.0, 4.0, 3.0, 1, 1\n";
let parsed = parse_csv(csv).unwrap();

// Build connected triangle mesh
let rows: Vec<_> = parsed.triangles.iter().map(|t| {
    (t.length_a, t.length_b, t.length_c, t.parent_number, t.connection_type)
}).collect();
let triangles = build_connected_list(&rows).unwrap();

// Query geometry
let t = &triangles[0];
println!("Area: {}", t.area());           // Heron's formula
println!("Angle A: {:.1}°", t.angle_a()); // Internal angle
```
