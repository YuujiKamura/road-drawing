import 'wasm_bridge_stub.dart'
    if (dart.library.js_interop) 'wasm_bridge_web.dart' as impl;

/// Bridge to Rust WASM functions exported by road-drawing-wasm crate.
///
/// Uses conditional import: web → dart:js_interop, VM/test → stub.
class WasmBridge {
  /// Whether the WASM module has been successfully initialized.
  static bool get isInitialized => impl.wasmIsInitialized;

  /// Load and initialize the WASM module.
  static Future<void> init() => impl.wasmInit();

  /// Parse CSV text → JSON string of [{name, x, wl, wr}, ...]
  static String parseCsv(String csvText) => impl.wasmParseCsv(csvText);

  /// Generate DXF string from CSV text
  static String generateDxf(String csvText) => impl.wasmGenerateDxf(csvText);

  /// Get preview data as JSON: {lines: [...], texts: [...]}
  static String getPreviewData(String csvText) =>
      impl.wasmGetPreviewData(csvText);
}
