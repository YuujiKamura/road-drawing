import 'dart:async';
import 'dart:js_interop';
import 'dart:js_interop_unsafe';

/// Web implementation using dart:js_interop to call Rust WASM exports.
bool wasmIsInitialized = false;

Future<void> wasmInit() async {
  if (wasmIsInitialized) return;

  // Wait for index.html <script type="module"> to register window globals.
  // Module scripts are deferred, so they may not have run yet when main() fires.
  JSFunction? initFn;
  for (var i = 0; i < 50; i++) {
    initFn = _getGlobal('__wasmInit');
    if (initFn != null) break;
    await Future.delayed(const Duration(milliseconds: 100));
  }
  if (initFn == null) {
    throw StateError(
      'WASM init function not found after 5s. '
      'Ensure index.html loads wasm/road_drawing_wasm.js',
    );
  }
  final promise = initFn.callAsFunction(null);
  await (promise! as JSPromise).toDart;
  wasmIsInitialized = true;
}

String wasmParseCsv(String csvText) {
  return _callWasm('__wasm_parse_csv', csvText);
}

String wasmGenerateDxf(String csvText) {
  return _callWasm('__wasm_generate_dxf', csvText);
}

String wasmGetPreviewData(String csvText) {
  return _callWasm('__wasm_get_preview_data', csvText);
}

String _callWasm(String globalFnName, String arg) {
  final fn = _getGlobal(globalFnName);
  if (fn == null) {
    throw StateError('WASM function $globalFnName not found on window');
  }
  final result = fn.callAsFunction(null, arg.toJS);
  return (result! as JSString).toDart;
}

JSFunction? _getGlobal(String name) {
  final val = globalContext.getProperty(name.toJS);
  if (val == null || val.isUndefined) return null;
  return val as JSFunction;
}
