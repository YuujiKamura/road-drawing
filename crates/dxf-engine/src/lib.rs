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

pub use dxf::entities::{DxfCircle, DxfLine, DxfLwPolyline, DxfText, HorizontalAlignment, VerticalAlignment};
pub use dxf::handle::{HandleGenerator, owners};
pub use dxf::linter::{DxfLinter, LintResult, LintError, LintErrorCode};
pub use dxf::writer::DxfWriter;
