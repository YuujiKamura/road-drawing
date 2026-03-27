//! DXF hot-swap viewer binary.
//!
//! Usage: cargo run --bin dxf-viewer -- <path-to-dxf-file>
//!
//! Watches the DXF file for changes and auto-reloads the view.

fn main() -> eframe::Result {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <dxf-file>", args[0]);
        std::process::exit(1);
    }

    let dxf_path = std::path::PathBuf::from(&args[1]);
    if !dxf_path.exists() {
        eprintln!("File not found: {}", dxf_path.display());
        std::process::exit(1);
    }

    let dxf_path_clone = dxf_path.clone();
    let title = format!("DXF Viewer - {}", dxf_path.file_name().unwrap_or_default().to_string_lossy());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title(&title),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| Ok(Box::new(road_drawing_web::DxfViewerApp::new(cc, dxf_path_clone)))),
    )
}
