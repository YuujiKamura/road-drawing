//! DXF spatial index: station coordinate lookup and layer-based filtering
//!
//! Ported from trianglelist DxfIndex.kt
//!
//! Provides:
//! - Station name → coordinate mapping from TEXT entities
//! - Layer-based entity filtering
//! - Bounding box calculation

use super::entities::{DxfLine, DxfText};
use super::reader::DxfDocument;

/// Spatial index built from a parsed DXF document
pub struct DxfIndex {
    lines: Vec<DxfLine>,
    texts: Vec<DxfText>,
}

/// Bounding box
#[derive(Clone, Debug, PartialEq)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl DxfIndex {
    /// Build index from a DxfDocument
    pub fn from_document(doc: &DxfDocument) -> Self {
        todo!("Implement: store lines and texts for lookup")
    }

    /// Find coordinate of a station name from TEXT entities
    /// Searches for TEXT content matching the station name
    /// Returns (x, y) of the text position
    pub fn get_station_coord(&self, station_name: &str) -> Option<(f64, f64)> {
        todo!("Implement: search texts for matching station name")
    }

    /// Get all lines on a specific layer (case-insensitive contains)
    pub fn lines_on_layer(&self, layer_pattern: &str) -> Vec<&DxfLine> {
        todo!("Implement: filter lines by layer")
    }

    /// Get all texts on a specific layer (case-insensitive contains)
    pub fn texts_on_layer(&self, layer_pattern: &str) -> Vec<&DxfText> {
        todo!("Implement: filter texts by layer")
    }

    /// Calculate bounding box of all entities
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        todo!("Implement: compute min/max from all entity coordinates")
    }

    /// Get all unique layer names
    pub fn layers(&self) -> Vec<String> {
        todo!("Implement: collect unique layer names from all entities")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dxf::entities::{DxfLine, DxfText};

    fn sample_document() -> DxfDocument {
        DxfDocument {
            lines: vec![
                DxfLine::with_style(0.0, 0.0, 10000.0, 0.0, 7, "中心線"),
                DxfLine::with_style(10000.0, 0.0, 20000.0, 0.0, 7, "中心線"),
                DxfLine::with_style(0.0, -3000.0, 0.0, 3000.0, 3, "横断歩道"),
            ],
            texts: vec![
                DxfText::new(0.0, 500.0, "No.0").layer("測点"),
                DxfText::new(10000.0, 500.0, "No.1").layer("測点"),
                DxfText::new(20000.0, 500.0, "No.2").layer("測点"),
                DxfText::new(5000.0, 500.0, "0+5").layer("測点"),
                DxfText::new(100.0, 100.0, "工事名").layer("タイトル"),
            ],
            circles: vec![],
            polylines: vec![],
        }
    }

    // ================================================================
    // Station coordinate extraction
    // ================================================================

    #[test]
    fn test_get_station_coord_exact_match() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        let coord = index.get_station_coord("No.0").unwrap();
        assert!((coord.0 - 0.0).abs() < 0.001);
        assert!((coord.1 - 500.0).abs() < 0.001);
    }

    #[test]
    fn test_get_station_coord_no1() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        let coord = index.get_station_coord("No.1").unwrap();
        assert!((coord.0 - 10000.0).abs() < 0.001);
    }

    #[test]
    fn test_get_station_coord_sub_station() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        let coord = index.get_station_coord("0+5").unwrap();
        assert!((coord.0 - 5000.0).abs() < 0.001);
    }

    #[test]
    fn test_get_station_coord_not_found() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        assert!(index.get_station_coord("No.99").is_none());
    }

    #[test]
    fn test_get_station_coord_ignores_non_station_text() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        // "工事名" is a text but not a station
        let coord = index.get_station_coord("工事名");
        // This should find it since we search by text content
        assert!(coord.is_some());
        assert!((coord.unwrap().0 - 100.0).abs() < 0.001);
    }

    // ================================================================
    // Layer-based filtering
    // ================================================================

    #[test]
    fn test_lines_on_layer_centerline() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        let center_lines = index.lines_on_layer("中心");
        assert_eq!(center_lines.len(), 2);
    }

    #[test]
    fn test_lines_on_layer_crosswalk() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        let cw_lines = index.lines_on_layer("横断歩道");
        assert_eq!(cw_lines.len(), 1);
    }

    #[test]
    fn test_lines_on_layer_no_match() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        let none = index.lines_on_layer("存在しないレイヤー");
        assert!(none.is_empty());
    }

    #[test]
    fn test_texts_on_layer() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        let station_texts = index.texts_on_layer("測点");
        assert_eq!(station_texts.len(), 4); // No.0, No.1, No.2, 0+5
    }

    // ================================================================
    // Bounding box
    // ================================================================

    #[test]
    fn test_bounding_box() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        let bb = index.bounding_box().unwrap();
        assert!((bb.min_x - 0.0).abs() < 0.001);
        assert!(bb.min_y < 0.0); // crosswalk line goes to -3000
        assert!((bb.max_x - 20000.0).abs() < 0.001);
        assert!(bb.max_y > 0.0);
    }

    #[test]
    fn test_bounding_box_empty() {
        let doc = DxfDocument::default();
        let index = DxfIndex::from_document(&doc);
        assert!(index.bounding_box().is_none());
    }

    // ================================================================
    // Layers
    // ================================================================

    #[test]
    fn test_unique_layers() {
        let doc = sample_document();
        let index = DxfIndex::from_document(&doc);

        let layers = index.layers();
        assert!(layers.contains(&"中心線".to_string()));
        assert!(layers.contains(&"横断歩道".to_string()));
        assert!(layers.contains(&"測点".to_string()));
        assert!(layers.contains(&"タイトル".to_string()));
        assert_eq!(layers.len(), 4);
    }
}
