/// Stub implementation for VM/test environment (no dart:js_interop).
bool wasmIsInitialized = false;

Future<void> wasmInit() async {
  // No-op in non-web environment
  wasmIsInitialized = false;
}

String wasmParseCsv(String csvText) {
  throw UnsupportedError('WASM not available in this environment');
}

String wasmGenerateDxf(String csvText) {
  throw UnsupportedError('WASM not available in this environment');
}

String wasmGetPreviewData(String csvText) {
  throw UnsupportedError('WASM not available in this environment');
}
