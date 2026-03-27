//! DXF file generation and validation library
//!
//! Provides functionality to generate and validate DXF (Drawing Exchange Format) files.
//!
//! # Example
//!
//! ```
//! use dxf_engine::{DxfLine, DxfText, DxfWriter};
//!
//! let lines = vec![
//!     DxfLine::new(0.0, 0.0, 100.0, 100.0),
//!     DxfLine::with_style(0.0, 100.0, 100.0, 0.0, 3, "Layer1"),
//! ];
//!
//! let texts = vec![
//!     DxfText::new(50.0, 50.0, "Center"),
//! ];
//!
//! let writer = DxfWriter::new();
//! let dxf_content = writer.write(&lines, &texts);
//! assert!(dxf_content.contains("LINE"));
//! assert!(dxf_content.contains("TEXT"));
//! ```

pub mod dxf;

// Re-export all public types at crate root.
// These match the old `dxf` (rust-dxf) crate API 1:1.
// Migration from trianglelist-web:
//   Option A (zero code changes): In Cargo.toml use a rename:
//     dxf = { package = "dxf-engine", path = "../dxf-engine" }
//     Then `use dxf::{DxfLine, ...}` and `dxf::HorizontalAlignment::Left` still work.
//   Option B: Replace `use dxf::` with `use dxf_engine::` in source files.
pub use dxf::entities::{DxfCircle, DxfLine, DxfLwPolyline, DxfText, HorizontalAlignment, VerticalAlignment};
pub use dxf::handle::{HandleGenerator, owners};
pub use dxf::linter::{DxfLinter, LintResult, LintError, LintErrorCode};
pub use dxf::reader::{DxfDocument, ReaderError, parse_dxf};
pub use dxf::index::{DxfIndex, BoundingBox};
pub use dxf::writer::DxfWriter;
pub use dxf::comparator::{DxfComparable, compare_dxf_strings};
