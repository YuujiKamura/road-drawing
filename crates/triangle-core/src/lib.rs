//! Triangle list geometry calculation engine
//!
//! Ported from trianglelist (Kotlin Multiplatform) + rust-trilib
//!
//! Core concepts:
//! - Triangle: 3 sides (A=connection edge, B/C=free edges)
//! - Connection: child's A-edge matches parent's B or C edge
//! - Area: Heron's formula, rounded to 2 decimal places
//! - Vertices: CA=origin, AB=x-axis, BC=cosine rule

pub mod triangle;
pub mod csv_loader;
pub mod connection;
