//! File change detection tests for the DXF hot-swap viewer (Issue #9).
//!
//! Tests that FileWatcher fires events when a temp DXF file is modified.
//! Does NOT require egui — tests the watcher mechanism in isolation.

use std::time::Duration;

use file_watcher::FileWatcher;

#[test]
fn test_watcher_detects_file_creation() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.dxf");
    // FileWatcher needs the parent to exist, file can be absent
    let w = FileWatcher::new(&file_path).unwrap();

    std::fs::write(&file_path, "initial content").unwrap();
    std::thread::sleep(Duration::from_millis(500));

    assert!(w.check_changed(), "Should detect file creation");
}

#[test]
fn test_watcher_detects_file_modification() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test.dxf");
    std::fs::write(&file_path, "initial").unwrap();

    let w = FileWatcher::new(&file_path).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    // Drain initial flag
    let _ = w.check_changed();

    std::fs::write(&file_path, "modified content").unwrap();
    std::thread::sleep(Duration::from_millis(500));

    assert!(w.check_changed(), "Should detect file modification");
}

#[test]
fn test_watcher_fires_on_dxf_overwrite() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("road.dxf");
    let dxf_v1 = "0\nSECTION\n2\nENTITIES\n0\nEOF\n";
    std::fs::write(&file_path, dxf_v1).unwrap();

    let w = FileWatcher::new(&file_path).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let _ = w.check_changed();

    // Overwrite with generated DXF
    let stations = vec![
        road_section::StationData::new("No.0", 0.0, 2.5, 2.5),
        road_section::StationData::new("No.1", 20.0, 2.5, 2.5),
    ];
    let dxf_v2 = road_drawing_web::dxf_export::stations_to_dxf(&stations);
    std::fs::write(&file_path, &dxf_v2).unwrap();
    std::thread::sleep(Duration::from_millis(500));

    assert!(w.check_changed(), "Should detect DXF file overwrite");

    let content = std::fs::read_to_string(&file_path).unwrap();
    let doc = dxf_engine::parse_dxf(&content).unwrap();
    assert!(!doc.lines.is_empty(), "Overwritten DXF should have lines");
}

#[test]
fn test_watcher_multiple_rapid_writes() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("rapid.dxf");
    std::fs::write(&file_path, "v0").unwrap();

    let w = FileWatcher::new(&file_path).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let _ = w.check_changed();

    for i in 1..=5 {
        std::fs::write(&file_path, format!("version {i}")).unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(500));

    assert!(w.check_changed(), "Should detect at least one rapid write");
}
