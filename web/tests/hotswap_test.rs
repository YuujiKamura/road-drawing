//! File change detection tests for the DXF hot-swap viewer (Issue #9).
//!
//! Tests that notify watcher fires events when a temp DXF file is modified.
//! Does NOT require egui — tests the watcher mechanism in isolation.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Set up a file watcher on a temp directory, return (watcher, receiver, temp_dir).
fn setup_watcher() -> (RecommendedWatcher, mpsc::Receiver<Event>, tempfile::TempDir) {
    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(Mutex::new(tx));

    let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let _ = tx.lock().unwrap().send(event);
        }
    })
    .expect("Failed to create watcher");

    (watcher, rx, tmp_dir)
}

#[test]
fn test_watcher_detects_file_creation() {
    let (mut watcher, rx, tmp_dir) = setup_watcher();
    watcher
        .watch(tmp_dir.path(), RecursiveMode::NonRecursive)
        .expect("Failed to watch");

    // Create a new file
    let file_path = tmp_dir.path().join("test.dxf");
    std::fs::write(&file_path, "initial content").unwrap();

    // Wait for event (up to 2s)
    let event = rx.recv_timeout(Duration::from_secs(2));
    assert!(event.is_ok(), "Should receive file creation event");
    let event = event.unwrap();
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {} // OK
        other => panic!("Expected Create or Modify event, got: {other:?}"),
    }
}

#[test]
fn test_watcher_detects_file_modification() {
    let (mut watcher, rx, tmp_dir) = setup_watcher();

    // Create file first
    let file_path = tmp_dir.path().join("test.dxf");
    std::fs::write(&file_path, "initial").unwrap();

    watcher
        .watch(tmp_dir.path(), RecursiveMode::NonRecursive)
        .expect("Failed to watch");

    // Drain any creation events
    std::thread::sleep(Duration::from_millis(100));
    while rx.try_recv().is_ok() {}

    // Modify the file
    std::fs::write(&file_path, "modified content").unwrap();

    let event = rx.recv_timeout(Duration::from_secs(2));
    assert!(event.is_ok(), "Should receive file modification event");
}

#[test]
fn test_watcher_fires_on_dxf_overwrite() {
    let (mut watcher, rx, tmp_dir) = setup_watcher();

    // Write a minimal DXF
    let file_path = tmp_dir.path().join("road.dxf");
    let dxf_v1 = "0\nSECTION\n2\nENTITIES\n0\nEOF\n";
    std::fs::write(&file_path, dxf_v1).unwrap();

    watcher
        .watch(tmp_dir.path(), RecursiveMode::NonRecursive)
        .expect("Failed to watch");

    std::thread::sleep(Duration::from_millis(100));
    while rx.try_recv().is_ok() {}

    // Overwrite with updated DXF (simulating regeneration)
    let stations = vec![
        road_section::StationData::new("No.0", 0.0, 2.5, 2.5),
        road_section::StationData::new("No.1", 20.0, 2.5, 2.5),
    ];
    let dxf_v2 = road_drawing_web::dxf_export::stations_to_dxf(&stations);
    std::fs::write(&file_path, &dxf_v2).unwrap();

    let event = rx.recv_timeout(Duration::from_secs(2));
    assert!(event.is_ok(), "Should detect DXF file overwrite");

    // Verify the new content is parseable
    let content = std::fs::read_to_string(&file_path).unwrap();
    let doc = dxf_engine::parse_dxf(&content).unwrap();
    assert!(!doc.lines.is_empty(), "Overwritten DXF should have lines");
}

#[test]
fn test_watcher_multiple_rapid_writes() {
    let (mut watcher, rx, tmp_dir) = setup_watcher();
    let file_path = tmp_dir.path().join("rapid.dxf");
    std::fs::write(&file_path, "v0").unwrap();

    watcher
        .watch(tmp_dir.path(), RecursiveMode::NonRecursive)
        .expect("Failed to watch");

    std::thread::sleep(Duration::from_millis(100));
    while rx.try_recv().is_ok() {}

    // Rapid writes (simulating editor saves)
    for i in 1..=5 {
        std::fs::write(&file_path, format!("version {i}")).unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }

    // Should receive at least one event
    let mut count = 0;
    while rx.recv_timeout(Duration::from_secs(1)).is_ok() {
        count += 1;
    }
    assert!(count >= 1, "Should receive at least 1 event for rapid writes, got {count}");
}
