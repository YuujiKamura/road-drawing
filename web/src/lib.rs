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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug)
        .expect("failed to init logger");

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        // eframe 0.29 requires HtmlCanvasElement, not a string ID
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
        if let Err(e) = start_result {
            log::error!("Failed to start eframe: {e:?}");
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
