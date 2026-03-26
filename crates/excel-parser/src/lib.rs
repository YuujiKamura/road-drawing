//! Excel/CSV parser for road section survey data
//!
//! Ported from csv_to_dxf's processing.py and station_name_utils.py

pub mod section_detector;
pub mod station_name;
pub mod distance;
pub mod transform;

/// Constants matching Python source
pub const PITCH_M: f64 = 20.0;
pub const ROUND_N: u32 = 2;
pub const SPAN: f64 = 20.0;

/// Raw row extracted from CSV/Excel before transformation
#[derive(Clone, Debug, PartialEq)]
pub struct RawRow {
    pub name: String,
    pub x: f64,
    pub wl: f64,
    pub wr: f64,
}
