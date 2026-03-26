# road-drawing Project Summary

Road section and triangle list diagram generator. Converts CSV/Excel input to DXF drawings for civil engineering road construction projects.

## Test Results: 674 tests, all passing

| Crate | Unit | Integration | Total |
|-------|------|-------------|-------|
| dxf-engine | 228 | 10 (compat) | 238 |
| triangle-core | 120 | 19 + 8 (property) | 147 |
| road-section | 83 | 18 (e2e) | 101 |
| excel-parser | 77 | 7 (calamine) + 1 (real_file) | 85 |
| road-drawing-web | 51 | — | 51 |
| road-marking | 44 | — | 44 |
| cli | — | 6 (cli_test) | 6 |
| doctests | 2 | — | 2 |
| **Total** | **605** | **69** | **674** |

## Architecture

```
road-drawing/
├── crates/
│   ├── dxf-engine/       DXF read/write/lint/index (238 tests)
│   ├── triangle-core/    Triangle list geometry engine (147 tests)
│   ├── road-section/     Road section diagram generator (101 tests)
│   ├── excel-parser/     Excel/CSV input parser (85 tests)
│   └── road-marking/     Crosswalk/marking generator (44 tests)
├── cli/                  Command-line tool (6 tests)
├── web/                  egui WASM web app (51 tests)
└── .github/workflows/    GitHub Pages deploy CI
```

### Dependency graph

```
road-marking → triangle-core → dxf-engine
road-section → dxf-engine
excel-parser → (calamine, encoding_rs)
cli → excel-parser + road-section + triangle-core + road-marking
web → dxf-engine + road-section + excel-parser + triangle-core
```

## Phase Completion

### Phase 1: Core (dxf-engine + road-section + CLI)
- DXF entity builders: Line, Text, Circle, LwPolyline
- DXF writer with handle management, section structure, EOF
- DXF linter with 8 error codes (handle uniqueness, section pairing, chunk integrity)
- Road section geometry: station data → width lines, connecting outlines, dimension texts
- Scale conversion: meters → millimeters (x1000 default)
- CSV parser with Japanese header detection

### Phase 2: Excel Parser
- Section detector: finds `区間X,台形計算` headers in multi-section files
- Station name generator: 20m pitch naming (No.0, No.0+10.5 format)
- Distance converter: cumulative vs incremental detection (median < 16m threshold)
- Transform pipeline: extract → to_cumulative → fill_station_names → round
- Shift_JIS and UTF-8 encoding support via encoding_rs

### Phase 3: Triangle Core + Road Marking
- Triangle geometry: Heron area, cosine vertex placement, parent-child connections
- Connection engine: type 1 (B-edge) and type 2 (C-edge), chain verification
- CSV loader: MIN (4-col), CONN (6-col), FULL (28-col) formats with header parsing
- DXF reader: parse ENTITIES section back to entity structs
- DXF spatial index: station coordinate lookup, layer filtering, bounding box
- Crosswalk generator: perpendicular stripes from centerline path
- JSON command engine: parse and execute marking commands

### Phase 4: Web UI (egui WASM)
- eframe 0.29 + egui 0.29 application
- CSV drag-and-drop with Shift_JIS auto-detection
- Road section preview: DXF Y-up → screen Y-down coordinate transform
- Viewport: auto-fit with 90% margin, aspect-ratio-aware scaling
- DXF export: stations_to_dxf() with roundtrip verification
- DXF color mapping: 7 standard CAD colors
- WASM build: `trunk build --release` (calamine compiles natively for wasm32)
- GitHub Pages CI: `.github/workflows/deploy.yml` (push to master → deploy)

## CLI Usage

```bash
# Generate road section DXF from CSV
road-drawing generate --input data.csv --output output.dxf --type road-section

# Custom scale factor
road-drawing generate --input data.csv --output output.dxf --scale 500

# Excel with section selection
road-drawing generate --input data.xlsx --output output.dxf --section "区間1"

# List available sections
road-drawing generate --input data.xlsx --output /dev/null --list-sections

# Generate marking from JSON commands
road-drawing generate --input commands.json --output marking.dxf --type marking
```

## Web App

```bash
# Development (native window)
cargo run -p road-drawing-web

# WASM build
cd web && trunk build --release

# Local preview
cd web && trunk serve
```

Deploy: push to master triggers GitHub Actions → trunk build → GitHub Pages.
URL: `https://yujikamura.github.io/road-drawing/`

Enable: Settings → Pages → Source: GitHub Actions.

## Known Issues

1. **FULL 28-column CSV**: Parser accepts the format but silently ignores columns 7-28 (name, color, dim alignment, angle). Only the first 6 columns (number, A, B, C, parent, connection_type) are extracted.

2. **DXF linter warnings**: The `warnings` field in `LintResult` is initialized but never populated. All issues are classified as errors. No warning-level diagnostics exist.

3. **WASM size**: Debug WASM is 6.2MB. Release with `wasm-opt` should bring it under 2MB. No `wasm-opt` step in CI yet.

4. **Phase 2.5 not started**: LLM-based format conversion (arbitrary Excel → master CSV format) is planned but not implemented.

5. **Phase 5 not started**: trianglelist dependency switch and crates.io publish pending.

6. **GitHub Pages**: Requires manual activation in repo settings (Source: GitHub Actions).
