# Migration Guide: trianglelist → road-drawing

## Overview

`road-drawing` consolidates the DXF generation code previously spread across:
- `trianglelist/rust-dxf/` → `road-drawing/crates/dxf-engine/`
- `trianglelist-web/src/road_section.rs` → `road-drawing/crates/road-section/`
- `csv_to_dxf/src/processing.py` → `road-drawing/crates/excel-parser/`

This guide covers switching `trianglelist-web` from `rust-dxf` (local path dep) to `dxf-engine` (git dep from road-drawing).

## Step 1: Update Cargo.toml

### Before (rust-dxf local path)
```toml
[dependencies]
dxf = { path = "../rust-dxf" }
```

### After (dxf-engine from road-drawing)
```toml
[dependencies]
dxf-engine = { git = "https://github.com/YuujiKamura/road-drawing", package = "dxf-engine" }
road-section = { git = "https://github.com/YuujiKamura/road-drawing", package = "road-section" }
triangle-core = { git = "https://github.com/YuujiKamura/road-drawing", package = "triangle-core" }
```

After crates.io publish (Phase 5):
```toml
[dependencies]
dxf-engine = "0.1"
road-section = "0.1"
triangle-core = "0.1"
```

## Step 2: Update `use` statements

### Entity types

| Before (rust-dxf) | After (dxf-engine) |
|---|---|
| `use dxf::DxfLine;` | `use dxf_engine::DxfLine;` |
| `use dxf::DxfText;` | `use dxf_engine::DxfText;` |
| `use dxf::DxfWriter;` | `use dxf_engine::DxfWriter;` |
| `use dxf::{HorizontalAlignment, VerticalAlignment};` | `use dxf_engine::{HorizontalAlignment, VerticalAlignment};` |

### New types available in dxf-engine (not in rust-dxf)

```rust
use dxf_engine::DxfCircle;      // Circle entity
use dxf_engine::DxfLwPolyline;  // Lightweight polyline
use dxf_engine::DxfLinter;      // DXF validation
use dxf_engine::parse_dxf;      // DXF reader (new!)
use dxf_engine::DxfDocument;    // Parsed DXF container
use dxf_engine::DxfIndex;       // Spatial index + station lookup
use dxf_engine::BoundingBox;    // Bounding box
```

### Road section (replaces trianglelist-web's road_section.rs)

| Before (local module) | After (road-section crate) |
|---|---|
| `use crate::road_section::StationData;` | `use road_section::StationData;` |
| `use crate::road_section::calculate_road_section;` | `use road_section::calculate_road_section;` |
| `use crate::road_section::geometry_to_dxf;` | `use road_section::geometry_to_dxf;` |
| `use crate::road_section::RoadSectionConfig;` | `use road_section::RoadSectionConfig;` |
| `use crate::road_section::parse_road_section_csv;` | `use road_section::parse_road_section_csv;` |

### Triangle core (replaces Kotlin Triangle logic)

```rust
use triangle_core::triangle::{Triangle, Point};
use triangle_core::csv_loader::parse_csv;
use triangle_core::connection::{build_connected_list, verify_connection};
```

## Step 3: API changes

### DxfWriter

The API is identical. No changes needed:

```rust
// Both old and new
let writer = DxfWriter::new();
let content = writer.write(&lines, &texts);

// New: write_all (includes circles and polylines)
let mut writer = DxfWriter::new();
let content = writer.write_all(&lines, &texts, &circles, &polylines);
```

### DxfLine / DxfText

Constructors and builder methods are identical:

```rust
// Both old and new
DxfLine::new(x1, y1, x2, y2)
DxfLine::with_style(x1, y1, x2, y2, color, "layer")
DxfText::new(x, y, "text").height(350.0).rotation(-90.0).color(5)
```

### Float output format change

dxf-engine always outputs floats with a decimal point (`5` → `5.0`).
This is a DXF best practice and should not affect CAD software compatibility.

## Step 4: Remove old code

After switching dependencies and verifying tests pass:

1. Delete `trianglelist/rust-dxf/` directory
2. Delete `trianglelist-web/src/road_section.rs` (now in road-section crate)
3. Remove `mod road_section;` from `trianglelist-web/src/lib.rs`
4. Update `trianglelist-web/src/dxf/converter.rs` imports

## Step 5: Verify

```bash
# In trianglelist-web/
cargo test
trunk serve  # Check browser rendering

# In road-drawing/
cargo test   # Should still pass (633+ tests)
```

## File mapping reference

| trianglelist (old) | road-drawing (new) |
|---|---|
| `rust-dxf/src/dxf/entities.rs` | `crates/dxf-engine/src/dxf/entities.rs` |
| `rust-dxf/src/dxf/writer.rs` | `crates/dxf-engine/src/dxf/writer.rs` |
| `rust-dxf/src/dxf/handle.rs` | `crates/dxf-engine/src/dxf/handle.rs` |
| `rust-dxf/src/dxf/linter.rs` | `crates/dxf-engine/src/dxf/linter.rs` |
| (none) | `crates/dxf-engine/src/dxf/reader.rs` |
| (none) | `crates/dxf-engine/src/dxf/index.rs` |
| `trianglelist-web/src/road_section.rs` | `crates/road-section/src/lib.rs` |
| `csv_to_dxf/src/processing.py` | `crates/excel-parser/src/` |
| Kotlin `Triangle.kt` | `crates/triangle-core/src/triangle.rs` |
| Kotlin `CsvLoader.kt` | `crates/triangle-core/src/csv_loader.rs` |
| Kotlin `CrosswalkGenerator.kt` | `crates/road-marking/src/crosswalk.rs` |
| Kotlin `MarkingCommand.kt` | `crates/road-marking/src/command.rs` |
