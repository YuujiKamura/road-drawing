# dxf-engine

DXF file generation, parsing, and validation library. Provides builder-style entity construction (LINE, TEXT, CIRCLE, LWPOLYLINE), a writer that outputs valid DXF strings, a reader that parses DXF back into a document model, a linter for structural validation, and a spatial index for bounding-box queries. Drop-in replacement for the `rust-dxf` crate used by trianglelist-web.

## Usage

```rust
use dxf_engine::{DxfLine, DxfText, DxfWriter, HorizontalAlignment};

let lines = vec![
    DxfLine::new(0.0, 0.0, 100.0, 100.0),
    DxfLine::with_style(0.0, 100.0, 100.0, 0.0, 3, "Layer1"),
];

let texts = vec![
    DxfText::new(50.0, 50.0, "Center")
        .height(2.5)
        .align_h(HorizontalAlignment::Center),
];

let writer = DxfWriter::new();
let dxf_content = writer.write(&lines, &texts);

// Validate output
use dxf_engine::DxfLinter;
let result = DxfLinter::lint(&dxf_content);
assert!(result.is_valid());
```
