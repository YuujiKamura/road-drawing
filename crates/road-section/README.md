# road-section

Road cross-section (路線展開図) geometry calculator and DXF generator. Takes station data (name, distance, left/right widths) and produces scaled line segments and dimension texts suitable for civil engineering drawings. Outputs DXF LINE and TEXT entities via `dxf-engine`, with station labels in blue (color 5) and text rotated -90 degrees per convention.

## Usage

```rust
use road_section::{calculate_road_section, geometry_to_dxf, StationData, RoadSectionConfig};
use dxf_engine::DxfWriter;

let stations = vec![
    StationData { name: "NO.1".into(), distance: 0.0, left: 3.5, right: 3.5 },
    StationData { name: "NO.2".into(), distance: 20.0, left: 3.0, right: 4.0 },
];
let config = RoadSectionConfig::default();
let geometry = calculate_road_section(&stations, &config);
let (lines, texts) = geometry_to_dxf(&geometry);

let writer = DxfWriter::new();
let dxf = writer.write(&lines, &texts);
```
