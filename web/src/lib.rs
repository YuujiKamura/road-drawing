//! Road Drawing Web — egui WASM entry point
//!
//! Provides browser-based CSV/Excel drag-and-drop → DXF preview → download.

mod app;
pub mod dxf_export;
#[cfg(not(target_arch = "wasm32"))]
pub mod dxf_viewer;
pub mod grid_data;
pub mod renderer;

pub use app::RoadDrawingApp;
#[cfg(not(target_arch = "wasm32"))]
pub use dxf_viewer::DxfViewerApp;

// WASM entry point
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Shared state for JS↔WASM bridge.
/// Grid edits in Tabulator JS → CSV string → this cell → egui reads it.
#[cfg(target_arch = "wasm32")]
static CSV_CELL: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Called from JS when Tabulator grid is edited.
/// Stores CSV text for egui to pick up on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_update_csv(csv: &str) {
    if let Ok(mut cell) = CSV_CELL.lock() {
        *cell = Some(csv.to_string());
    }
}

/// Called from JS "DXF" button. Generates DXF and triggers browser download.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_download_dxf() {
    let csv = match CSV_CELL.lock() {
        Ok(cell) => cell.clone(),
        Err(_) => None,
    };
    let Some(csv) = csv else { return };

    let rows = grid_data::csv_to_grid(&csv);
    let stations = grid_data::grid_to_stations(&rows);
    if stations.is_empty() { return; }

    let dxf_content = dxf_export::stations_to_dxf(&stations);

    // Trigger browser download via Blob + <a> click
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(&dxf_content));
    let mut opts = web_sys::BlobPropertyBag::new();
    opts.set_type("application/dxf");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&array, &opts).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
    let a: web_sys::HtmlAnchorElement = document
        .create_element("a").unwrap()
        .dyn_into().unwrap();
    a.set_href(&url);
    a.set_download("road_section.dxf");
    a.click();
    web_sys::Url::revoke_object_url(&url).ok();
}

/// Take pending CSV from JS bridge (returns None if no update).
#[cfg(target_arch = "wasm32")]
pub fn take_pending_csv() -> Option<String> {
    CSV_CELL.lock().ok().and_then(|mut cell| cell.take())
}

/// Push CSV to JS grid (called when file is dropped on egui canvas).
#[cfg(target_arch = "wasm32")]
pub fn push_csv_to_js_grid(csv: &str) {
    let window = web_sys::window().unwrap();
    let _ = js_sys::Reflect::get(&window, &JsValue::from_str("js_load_csv_to_grid"))
        .ok()
        .and_then(|f| f.dyn_ref::<js_sys::Function>().cloned())
        .map(|f| f.call1(&JsValue::NULL, &JsValue::from_str(csv)));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug)
        .expect("failed to init logger");

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("road_drawing_canvas")
            .expect("canvas element 'road_drawing_canvas' not found");
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into()
            .expect("element is not a canvas");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(RoadDrawingApp::new(cc)))),
            )
            .await;
        match &start_result {
            Ok(_) => {
                // Hide loading spinner via setAttribute (no extra web-sys features needed)
                if let Some(el) = document.get_element_by_id("loading") {
                    let _ = el.set_attribute("class", "hidden");
                }
                log::info!("eframe started successfully");
            }
            Err(e) => {
                log::error!("Failed to start eframe: {e:?}");
            }
        }
    });

    Ok(())
}

// Native entry point (for development)
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("Road Drawing"),
        ..Default::default()
    };
    eframe::run_native(
        "Road Drawing",
        options,
        Box::new(|cc| Ok(Box::new(RoadDrawingApp::new(cc)))),
    )
}
