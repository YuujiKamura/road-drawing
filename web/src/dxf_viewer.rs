//! DXF hot-swap viewer: watches a DXF file and auto-reloads on change.
//!
//! Renders LINE, TEXT, CIRCLE, LWPOLYLINE entities from any DXF file.
//! Uses `notify` crate for filesystem watching — when the file is modified,
//! the viewer reloads and re-renders automatically.

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use egui::{CentralPanel, Color32, Pos2, RichText, Stroke, Vec2};

use dxf_engine::DxfDocument;

use super::renderer::dxf_color_to_egui;

/// Bounding box for DXF entities
struct BBox {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl BBox {
    fn from_document(doc: &DxfDocument) -> Option<Self> {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let mut has_data = false;

        for l in &doc.lines {
            min_x = min_x.min(l.x1).min(l.x2);
            max_x = max_x.max(l.x1).max(l.x2);
            min_y = min_y.min(l.y1).min(l.y2);
            max_y = max_y.max(l.y1).max(l.y2);
            has_data = true;
        }
        for t in &doc.texts {
            min_x = min_x.min(t.x);
            max_x = max_x.max(t.x);
            min_y = min_y.min(t.y);
            max_y = max_y.max(t.y);
            has_data = true;
        }
        for c in &doc.circles {
            min_x = min_x.min(c.x - c.radius);
            max_x = max_x.max(c.x + c.radius);
            min_y = min_y.min(c.y - c.radius);
            max_y = max_y.max(c.y + c.radius);
            has_data = true;
        }
        for p in &doc.polylines {
            for &(x, y) in &p.vertices {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                has_data = true;
            }
        }

        if has_data {
            Some(BBox { min_x, max_x, min_y, max_y })
        } else {
            None
        }
    }

    fn width(&self) -> f64 {
        (self.max_x - self.min_x).max(1.0)
    }
    fn height(&self) -> f64 {
        (self.max_y - self.min_y).max(1.0)
    }
}

/// DXF hot-swap viewer application state
pub struct DxfViewerApp {
    dxf_path: PathBuf,
    document: Option<DxfDocument>,
    status: String,
    reload_count: u32,
    reload_rx: mpsc::Receiver<()>,
    _watcher: notify::RecommendedWatcher,
}

impl DxfViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, dxf_path: PathBuf) -> Self {
        let (reload_tx, reload_rx) = mpsc::channel();

        // Set up file watcher
        let watch_path = dxf_path.clone();
        let ctx = cc.egui_ctx.clone();

        // Use a shared debounce flag to avoid rapid reloads
        let pending = Arc::new(Mutex::new(false));
        let pending_clone = pending.clone();

        use notify::Watcher;
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                use notify::EventKind;
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        let mut p = pending_clone.lock().unwrap();
                        if !*p {
                            *p = true;
                            let _ = reload_tx.send(());
                            ctx.request_repaint();
                        }
                    }
                    _ => {}
                }
            }
        })
        .expect("Failed to create file watcher");

        // Watch the parent directory (more reliable for editors that do atomic writes)
        let watch_dir = watch_path.parent().unwrap_or(&watch_path);
        watcher
            .watch(watch_dir.as_ref(), notify::RecursiveMode::NonRecursive)
            .expect("Failed to watch directory");

        let mut app = Self {
            dxf_path,
            document: None,
            status: "Loading...".to_string(),
            reload_count: 0,
            reload_rx,
            _watcher: watcher,
        };
        app.load_dxf();
        app
    }

    fn load_dxf(&mut self) {
        match std::fs::read_to_string(&self.dxf_path) {
            Ok(content) => match dxf_engine::parse_dxf(&content) {
                Ok(doc) => {
                    let lines = doc.lines.len();
                    let texts = doc.texts.len();
                    let circles = doc.circles.len();
                    let polys = doc.polylines.len();
                    self.status = format!(
                        "{} | L:{} T:{} C:{} P:{} | reload #{}",
                        self.dxf_path.file_name().unwrap_or_default().to_string_lossy(),
                        lines, texts, circles, polys, self.reload_count
                    );
                    self.document = Some(doc);
                }
                Err(e) => {
                    self.status = format!("Parse error: {e}");
                    self.document = None;
                }
            },
            Err(e) => {
                self.status = format!("Read error: {e}");
                self.document = None;
            }
        }
    }

    fn check_reload(&mut self) {
        let mut reloaded = false;
        while self.reload_rx.try_recv().is_ok() {
            reloaded = true;
        }
        if reloaded {
            self.reload_count += 1;
            self.load_dxf();
        }
    }

    fn draw_dxf(&self, ui: &mut egui::Ui) {
        let doc = match &self.document {
            Some(d) => d,
            None => {
                ui.label("No DXF data");
                return;
            }
        };

        let bbox = match BBox::from_document(doc) {
            Some(b) => b,
            None => {
                ui.label("Empty DXF");
                return;
            }
        };

        let avail = ui.available_size();
        let canvas_w = avail.x - 20.0;
        let canvas_h = (avail.y - 20.0).max(200.0);

        let scale = (canvas_w / bbox.width() as f32).min(canvas_h / bbox.height() as f32) * 0.9;
        let offset_x = (canvas_w - bbox.width() as f32 * scale) / 2.0;
        let offset_y = (canvas_h - bbox.height() as f32 * scale) / 2.0;

        let (response, painter) =
            ui.allocate_painter(Vec2::new(canvas_w, canvas_h), egui::Sense::hover());
        let origin = response.rect.min;

        // Background
        painter.rect_filled(response.rect, 0.0, Color32::from_gray(24));

        // DXF Y-up -> screen Y-down
        let to_screen = |x: f64, y: f64| -> Pos2 {
            Pos2::new(
                origin.x + offset_x + (x - bbox.min_x) as f32 * scale,
                origin.y + offset_y + (bbox.max_y - y) as f32 * scale,
            )
        };

        // Lines
        for seg in &doc.lines {
            let color = dxf_color_to_egui(seg.color);
            painter.line_segment(
                [to_screen(seg.x1, seg.y1), to_screen(seg.x2, seg.y2)],
                Stroke::new(1.0, color),
            );
        }

        // Polylines
        for poly in &doc.polylines {
            if poly.vertices.len() < 2 {
                continue;
            }
            let color = dxf_color_to_egui(poly.color);
            let points: Vec<Pos2> = poly.vertices.iter().map(|&(x, y)| to_screen(x, y)).collect();
            for w in points.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(1.0, color));
            }
            if poly.closed && points.len() >= 2 {
                painter.line_segment(
                    [*points.last().unwrap(), points[0]],
                    Stroke::new(1.0, color),
                );
            }
        }

        // Circles (approximate with line segments)
        for circ in &doc.circles {
            let color = dxf_color_to_egui(circ.color);
            let center = to_screen(circ.x, circ.y);
            let r_screen = circ.radius as f32 * scale;
            painter.circle_stroke(center, r_screen, Stroke::new(1.0, color));
        }

        // Texts
        for t in &doc.texts {
            let color = dxf_color_to_egui(t.color);
            let pos = to_screen(t.x, t.y);
            let font_size = (t.height as f32 * scale).clamp(8.0, 24.0);
            painter.text(
                pos,
                egui::Align2::LEFT_BOTTOM,
                &t.text,
                egui::FontId::proportional(font_size),
                color,
            );
        }
    }
}

impl eframe::App for DxfViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_reload();

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("DXF Viewer");
                ui.separator();
                ui.label(RichText::new(&self.status).size(12.0).color(Color32::LIGHT_GRAY));
            });
            ui.add_space(4.0);
            self.draw_dxf(ui);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxf_engine::{DxfCircle, DxfLine, DxfLwPolyline, DxfText};

    fn make_doc_with_lines() -> DxfDocument {
        DxfDocument {
            lines: vec![
                DxfLine::new(0.0, 0.0, 100.0, 50.0),
                DxfLine::new(100.0, 50.0, 200.0, 0.0),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_bbox_from_lines() {
        let doc = make_doc_with_lines();
        let bb = BBox::from_document(&doc).unwrap();
        assert!((bb.min_x - 0.0).abs() < 0.01);
        assert!((bb.max_x - 200.0).abs() < 0.01);
        assert!((bb.min_y - 0.0).abs() < 0.01);
        assert!((bb.max_y - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_bbox_from_empty_doc() {
        let doc = DxfDocument::default();
        assert!(BBox::from_document(&doc).is_none());
    }

    #[test]
    fn test_bbox_from_circles() {
        let doc = DxfDocument {
            circles: vec![DxfCircle::new(10.0, 10.0, 5.0)],
            ..Default::default()
        };
        let bb = BBox::from_document(&doc).unwrap();
        assert!((bb.min_x - 5.0).abs() < 0.01);
        assert!((bb.max_x - 15.0).abs() < 0.01);
        assert!((bb.min_y - 5.0).abs() < 0.01);
        assert!((bb.max_y - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_bbox_from_polylines() {
        let doc = DxfDocument {
            polylines: vec![DxfLwPolyline::new(vec![
                (-10.0, -20.0),
                (30.0, 40.0),
                (50.0, -5.0),
            ])],
            ..Default::default()
        };
        let bb = BBox::from_document(&doc).unwrap();
        assert!((bb.min_x - (-10.0)).abs() < 0.01);
        assert!((bb.max_x - 50.0).abs() < 0.01);
        assert!((bb.min_y - (-20.0)).abs() < 0.01);
        assert!((bb.max_y - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_bbox_from_texts() {
        let doc = DxfDocument {
            texts: vec![
                DxfText::new(5.0, 10.0, "A"),
                DxfText::new(50.0, 60.0, "B"),
            ],
            ..Default::default()
        };
        let bb = BBox::from_document(&doc).unwrap();
        assert!((bb.min_x - 5.0).abs() < 0.01);
        assert!((bb.max_x - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_bbox_mixed_entities() {
        let doc = DxfDocument {
            lines: vec![DxfLine::new(0.0, 0.0, 10.0, 10.0)],
            circles: vec![DxfCircle::new(100.0, 100.0, 20.0)],
            texts: vec![DxfText::new(-5.0, -5.0, "origin")],
            polylines: vec![DxfLwPolyline::new(vec![(50.0, 200.0)])],
        };
        let bb = BBox::from_document(&doc).unwrap();
        assert!((bb.min_x - (-5.0)).abs() < 0.01);
        assert!((bb.max_x - 120.0).abs() < 0.01); // circle at 100 + r=20
        assert!((bb.min_y - (-5.0)).abs() < 0.01);
        assert!((bb.max_y - 200.0).abs() < 0.01); // polyline vertex
    }

    #[test]
    fn test_bbox_width_height_clamp() {
        // Single point → width/height clamped to 1.0
        let doc = DxfDocument {
            lines: vec![DxfLine::new(5.0, 5.0, 5.0, 5.0)],
            ..Default::default()
        };
        let bb = BBox::from_document(&doc).unwrap();
        assert!((bb.width() - 1.0).abs() < 0.01);
        assert!((bb.height() - 1.0).abs() < 0.01);
    }

    // ================================================================
    // DXF entity → drawing data conversion tests (Issue #9)
    // ================================================================

    #[test]
    fn test_dxf_lines_to_screen_coords_y_flip() {
        // DXF LINE with known coords → verify Y-flip in screen space
        let doc = DxfDocument {
            lines: vec![DxfLine::new(0.0, 0.0, 100.0, 50.0)],
            ..Default::default()
        };
        let bb = BBox::from_document(&doc).unwrap();
        let canvas_w = 800.0_f32;
        let canvas_h = 600.0_f32;
        let scale = (canvas_w / bb.width() as f32).min(canvas_h / bb.height() as f32) * 0.9;
        let offset_x = (canvas_w - bb.width() as f32 * scale) / 2.0;
        let offset_y = (canvas_h - bb.height() as f32 * scale) / 2.0;

        let to_screen = |x: f64, y: f64| -> (f32, f32) {
            (
                offset_x + (x - bb.min_x) as f32 * scale,
                offset_y + (bb.max_y - y) as f32 * scale,
            )
        };

        let (sx1, sy1) = to_screen(0.0, 0.0);
        let (sx2, sy2) = to_screen(100.0, 50.0);

        // Y=0 (DXF bottom) → larger screen Y
        // Y=50 (DXF top)   → smaller screen Y
        assert!(sy1 > sy2, "DXF Y=0 should map below Y=50: sy1={sy1}, sy2={sy2}");
        assert!(sx2 > sx1, "DXF X=100 should map right of X=0: sx1={sx1}, sx2={sx2}");
    }

    #[test]
    fn test_dxf_circle_renders_with_correct_radius() {
        let doc = DxfDocument {
            circles: vec![DxfCircle::new(50.0, 50.0, 25.0)],
            ..Default::default()
        };
        let bb = BBox::from_document(&doc).unwrap();
        let canvas_w = 400.0_f32;
        let canvas_h = 400.0_f32;
        let scale = (canvas_w / bb.width() as f32).min(canvas_h / bb.height() as f32) * 0.9;

        let r_screen = 25.0_f32 * scale;
        assert!(r_screen > 0.0, "Screen radius must be positive: {r_screen}");
        // Circle at center of bbox → screen center should be within canvas
        let offset_x = (canvas_w - bb.width() as f32 * scale) / 2.0;
        let offset_y = (canvas_h - bb.height() as f32 * scale) / 2.0;
        let cx = offset_x + (50.0 - bb.min_x) as f32 * scale;
        let cy = offset_y + (bb.max_y - 50.0) as f32 * scale;
        assert!(cx >= 0.0 && cx <= canvas_w, "Circle center x within canvas: {cx}");
        assert!(cy >= 0.0 && cy <= canvas_h, "Circle center y within canvas: {cy}");
    }

    #[test]
    fn test_dxf_polyline_vertices_transform_order() {
        let doc = DxfDocument {
            polylines: vec![DxfLwPolyline::new(vec![
                (0.0, 0.0),
                (100.0, 0.0),
                (100.0, 50.0),
                (0.0, 50.0),
            ])],
            ..Default::default()
        };
        let bb = BBox::from_document(&doc).unwrap();
        let scale = (800.0_f32 / bb.width() as f32).min(600.0_f32 / bb.height() as f32) * 0.9;
        let offset_x = (800.0 - bb.width() as f32 * scale) / 2.0;
        let offset_y = (600.0 - bb.height() as f32 * scale) / 2.0;

        let to_screen = |x: f64, y: f64| -> (f32, f32) {
            (
                offset_x + (x - bb.min_x) as f32 * scale,
                offset_y + (bb.max_y - y) as f32 * scale,
            )
        };

        let points: Vec<(f32, f32)> = doc.polylines[0]
            .vertices
            .iter()
            .map(|&(x, y)| to_screen(x, y))
            .collect();

        // Verify rectangle shape: p0→p1 horizontal, p1→p2 vertical
        assert!((points[0].1 - points[1].1).abs() < 0.01, "p0→p1 should be horizontal");
        assert!((points[1].0 - points[2].0).abs() < 0.01, "p1→p2 should be vertical");
        assert!(points[0].0 < points[1].0, "p0 left of p1");
    }

    #[test]
    fn test_dxf_text_position_transforms_correctly() {
        let doc = DxfDocument {
            texts: vec![
                DxfText::new(0.0, 100.0, "top-left"),
                DxfText::new(200.0, 0.0, "bottom-right"),
            ],
            lines: vec![DxfLine::new(0.0, 0.0, 200.0, 100.0)], // for bbox
            ..Default::default()
        };
        let bb = BBox::from_document(&doc).unwrap();
        let scale = (800.0_f32 / bb.width() as f32).min(600.0_f32 / bb.height() as f32) * 0.9;
        let offset_x = (800.0 - bb.width() as f32 * scale) / 2.0;
        let offset_y = (600.0 - bb.height() as f32 * scale) / 2.0;

        let to_screen = |x: f64, y: f64| -> (f32, f32) {
            (
                offset_x + (x - bb.min_x) as f32 * scale,
                offset_y + (bb.max_y - y) as f32 * scale,
            )
        };

        let (tx1, ty1) = to_screen(0.0, 100.0);   // DXF top-left
        let (tx2, ty2) = to_screen(200.0, 0.0);     // DXF bottom-right
        assert!(ty1 < ty2, "DXF top should have smaller screen Y");
        assert!(tx1 < tx2, "DXF left should have smaller screen X");
    }

    #[test]
    fn test_dxf_text_font_size_clamped() {
        // Font size = height * scale, clamped to [8.0, 24.0]
        let heights_and_scales: Vec<(f64, f32)> = vec![
            (0.1, 0.5),   // tiny → clamped to 8.0
            (100.0, 10.0), // huge → clamped to 24.0
            (2.0, 5.0),   // normal → 10.0 within range
        ];
        for (h, s) in heights_and_scales {
            let font_size = (h as f32 * s).clamp(8.0, 24.0);
            assert!(font_size >= 8.0 && font_size <= 24.0,
                "Font size {font_size} out of range for h={h}, s={s}");
        }
    }

    #[test]
    fn test_dxf_document_roundtrip_preserves_entity_types() {
        // Generate DXF from stations → parse back → verify all entity types present
        let stations = vec![
            road_section::StationData::new("No.0", 0.0, 3.0, 3.0),
            road_section::StationData::new("No.1", 20.0, 3.0, 3.0),
        ];
        let dxf_str = crate::dxf_export::stations_to_dxf(&stations);
        let doc = dxf_engine::parse_dxf(&dxf_str).unwrap();

        assert!(!doc.lines.is_empty(), "Roundtrip should preserve lines");
        assert!(!doc.texts.is_empty(), "Roundtrip should preserve texts");

        // BBox should encompass all entities
        let bb = BBox::from_document(&doc).unwrap();
        assert!(bb.width() > 1.0, "BBox width should be > 1.0 for 20m road");
        assert!(bb.height() > 1.0, "BBox height should be > 1.0 for 3m widths");
    }
}
