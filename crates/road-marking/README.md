# road-marking

Road marking generator for crosswalks, stop lines, and centerline markings. Builds a centerline path from DXF LINE entities, then generates perpendicular stripe patterns at specified station offsets. Supports JSON command input for batch marking generation with configurable stripe width, gap, count, and anchor position.

## Usage

```rust
use road_marking::crosswalk::{generate_crosswalk, CrosswalkConfig, build_centerline_path};
use dxf_engine::DxfLine;

let centerlines = vec![
    DxfLine::new(0.0, 0.0, 100.0, 0.0),
];
let config = CrosswalkConfig {
    station: 50.0,
    width: 3.0,
    stripe_width: 0.45,
    stripe_gap: 0.45,
    stripe_count: 6,
    offset: 0.0,
    anchor: "center".to_string(),
};
let stripes = generate_crosswalk(&centerlines, &config);
```
