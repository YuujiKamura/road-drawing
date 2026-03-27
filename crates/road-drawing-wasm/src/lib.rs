use wasm_bindgen::prelude::*;
use serde::Serialize;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Parse CSV text, return JSON array of {name, x, wl, wr}
#[wasm_bindgen]
pub fn parse_csv(csv_text: &str) -> String {
    match road_section::parse_road_section_csv(csv_text) {
        Ok(stations) => {
            let rows: Vec<StationRow> = stations
                .iter()
                .map(|s| StationRow {
                    name: s.name.clone(),
                    x: s.x,
                    wl: s.wl,
                    wr: s.wr,
                })
                .collect();
            serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into())
        }
        Err(e) => format!(r#"{{"error":"{}"}}"#, e),
    }
}

/// Generate DXF string from CSV text
#[wasm_bindgen]
pub fn generate_dxf(csv_text: &str) -> String {
    let stations = match road_section::parse_road_section_csv(csv_text) {
        Ok(s) => s,
        Err(e) => return format!("ERROR: {}", e),
    };
    let config = road_section::RoadSectionConfig::default();
    let geometry = road_section::calculate_road_section(&stations, &config);
    let (lines, texts) = road_section::geometry_to_dxf(&geometry);
    let writer = dxf_engine::DxfWriter::new();
    writer.write(&lines, &texts)
}

/// Get preview data as JSON for Canvas rendering
#[wasm_bindgen]
pub fn get_preview_data(csv_text: &str) -> String {
    let stations = match road_section::parse_road_section_csv(csv_text) {
        Ok(s) => s,
        Err(_) => return r#"{"lines":[],"texts":[]}"#.into(),
    };
    let config = road_section::RoadSectionConfig::default();
    let geometry = road_section::calculate_road_section(&stations, &config);

    let lines: Vec<PreviewLine> = geometry
        .lines
        .iter()
        .map(|l| PreviewLine {
            x1: l.x1,
            y1: l.y1,
            x2: l.x2,
            y2: l.y2,
            color: l.color,
        })
        .collect();
    let texts: Vec<PreviewText> = geometry
        .texts
        .iter()
        .map(|t| PreviewText {
            text: t.text.clone(),
            x: t.x,
            y: t.y,
            rotation: t.rotation,
            height: t.height,
            color: t.color,
        })
        .collect();

    serde_json::to_string(&PreviewData { lines, texts })
        .unwrap_or_else(|_| r#"{"lines":[],"texts":[]}"#.into())
}

#[derive(Serialize)]
struct StationRow {
    name: String,
    x: f64,
    wl: f64,
    wr: f64,
}

#[derive(Serialize)]
struct PreviewData {
    lines: Vec<PreviewLine>,
    texts: Vec<PreviewText>,
}

#[derive(Serialize)]
struct PreviewLine {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: i32,
}

#[derive(Serialize)]
struct PreviewText {
    text: String,
    x: f64,
    y: f64,
    rotation: f64,
    height: f64,
    color: i32,
}
