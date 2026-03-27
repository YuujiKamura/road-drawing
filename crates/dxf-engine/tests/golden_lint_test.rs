//! Golden file DXF lint validation
//!
//! Scans tests/golden/*.dxf and runs DxfLinter::is_valid() on each.
//! Also parses each file with parse_dxf() to verify roundtrip readability.
//!
//! If no golden files exist, the test passes with a skip message.
//! All test functions prefixed with test_golden_.

use std::fs;
use std::path::PathBuf;

use dxf_engine::{DxfLinter, parse_dxf};

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("golden")
}

fn collect_golden_files() -> Vec<PathBuf> {
    let dir = golden_dir();
    if !dir.exists() {
        return vec![];
    }
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("Failed to read golden dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "dxf"))
        .collect();
    files.sort();
    files
}

#[test]
fn test_golden_all_files_pass_lint() {
    let files = collect_golden_files();
    if files.is_empty() {
        eprintln!("[SKIP] No golden DXF files in {:?}", golden_dir());
        return;
    }

    let mut failures = Vec::new();
    for path in &files {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
        let result = DxfLinter::lint(&content);
        if result.has_errors() {
            failures.push((
                path.file_name().unwrap().to_string_lossy().to_string(),
                result.errors,
            ));
        }
    }

    if !failures.is_empty() {
        let mut msg = format!("{} golden file(s) failed lint:\n", failures.len());
        for (name, errors) in &failures {
            msg.push_str(&format!("  {} ({} errors):\n", name, errors.len()));
            for e in errors.iter().take(5) {
                msg.push_str(&format!("    line {}: {:?} — {}\n", e.line, e.code, e.message));
            }
        }
        panic!("{msg}");
    }

    eprintln!("[OK] {} golden DXF file(s) passed lint", files.len());
}

#[test]
fn test_golden_all_files_parseable() {
    let files = collect_golden_files();
    if files.is_empty() {
        eprintln!("[SKIP] No golden DXF files in {:?}", golden_dir());
        return;
    }

    let mut failures = Vec::new();
    for path in &files {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
        match parse_dxf(&content) {
            Ok(doc) => {
                let total = doc.lines.len() + doc.texts.len()
                    + doc.circles.len() + doc.polylines.len();
                eprintln!(
                    "  {} — {} entities ({}L {}T {}C {}P)",
                    path.file_name().unwrap().to_string_lossy(),
                    total, doc.lines.len(), doc.texts.len(),
                    doc.circles.len(), doc.polylines.len()
                );
            }
            Err(e) => {
                failures.push((
                    path.file_name().unwrap().to_string_lossy().to_string(),
                    format!("{e}"),
                ));
            }
        }
    }

    if !failures.is_empty() {
        let mut msg = format!("{} golden file(s) failed parse:\n", failures.len());
        for (name, err) in &failures {
            msg.push_str(&format!("  {}: {}\n", name, err));
        }
        panic!("{msg}");
    }

    eprintln!("[OK] {} golden DXF file(s) parsed successfully", files.len());
}

#[test]
fn test_golden_file_count() {
    let files = collect_golden_files();
    eprintln!(
        "[INFO] Golden DXF files: {} in {:?}",
        files.len(),
        golden_dir()
    );
    // This test always passes — it's informational.
    // When golden files are added, the lint and parse tests above
    // will enforce quality on each one.
}
