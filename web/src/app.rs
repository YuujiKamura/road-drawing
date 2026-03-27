//! Main egui application for road drawing web viewer.
//!
//! Supports:
//! - File drag-and-drop (CSV/Excel)
//! - Road section preview rendering
//! - DXF download

use egui::{CentralPanel, Color32, RichText, Stroke, Vec2};

/// Application state
pub struct RoadDrawingApp {
    /// Loaded file name
    file_name: Option<String>,
    /// Loaded CSV content
    csv_content: Option<String>,
    /// Parsed station data
    stations: Vec<road_section::StationData>,
    /// Status message
    status: String,
}

impl RoadDrawingApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            file_name: None,
            csv_content: None,
            stations: Vec::new(),
            status: "CSVファイルをドロップしてください".to_string(),
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for file in &dropped {
            if let Some(bytes) = &file.bytes {
                let name = file.name.clone();
                log::info!("Dropped file: {name} ({} bytes)", bytes.len());

                // Try UTF-8 first, then Shift_JIS
                let text = match std::str::from_utf8(bytes) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
                        decoded.into_owned()
                    }
                };

                match road_section::parse_road_section_csv(&text) {
                    Ok(stations) => {
                        self.status = format!("{}を読み込みました（{}測点）", name, stations.len());
                        self.stations = stations;
                        self.csv_content = Some(text.clone());
                        self.file_name = Some(name);
                        // Sync dropped CSV to Tabulator grid
                        #[cfg(target_arch = "wasm32")]
                        crate::push_csv_to_js_grid(&text);
                    }
                    Err(e) => {
                        self.status = format!("パースエラー: {e}");
                        log::error!("Parse error for {name}: {e}");
                    }
                }
            }
        }
    }

    fn draw_drop_zone(&self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let zone_size = Vec2::new(rect.width().min(400.0), 200.0);
        let (response, painter) = ui.allocate_painter(zone_size, egui::Sense::hover());
        let rect = response.rect;

        // Dashed border
        let stroke = Stroke::new(2.0, Color32::from_gray(128));
        painter.rect_stroke(rect, 8.0, stroke);

        // Drop icon and text
        painter.text(
            rect.center() - Vec2::new(0.0, 20.0),
            egui::Align2::CENTER_CENTER,
            "📁",
            egui::FontId::proportional(48.0),
            Color32::from_gray(128),
        );
        painter.text(
            rect.center() + Vec2::new(0.0, 30.0),
            egui::Align2::CENTER_CENTER,
            "CSV / Excelファイルをここにドロップ",
            egui::FontId::proportional(16.0),
            Color32::from_gray(160),
        );
    }

    fn draw_preview(&self, ui: &mut egui::Ui) {
        if self.stations.is_empty() {
            return;
        }

        let config = road_section::RoadSectionConfig::default();
        let geometry = road_section::calculate_road_section(&self.stations, &config);

        if geometry.lines.is_empty() {
            ui.label("描画データがありません");
            return;
        }

        // Calculate bounding box
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for seg in &geometry.lines {
            min_x = min_x.min(seg.x1).min(seg.x2);
            max_x = max_x.max(seg.x1).max(seg.x2);
            min_y = min_y.min(seg.y1).min(seg.y2);
            max_y = max_y.max(seg.y1).max(seg.y2);
        }

        let data_w = (max_x - min_x).max(1.0);
        let data_h = (max_y - min_y).max(1.0);

        let avail = ui.available_size();
        let canvas_w = avail.x - 20.0;
        let canvas_h = (avail.y - 40.0).max(200.0);

        let scale = (canvas_w / data_w as f32).min(canvas_h / data_h as f32) * 0.9;
        let offset_x = (canvas_w - data_w as f32 * scale) / 2.0;
        let offset_y = (canvas_h - data_h as f32 * scale) / 2.0;

        let (response, painter) = ui.allocate_painter(
            Vec2::new(canvas_w, canvas_h),
            egui::Sense::hover(),
        );
        let origin = response.rect.min;

        // Background
        painter.rect_filled(response.rect, 0.0, Color32::from_gray(24));

        // Transform: DXF Y-up → screen Y-down
        let to_screen = |x: f64, y: f64| -> egui::Pos2 {
            egui::Pos2::new(
                origin.x + offset_x + (x - min_x) as f32 * scale,
                origin.y + offset_y + (max_y - y) as f32 * scale, // flip Y
            )
        };

        // Draw lines
        for seg in &geometry.lines {
            let color = dxf_color_to_egui(seg.color);
            painter.line_segment(
                [to_screen(seg.x1, seg.y1), to_screen(seg.x2, seg.y2)],
                Stroke::new(1.0, color),
            );
        }

        // Draw station name labels
        for dim in &geometry.texts {
            if dim.color == 5 {
                // Blue station names
                let pos = to_screen(dim.x, dim.y);
                painter.text(
                    pos,
                    egui::Align2::CENTER_BOTTOM,
                    &dim.text,
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(0, 128, 255),
                );
            }
        }
    }
}

impl eframe::App for RoadDrawingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_dropped_files(ctx);

        // Poll JS bridge for grid edits (Tabulator → CSV → WASM)
        #[cfg(target_arch = "wasm32")]
        if let Some(csv) = crate::take_pending_csv() {
            let rows = crate::grid_data::csv_to_grid(&csv);
            let stations = crate::grid_data::grid_to_stations(&rows);
            if !stations.is_empty() {
                self.status = format!("Grid: {}測点", stations.len());
                self.stations = stations;
                self.csv_content = Some(csv);
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            if self.stations.is_empty() {
                ui.heading("Road Drawing");
                ui.label(RichText::new(&self.status).size(14.0));
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    self.draw_drop_zone(ui);
                });
            } else {
                self.draw_preview(ui);
            }
        });

        // Show file hover overlay
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let screen = ctx.screen_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("file_drop_overlay"),
            ));
            painter.rect_filled(screen, 0.0, Color32::from_rgba_premultiplied(0, 0, 0, 128));
            painter.text(
                screen.center(),
                egui::Align2::CENTER_CENTER,
                "ファイルをドロップして読み込み",
                egui::FontId::proportional(24.0),
                Color32::WHITE,
            );
        }
    }
}

/// Map DXF color index to egui Color32
fn dxf_color_to_egui(color: i32) -> Color32 {
    match color {
        1 => Color32::RED,
        2 => Color32::YELLOW,
        3 => Color32::GREEN,
        4 => Color32::from_rgb(0, 255, 255),  // cyan
        5 => Color32::from_rgb(0, 128, 255),  // blue
        6 => Color32::from_rgb(255, 0, 255),  // magenta
        7 => Color32::WHITE,
        _ => Color32::LIGHT_GRAY,
    }
}

/// Decode bytes to string: try UTF-8 first, fall back to Shift_JIS.
/// Extracted for testability.
#[cfg(test)]
fn decode_csv_bytes(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
            decoded.into_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    // decode_csv_bytes — encoding detection
    // ================================================================

    #[test]
    fn test_decode_utf8_csv() {
        let csv = "測点名,累積延長,左幅員,右幅員\nNo.1,0.0,2.5,2.5\n";
        let decoded = decode_csv_bytes(csv.as_bytes());
        assert_eq!(decoded, csv);
    }

    #[test]
    fn test_decode_ascii_csv() {
        let csv = "name,x,wl,wr\nNo.1,0.0,2.5,2.5\n";
        let decoded = decode_csv_bytes(csv.as_bytes());
        assert_eq!(decoded, csv);
    }

    #[test]
    fn test_decode_shift_jis_csv() {
        // "測点名" in Shift_JIS: 0x91AA 0x935F 0x96BC
        let sjis_bytes: Vec<u8> = {
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode("測点名,延長\nNo.1,10.0\n");
            encoded.into_owned()
        };
        let decoded = decode_csv_bytes(&sjis_bytes);
        assert!(decoded.contains("測点名"), "Should decode Shift_JIS to UTF-8");
        assert!(decoded.contains("No.1"));
    }

    #[test]
    fn test_decode_empty_bytes() {
        let decoded = decode_csv_bytes(b"");
        assert_eq!(decoded, "");
    }

    // ================================================================
    // decode + parse roundtrip
    // ================================================================

    #[test]
    fn test_decode_and_parse_utf8_csv() {
        let csv = "測点名,累積延長,左幅員,右幅員\nNo.0,0.0,2.5,2.5\nNo.1,20.0,3.0,2.0\n";
        let text = decode_csv_bytes(csv.as_bytes());
        let stations = road_section::parse_road_section_csv(&text).unwrap();
        assert_eq!(stations.len(), 2);
        assert_eq!(stations[0].name, "No.0");
        assert_eq!(stations[1].wr, 2.0);
    }

    #[test]
    fn test_decode_and_parse_shift_jis_csv() {
        let csv_utf8 = "測点名,累積延長,左幅員,右幅員\nNo.0,0.0,1.5,1.5\n";
        let (sjis_bytes, _, _) = encoding_rs::SHIFT_JIS.encode(csv_utf8);
        let text = decode_csv_bytes(&sjis_bytes);
        let stations = road_section::parse_road_section_csv(&text).unwrap();
        assert_eq!(stations.len(), 1);
        assert_eq!(stations[0].wl, 1.5);
    }

    // ================================================================
    // dxf_color_to_egui (app-local copy)
    // ================================================================

    #[test]
    fn test_app_dxf_color_all_indices() {
        assert_eq!(dxf_color_to_egui(1), Color32::RED);
        assert_eq!(dxf_color_to_egui(5), Color32::from_rgb(0, 128, 255));
        assert_eq!(dxf_color_to_egui(7), Color32::WHITE);
        assert_eq!(dxf_color_to_egui(99), Color32::LIGHT_GRAY);
    }
}
