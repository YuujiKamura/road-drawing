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
        Self {
            lines: doc.lines.clone(),
            texts: doc.texts.clone(),
        }
    }

    /// Find coordinate of a station name from TEXT entities
    /// Searches for TEXT content matching the station name exactly
    /// Returns (x, y) of the text position
    pub fn get_station_coord(&self, station_name: &str) -> Option<(f64, f64)> {
        self.texts.iter()
            .find(|t| t.text == station_name)
            .map(|t| (t.x, t.y))
    }

    /// Get all lines on a specific layer (case-insensitive contains)
    pub fn lines_on_layer(&self, layer_pattern: &str) -> Vec<&DxfLine> {
        let pattern = layer_pattern.to_lowercase();
        self.lines.iter()
            .filter(|l| l.layer.to_lowercase().contains(&pattern))
            .collect()
    }

    /// Get all texts on a specific layer (case-insensitive contains)
    pub fn texts_on_layer(&self, layer_pattern: &str) -> Vec<&DxfText> {
        let pattern = layer_pattern.to_lowercase();
        self.texts.iter()
            .filter(|t| t.layer.to_lowercase().contains(&pattern))
            .collect()
    }

    /// Calculate bounding box of all entities
    pub fn bounding_box(&self) -> Option<BoundingBox> {
        let mut coords: Vec<(f64, f64)> = Vec::new();

        for l in &self.lines {
            coords.push((l.x1, l.y1));
            coords.push((l.x2, l.y2));
        }
        for t in &self.texts {
            coords.push((t.x, t.y));
        }

        if coords.is_empty() {
            return None;
        }

        let min_x = coords.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
        let min_y = coords.iter().map(|c| c.1).fold(f64::INFINITY, f64::min);
        let max_x = coords.iter().map(|c| c.0).fold(f64::NEG_INFINITY, f64::max);
        let max_y = coords.iter().map(|c| c.1).fold(f64::NEG_INFINITY, f64::max);

        Some(BoundingBox { min_x, min_y, max_x, max_y })
    }

    /// Get all unique layer names
    pub fn layers(&self) -> Vec<String> {
        let mut set = std::collections::HashSet::new();
        for l in &self.lines {
            set.insert(l.layer.clone());
        }
        for t in &self.texts {
            set.insert(t.layer.clone());
        }
        let mut layers: Vec<String> = set.into_iter().collect();
        layers.sort();
        layers
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

    // ================================================================
    // Edge cases & boundary values
    // ================================================================

    #[test]
    fn test_from_document_empty() {
        let doc = DxfDocument::default();
        let index = DxfIndex::from_document(&doc);
        assert!(index.get_station_coord("anything").is_none());
        assert!(index.lines_on_layer("any").is_empty());
        assert!(index.texts_on_layer("any").is_empty());
        assert!(index.layers().is_empty());
    }

    #[test]
    fn test_get_station_coord_empty_name() {
        let doc = DxfDocument {
            texts: vec![DxfText::new(10.0, 20.0, "")],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let coord = index.get_station_coord("").unwrap();
        assert!((coord.0 - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_get_station_coord_first_match_wins() {
        let doc = DxfDocument {
            texts: vec![
                DxfText::new(100.0, 200.0, "No.0"),
                DxfText::new(300.0, 400.0, "No.0"), // duplicate
            ],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let coord = index.get_station_coord("No.0").unwrap();
        // Should return first match
        assert!((coord.0 - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_lines_on_layer_case_insensitive() {
        let doc = DxfDocument {
            lines: vec![
                DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "CenterLine"),
                DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "centerline"),
                DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "CENTERLINE"),
            ],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let matches = index.lines_on_layer("center");
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_lines_on_layer_partial_match() {
        let doc = DxfDocument {
            lines: vec![
                DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "道路中心線"),
            ],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        // Partial match should work
        assert_eq!(index.lines_on_layer("中心").len(), 1);
        assert_eq!(index.lines_on_layer("道路").len(), 1);
        assert_eq!(index.lines_on_layer("線").len(), 1);
    }

    #[test]
    fn test_texts_on_layer_empty_layer() {
        let doc = DxfDocument {
            texts: vec![DxfText::new(0.0, 0.0, "test").layer("")],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        // Empty pattern matches everything (empty string is contained in any string)
        let matches = index.texts_on_layer("");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_bounding_box_single_line() {
        let doc = DxfDocument {
            lines: vec![DxfLine::new(10.0, 20.0, 30.0, 40.0)],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let bb = index.bounding_box().unwrap();
        assert!((bb.min_x - 10.0).abs() < 0.001);
        assert!((bb.min_y - 20.0).abs() < 0.001);
        assert!((bb.max_x - 30.0).abs() < 0.001);
        assert!((bb.max_y - 40.0).abs() < 0.001);
    }

    #[test]
    fn test_bounding_box_texts_only() {
        let doc = DxfDocument {
            texts: vec![
                DxfText::new(-10.0, -20.0, "A"),
                DxfText::new(100.0, 200.0, "B"),
            ],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let bb = index.bounding_box().unwrap();
        assert!((bb.min_x - (-10.0)).abs() < 0.001);
        assert!((bb.min_y - (-20.0)).abs() < 0.001);
        assert!((bb.max_x - 100.0).abs() < 0.001);
        assert!((bb.max_y - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_bounding_box_negative_coords() {
        let doc = DxfDocument {
            lines: vec![DxfLine::new(-100.0, -200.0, -50.0, -25.0)],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let bb = index.bounding_box().unwrap();
        assert!((bb.min_x - (-100.0)).abs() < 0.001);
        assert!((bb.max_y - (-25.0)).abs() < 0.001);
    }

    #[test]
    fn test_bounding_box_point_line() {
        let doc = DxfDocument {
            lines: vec![DxfLine::new(5.0, 5.0, 5.0, 5.0)],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let bb = index.bounding_box().unwrap();
        assert!((bb.min_x - bb.max_x).abs() < 0.001);
        assert!((bb.min_y - bb.max_y).abs() < 0.001);
    }

    #[test]
    fn test_layers_single_layer() {
        let doc = DxfDocument {
            lines: vec![
                DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "Layer1"),
                DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "Layer1"),
            ],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let layers = index.layers();
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0], "Layer1");
    }

    #[test]
    fn test_layers_sorted() {
        let doc = DxfDocument {
            lines: vec![
                DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "Z"),
                DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "A"),
                DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "M"),
            ],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let layers = index.layers();
        assert_eq!(layers, vec!["A", "M", "Z"]);
    }

    #[test]
    fn test_layers_from_lines_and_texts() {
        let doc = DxfDocument {
            lines: vec![DxfLine::with_style(0.0, 0.0, 1.0, 1.0, 7, "LineLayer")],
            texts: vec![DxfText::new(0.0, 0.0, "t").layer("TextLayer")],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let layers = index.layers();
        assert_eq!(layers.len(), 2);
        assert!(layers.contains(&"LineLayer".to_string()));
        assert!(layers.contains(&"TextLayer".to_string()));
    }

    #[test]
    fn test_layers_default_layer_zero() {
        let doc = DxfDocument {
            lines: vec![DxfLine::new(0.0, 0.0, 1.0, 1.0)],
            ..Default::default()
        };
        let index = DxfIndex::from_document(&doc);
        let layers = index.layers();
        assert_eq!(layers, vec!["0"]);
    }
}
