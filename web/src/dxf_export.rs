//! DXF export: convert road section geometry to DXF text for download.

use dxf_engine::DxfWriter;
use road_section::{
    calculate_road_section, geometry_to_dxf, RoadSectionConfig, RoadSectionGeometry, StationData,
};

/// Generate DXF text from station data using default config.
pub fn stations_to_dxf(stations: &[StationData]) -> String {
    let config = RoadSectionConfig::default();
    let geometry = calculate_road_section(stations, &config);
    geometry_to_dxf_string(&geometry)
}

/// Generate DXF text from station data with custom scale.
pub fn stations_to_dxf_with_scale(stations: &[StationData], scale: f64) -> String {
    let config = RoadSectionConfig {
        scale,
        ..Default::default()
    };
    let geometry = calculate_road_section(stations, &config);
    geometry_to_dxf_string(&geometry)
}

/// Convert RoadSectionGeometry to DXF string.
pub fn geometry_to_dxf_string(geometry: &RoadSectionGeometry) -> String {
    let (lines, texts) = geometry_to_dxf(geometry);
    let writer = DxfWriter::new();
    writer.write(&lines, &texts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stations() -> Vec<StationData> {
        vec![
            StationData::new("No.0", 0.0, 3.45, 3.55),
            StationData::new("No.1", 20.0, 3.50, 3.50),
            StationData::new("No.2", 40.0, 3.55, 3.55),
        ]
    }

    #[test]
    fn test_stations_to_dxf_produces_valid_output() {
        let dxf = stations_to_dxf(&sample_stations());
        assert!(dxf.contains("SECTION"));
        assert!(dxf.contains("ENTITIES"));
        assert!(dxf.contains("EOF"));
        assert!(dxf.contains("LINE"));
    }

    #[test]
    fn test_stations_to_dxf_contains_station_names() {
        let dxf = stations_to_dxf(&sample_stations());
        assert!(dxf.contains("No.0"));
        assert!(dxf.contains("No.1"));
        assert!(dxf.contains("No.2"));
    }

    #[test]
    fn test_stations_to_dxf_with_scale() {
        let dxf = stations_to_dxf_with_scale(&sample_stations(), 500.0);
        assert!(dxf.contains("LINE"));
        assert!(dxf.contains("EOF"));
    }

    #[test]
    fn test_empty_stations_produces_minimal_dxf() {
        let dxf = stations_to_dxf(&[]);
        assert!(dxf.contains("SECTION"));
        assert!(dxf.contains("EOF"));
    }

    #[test]
    fn test_geometry_to_dxf_string_roundtrip() {
        let stations = sample_stations();
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let dxf = geometry_to_dxf_string(&geometry);

        // Parse back with reader
        let doc = dxf_engine::parse_dxf(&dxf).unwrap();
        assert!(!doc.lines.is_empty());
        assert!(!doc.texts.is_empty());
    }

    // ================================================================
    // DXF lint validation
    // ================================================================

    #[test]
    fn test_export_passes_dxf_linter() {
        let dxf = stations_to_dxf(&sample_stations());
        assert!(dxf_engine::DxfLinter::is_valid(&dxf),
            "Exported DXF must pass linter validation");
    }

    #[test]
    fn test_export_with_scale_passes_linter() {
        let dxf = stations_to_dxf_with_scale(&sample_stations(), 500.0);
        assert!(dxf_engine::DxfLinter::is_valid(&dxf));
    }

    #[test]
    fn test_export_empty_passes_linter() {
        let dxf = stations_to_dxf(&[]);
        assert!(dxf_engine::DxfLinter::is_valid(&dxf));
    }

    // ================================================================
    // Roundtrip text/line count preservation
    // ================================================================

    #[test]
    fn test_roundtrip_line_count_matches() {
        let stations = sample_stations();
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let (lines, _texts) = geometry_to_dxf(&geometry);
        let dxf = geometry_to_dxf_string(&geometry);
        let doc = dxf_engine::parse_dxf(&dxf).unwrap();
        assert_eq!(doc.lines.len(), lines.len(),
            "Roundtrip should preserve line count: wrote {}, read {}", lines.len(), doc.lines.len());
    }

    #[test]
    fn test_roundtrip_text_count_matches() {
        let stations = sample_stations();
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let (_lines, texts) = geometry_to_dxf(&geometry);
        let dxf = geometry_to_dxf_string(&geometry);
        let doc = dxf_engine::parse_dxf(&dxf).unwrap();
        assert_eq!(doc.texts.len(), texts.len(),
            "Roundtrip should preserve text count: wrote {}, read {}", texts.len(), doc.texts.len());
    }

    #[test]
    fn test_roundtrip_station_name_text_preserved() {
        let dxf = stations_to_dxf(&sample_stations());
        let doc = dxf_engine::parse_dxf(&dxf).unwrap();
        let names: Vec<&str> = doc.texts.iter().map(|t| t.text.as_str()).collect();
        assert!(names.contains(&"No.0"), "Should contain station name No.0, got {:?}", names);
        assert!(names.contains(&"No.1"));
        assert!(names.contains(&"No.2"));
    }

    // ================================================================
    // Coordinate precision after roundtrip
    // ================================================================

    #[test]
    fn test_roundtrip_coordinate_precision() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.5, 2.5),
            StationData::new("No.1", 20.0, 2.5, 2.5),
        ];
        let config = RoadSectionConfig::default();
        let geometry = calculate_road_section(&stations, &config);
        let (orig_lines, _) = geometry_to_dxf(&geometry);
        let dxf = geometry_to_dxf_string(&geometry);
        let doc = dxf_engine::parse_dxf(&dxf).unwrap();

        // Compare first line coordinates
        let orig = &orig_lines[0];
        let read = &doc.lines[0];
        assert!((orig.x1 - read.x1).abs() < 0.01,
            "x1 drift: orig={}, read={}", orig.x1, read.x1);
        assert!((orig.y1 - read.y1).abs() < 0.01,
            "y1 drift: orig={}, read={}", orig.y1, read.y1);
    }

    // ================================================================
    // Custom scale affects coordinates
    // ================================================================

    #[test]
    fn test_scale_500_halves_coordinates() {
        let stations = vec![
            StationData::new("No.0", 0.0, 2.0, 2.0),
            StationData::new("No.1", 10.0, 2.0, 2.0),
        ];
        let dxf_1000 = stations_to_dxf(&stations);
        let dxf_500 = stations_to_dxf_with_scale(&stations, 500.0);

        let doc_1000 = dxf_engine::parse_dxf(&dxf_1000).unwrap();
        let doc_500 = dxf_engine::parse_dxf(&dxf_500).unwrap();

        // Find the connecting center line (x changes, y=0)
        let center_1000 = doc_1000.lines.iter()
            .find(|l| (l.y1).abs() < 0.01 && (l.y2).abs() < 0.01 && (l.x2 - l.x1).abs() > 100.0);
        let center_500 = doc_500.lines.iter()
            .find(|l| (l.y1).abs() < 0.01 && (l.y2).abs() < 0.01 && (l.x2 - l.x1).abs() > 100.0);

        assert!(center_1000.is_some() && center_500.is_some(),
            "Both should have a center line");
        let len_1000 = (center_1000.unwrap().x2 - center_1000.unwrap().x1).abs();
        let len_500 = (center_500.unwrap().x2 - center_500.unwrap().x1).abs();
        assert!((len_1000 / len_500 - 2.0).abs() < 0.01,
            "scale=1000 center line should be 2x scale=500: {} vs {}", len_1000, len_500);
    }

    // ================================================================
    // Single station export
    // ================================================================

    #[test]
    fn test_single_station_export_valid() {
        let stations = vec![StationData::new("No.0", 0.0, 3.0, 3.0)];
        let dxf = stations_to_dxf(&stations);
        assert!(dxf_engine::DxfLinter::is_valid(&dxf));
        let doc = dxf_engine::parse_dxf(&dxf).unwrap();
        assert_eq!(doc.lines.len(), 2, "Single station: 2 width lines");
        let has_name = doc.texts.iter().any(|t| t.text == "No.0");
        assert!(has_name, "Single station should have name text");
    }

    #[test]
    fn test_many_stations_export_valid() {
        let stations: Vec<StationData> = (0..20)
            .map(|i| StationData::new(&format!("No.{}", i), i as f64 * 20.0, 2.5, 2.5))
            .collect();
        let dxf = stations_to_dxf(&stations);
        assert!(dxf_engine::DxfLinter::is_valid(&dxf));
        let doc = dxf_engine::parse_dxf(&dxf).unwrap();
        // 20 stations: 20×2 width + 19×3 connecting = 97 lines
        assert_eq!(doc.lines.len(), 97, "20 stations should produce 97 lines");
    }
}
