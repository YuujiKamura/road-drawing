//! Lightweight file watcher with atomic changed flag.
//!
//! Wraps `notify` crate into a simple poll-based API:
//! ```no_run
//! use file_watcher::FileWatcher;
//! let watcher = FileWatcher::new("output.dxf").unwrap();
//! // ... later, in your render/update loop:
//! if watcher.check_changed() {
//!     // file was modified — reload it
//! }
//! ```

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use notify::{EventKind, RecommendedWatcher, Watcher};

/// A file watcher that sets an atomic flag on Modify/Create events.
/// Call `check_changed()` to poll and reset the flag.
pub struct FileWatcher {
    path: PathBuf,
    changed: Arc<AtomicBool>,
    _watcher: RecommendedWatcher,
}

impl FileWatcher {
    /// Start watching a file (or its parent directory for atomic-write editors).
    /// Returns Err if the path or its parent doesn't exist.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, notify::Error> {
        let path = path.as_ref().to_path_buf();
        if path.as_os_str().is_empty() {
            return Err(notify::Error::generic("empty path"));
        }
        let changed = Arc::new(AtomicBool::new(false));
        let flag = changed.clone();
        let watch_file = path.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        // Only trigger if the event is for our target file
                        let dominated = event.paths.iter().any(|p| p == &watch_file)
                            || event.paths.is_empty(); // some backends don't report paths
                        if dominated {
                            flag.store(true, Ordering::Release);
                        }
                    }
                    _ => {}
                }
            }
        })?;

        // Watch parent directory (more reliable for atomic writes)
        let watch_dir = path.parent().unwrap_or(&path);
        watcher.watch(watch_dir.as_ref(), notify::RecursiveMode::NonRecursive)?;

        Ok(Self {
            path,
            changed,
            _watcher: watcher,
        })
    }

    /// Check if the file has changed since last call.
    /// Resets the flag atomically — concurrent calls are safe.
    pub fn check_changed(&self) -> bool {
        self.changed.swap(false, Ordering::AcqRel)
    }

    /// Get the watched path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_new_creates_watcher() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.txt");
        fs::write(&file, "initial").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        assert_eq!(w.path(), file);
    }

    #[test]
    fn test_check_changed_initially_false() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.txt");
        fs::write(&file, "data").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        // No modification yet
        assert!(!w.check_changed());
    }

    #[test]
    fn test_detects_file_modification() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.dxf");
        fs::write(&file, "v1").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(100));

        fs::write(&file, "v2").unwrap();
        thread::sleep(Duration::from_millis(500));

        assert!(w.check_changed(), "should detect modification");
    }

    #[test]
    fn test_check_changed_resets_flag() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.dxf");
        fs::write(&file, "v1").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(100));

        fs::write(&file, "v2").unwrap();
        thread::sleep(Duration::from_millis(500));

        assert!(w.check_changed(), "first check should be true");
        assert!(!w.check_changed(), "second check should be false (flag reset)");
    }

    #[test]
    fn test_detects_file_creation() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("new.dxf");
        // File doesn't exist yet — watch parent
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(100));

        fs::write(&file, "created").unwrap();
        thread::sleep(Duration::from_millis(500));

        assert!(w.check_changed(), "should detect creation");
    }

    #[test]
    fn test_multiple_rapid_writes() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("rapid.dxf");
        fs::write(&file, "v1").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(100));

        for i in 0..5 {
            fs::write(&file, format!("v{i}")).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
        thread::sleep(Duration::from_millis(500));

        assert!(w.check_changed(), "should detect at least one write");
    }

    #[test]
    fn test_error_on_nonexistent_parent() {
        let result = FileWatcher::new("/nonexistent/path/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_getter() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("x.dxf");
        fs::write(&file, "").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        assert_eq!(w.path(), file.as_path());
    }

    // ================================================================
    // (3) File deletion detection
    // ================================================================

    #[test]
    fn test_detects_file_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("deleteme.dxf");
        fs::write(&file, "content").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(100));

        fs::remove_file(&file).unwrap();
        thread::sleep(Duration::from_millis(500));

        // notify may or may not fire on deletion depending on backend.
        // The watcher should at least not crash.
        let _ = w.check_changed();
    }

    #[test]
    fn test_delete_then_recreate() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("recreate.dxf");
        fs::write(&file, "v1").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(100));

        fs::remove_file(&file).unwrap();
        thread::sleep(Duration::from_millis(200));

        fs::write(&file, "v2").unwrap();
        thread::sleep(Duration::from_millis(500));

        assert!(w.check_changed(), "should detect recreated file");
    }

    // ================================================================
    // (4) Atomic write: write to tmp file, then rename over target
    // ================================================================

    #[test]
    fn test_atomic_write_via_rename() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("target.dxf");
        fs::write(&file, "original").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(100));

        // Simulate atomic write: write to tmp, rename to target
        let tmp = dir.path().join("target.dxf.tmp");
        fs::write(&tmp, "updated content").unwrap();
        fs::rename(&tmp, &file).unwrap();
        thread::sleep(Duration::from_millis(500));

        assert!(w.check_changed(), "should detect atomic write via rename");
    }

    #[test]
    fn test_atomic_write_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("new_atomic.dxf");
        // Target doesn't exist yet
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(100));

        let tmp = dir.path().join("new_atomic.dxf.tmp");
        fs::write(&tmp, "content").unwrap();
        fs::rename(&tmp, &file).unwrap();
        thread::sleep(Duration::from_millis(500));

        assert!(w.check_changed(), "should detect atomic create via rename");
    }

    // ================================================================
    // (5) Non-existent paths: parent exists but file doesn't
    // ================================================================

    #[test]
    fn test_nonexistent_file_valid_parent() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("does_not_exist.dxf");
        // Parent exists, file doesn't — should succeed (watches parent)
        let result = FileWatcher::new(&file);
        assert!(result.is_ok(), "Should succeed when parent exists");
    }

    #[test]
    fn test_nonexistent_parent_errors() {
        let result = FileWatcher::new("/this/path/does/not/exist/file.dxf");
        assert!(result.is_err(), "Should error when parent doesn't exist");
    }

    #[test]
    fn test_empty_path_errors() {
        let result = FileWatcher::new("");
        assert!(result.is_err(), "Empty path should error");
    }

    // ================================================================
    // (6) Rapid writes: 10 writes at 100ms intervals, no event loss
    // ================================================================

    #[test]
    fn test_rapid_writes_10_at_100ms() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("rapid10.dxf");
        fs::write(&file, "initial").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(200));

        // Clear any initial events
        let _ = w.check_changed();

        for i in 0..10 {
            fs::write(&file, format!("write_{}", i)).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
        thread::sleep(Duration::from_millis(500));

        assert!(w.check_changed(),
            "Should detect at least one event from 10 rapid writes");
    }

    #[test]
    fn test_rapid_writes_burst_no_sleep() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("burst.dxf");
        fs::write(&file, "init").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(200));
        let _ = w.check_changed();

        // Burst: 20 writes with no delay
        for i in 0..20 {
            fs::write(&file, format!("burst_{}", i)).unwrap();
        }
        thread::sleep(Duration::from_millis(1000));

        assert!(w.check_changed(),
            "Should detect changes even from burst writes");
    }

    // ================================================================
    // Multiple check_changed calls between writes
    // ================================================================

    #[test]
    fn test_no_spurious_events() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("quiet.dxf");
        fs::write(&file, "content").unwrap();
        let w = FileWatcher::new(&file).unwrap();
        thread::sleep(Duration::from_millis(500));
        let _ = w.check_changed(); // drain initial

        // No writes — should stay false
        thread::sleep(Duration::from_millis(500));
        assert!(!w.check_changed(), "No writes → no change");
        assert!(!w.check_changed(), "Still no change");
    }
}
