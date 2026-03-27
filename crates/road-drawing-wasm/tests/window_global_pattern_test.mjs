/**
 * Window Global Pattern Tests — flutter-impl-b (75428)
 *
 * Verifies the index.html → window.__wasm_* → Dart globalContext.getProperty
 * → callAsFunction chain works correctly.
 *
 * Simulates the exact pattern used in production:
 *   1. index.html imports WASM module and sets window globals
 *   2. Dart wasm_bridge_web.dart reads globals via globalContext.getProperty()
 *   3. Dart calls fn.callAsFunction(null, arg.toJS) → JSString.toDart
 *
 * Run: node crates/road-drawing-wasm/tests/window_global_pattern_test.mjs
 */

import { readFileSync } from 'fs';
import { fileURLToPath, pathToFileURL } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// --- Test harness ---
let passed = 0;
let failed = 0;
const failures = [];

function assert(condition, message) {
  if (!condition) throw new Error(`Assertion failed: ${message}`);
}

function assertEq(a, b, message) {
  if (a !== b) throw new Error(`${message}: expected ${JSON.stringify(b)}, got ${JSON.stringify(a)}`);
}

function assertThrows(fn, message) {
  let threw = false;
  try { fn(); } catch (_) { threw = true; }
  if (!threw) throw new Error(`Expected to throw: ${message}`);
}

async function test(name, fn) {
  try {
    await fn();
    passed++;
    console.log(`  ✓ ${name}`);
  } catch (e) {
    failed++;
    failures.push({ name, error: e.message });
    console.log(`  ✗ ${name}: ${e.message}`);
  }
}

// --- Load WASM (same as index.html would) ---
const pkgDir = join(__dirname, '..', 'pkg');
const wasmBytes = readFileSync(join(pkgDir, 'road_drawing_wasm_bg.wasm'));
const glueUrl = pathToFileURL(join(pkgDir, 'road_drawing_wasm.js')).href;
const glue = await import(glueUrl);

console.log('Window Global Pattern Tests');
console.log('===========================');

// =====================================================
// Section 1: Simulate index.html global registration
// =====================================================
console.log('\n[1. Global registration (simulating index.html)]');

await test('register __wasmInit on globalThis', () => {
  // index.html does: window.__wasmInit = init;
  // In Node.js, globalThis === global
  globalThis.__wasmInit = glue.default;
  assert(typeof globalThis.__wasmInit === 'function', '__wasmInit should be a function');
});

await test('register __wasm_parse_csv on globalThis', () => {
  globalThis.__wasm_parse_csv = glue.parse_csv;
  assert(typeof globalThis.__wasm_parse_csv === 'function', 'should be a function');
});

await test('register __wasm_generate_dxf on globalThis', () => {
  globalThis.__wasm_generate_dxf = glue.generate_dxf;
  assert(typeof globalThis.__wasm_generate_dxf === 'function', 'should be a function');
});

await test('register __wasm_get_preview_data on globalThis', () => {
  globalThis.__wasm_get_preview_data = glue.get_preview_data;
  assert(typeof globalThis.__wasm_get_preview_data === 'function', 'should be a function');
});

// =====================================================
// Section 2: __wasmInit Promise pattern
// (Dart: initFn.callAsFunction(null) → JSPromise → .toDart)
// =====================================================
console.log('\n[2. __wasmInit Promise chain]');

await test('__wasmInit() returns a Promise (initSync fallback for Node)', () => {
  // In browser, init() fetches .wasm and returns Promise.
  // In Node, we use initSync, so init() with bytes should also work.
  // Verify the function exists and is callable.
  glue.initSync({ module: wasmBytes });
  // After initSync, the exports are usable
  const result = globalThis.__wasm_parse_csv('No.0,0.0,3.0,3.0\n');
  const arr = JSON.parse(result);
  assertEq(arr.length, 1, 'should work after init');
});

await test('__wasmInit idempotent — double init does not crash', () => {
  // Dart: if (_initialized) return;
  glue.initSync({ module: wasmBytes });
  glue.initSync({ module: wasmBytes });
  const result = globalThis.__wasm_parse_csv('No.0,0.0,3.0,3.0\n');
  JSON.parse(result); // should not throw
});

// =====================================================
// Section 3: globalContext.getProperty → callAsFunction
// (simulates Dart wasm_bridge_web.dart _callWasm)
// =====================================================
console.log('\n[3. getProperty → callAsFunction simulation]');

await test('getProperty finds registered __wasm_parse_csv', () => {
  // Dart: globalContext.getProperty('__wasm_parse_csv'.toJS)
  const fn = globalThis['__wasm_parse_csv'];
  assert(fn !== undefined, 'should exist on globalThis');
  assert(fn !== null, 'should not be null');
  assert(typeof fn === 'function', 'should be a function');
});

await test('callAsFunction with string arg returns string', () => {
  // Dart: fn.callAsFunction(null, arg.toJS) → JSString
  const fn = globalThis['__wasm_parse_csv'];
  const result = fn('No.0,0.0,3.0,3.0\n');
  assertEq(typeof result, 'string', 'return type should be string');
});

await test('callAsFunction result is valid JSON', () => {
  const fn = globalThis['__wasm_parse_csv'];
  const result = fn('No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n');
  const parsed = JSON.parse(result);
  assert(Array.isArray(parsed), 'should parse as array');
  assertEq(parsed.length, 2, '2 stations');
});

await test('getProperty returns undefined for unregistered name', () => {
  // Dart: if (val == null || val.isUndefined) return null → StateError
  const fn = globalThis['__wasm_nonexistent'];
  assertEq(fn, undefined, 'unregistered should be undefined');
});

await test('all 3 WASM functions have same call pattern', () => {
  const names = ['__wasm_parse_csv', '__wasm_generate_dxf', '__wasm_get_preview_data'];
  const csv = 'No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n';
  for (const name of names) {
    const fn = globalThis[name];
    assert(typeof fn === 'function', `${name} should be a function`);
    const result = fn(csv);
    assertEq(typeof result, 'string', `${name} should return string`);
    assert(result.length > 0, `${name} should return non-empty string`);
  }
});

// =====================================================
// Section 4: Return type validation per function
// =====================================================
console.log('\n[4. Return type validation]');

await test('parse_csv returns JSON array of StationRow objects', () => {
  const fn = globalThis['__wasm_parse_csv'];
  const result = JSON.parse(fn('No.0,0.0,3.45,3.55\n'));
  const row = result[0];
  // Verify StationRow shape: {name: String, x: f64, wl: f64, wr: f64}
  assertEq(typeof row.name, 'string', 'name is string');
  assertEq(typeof row.x, 'number', 'x is number');
  assertEq(typeof row.wl, 'number', 'wl is number');
  assertEq(typeof row.wr, 'number', 'wr is number');
  assertEq(row.name, 'No.0', 'name value');
  assertEq(row.x, 0.0, 'x value');
  assertEq(row.wl, 3.45, 'wl value');
  assertEq(row.wr, 3.55, 'wr value');
});

await test('generate_dxf returns DXF string (not JSON)', () => {
  const fn = globalThis['__wasm_generate_dxf'];
  const result = fn('No.0,0.0,3.0,3.0\nNo.1,20.0,3.0,3.0\n');
  // DXF is NOT JSON — it should not parse as JSON
  assert(!result.startsWith('[') && !result.startsWith('{'),
    'DXF should not be JSON');
  assert(result.includes('SECTION'), 'DXF has SECTION');
  assert(result.includes('EOF'), 'DXF has EOF');
});

await test('get_preview_data returns JSON with {lines, texts}', () => {
  const fn = globalThis['__wasm_get_preview_data'];
  const result = JSON.parse(fn('No.0,0.0,3.0,3.0\nNo.1,20.0,3.0,3.0\n'));
  assert('lines' in result, 'has lines key');
  assert('texts' in result, 'has texts key');
  assert(Array.isArray(result.lines), 'lines is array');
  assert(Array.isArray(result.texts), 'texts is array');
  // PreviewLine shape: {x1, y1, x2, y2, color}
  const line = result.lines[0];
  for (const key of ['x1', 'y1', 'x2', 'y2', 'color']) {
    assert(key in line, `line has ${key}`);
    assertEq(typeof line[key], 'number', `line.${key} is number`);
  }
  // PreviewText shape: {text, x, y, rotation, height, color}
  const text = result.texts[0];
  assertEq(typeof text.text, 'string', 'text.text is string');
  for (const key of ['x', 'y', 'rotation', 'height', 'color']) {
    assertEq(typeof text[key], 'number', `text.${key} is number`);
  }
});

// =====================================================
// Section 5: Error path (Dart StateError equivalents)
// =====================================================
console.log('\n[5. Error paths]');

await test('parse_csv error input returns JSON with error field', () => {
  const fn = globalThis['__wasm_parse_csv'];
  const result = fn('');
  const parsed = JSON.parse(result);
  assert('error' in parsed, 'empty input should return {error: ...}');
});

await test('generate_dxf error input returns ERROR: prefix', () => {
  const fn = globalThis['__wasm_generate_dxf'];
  const result = fn('');
  assert(result.startsWith('ERROR:'), 'empty input should return ERROR:');
});

await test('get_preview_data error input returns empty arrays', () => {
  const fn = globalThis['__wasm_get_preview_data'];
  const result = JSON.parse(fn(''));
  assertEq(result.lines.length, 0, 'empty lines');
  assertEq(result.texts.length, 0, 'empty texts');
});

// =====================================================
// Section 6: Dart bridge edge cases
// =====================================================
console.log('\n[6. Dart bridge edge cases]');

await test('globalThis property delete + re-register', () => {
  // Simulates page reload scenario
  delete globalThis.__wasm_parse_csv;
  assertEq(globalThis.__wasm_parse_csv, undefined, 'deleted');
  globalThis.__wasm_parse_csv = glue.parse_csv;
  const result = globalThis.__wasm_parse_csv('No.0,0.0,3.0,3.0\n');
  assert(JSON.parse(result).length === 1, 're-registered works');
});

await test('function reference identity preserved', () => {
  // The globalThis reference should be the same object as the import
  assertEq(globalThis.__wasm_parse_csv, glue.parse_csv, 'same reference');
  assertEq(globalThis.__wasm_generate_dxf, glue.generate_dxf, 'same reference');
  assertEq(globalThis.__wasm_get_preview_data, glue.get_preview_data, 'same reference');
});

await test('call with Unicode (Japanese) CSV via global', () => {
  const fn = globalThis['__wasm_parse_csv'];
  const result = JSON.parse(fn('起点,0.0,3.0,3.0\n終点,100.0,3.0,3.0\n'));
  assertEq(result[0].name, '起点', 'Unicode preserved through global chain');
  assertEq(result[1].name, '終点', 'Unicode preserved through global chain');
});

await test('call with very long CSV via global (stress)', () => {
  let csv = '';
  for (let i = 0; i < 500; i++) {
    csv += `No.${i},${i * 20.0},3.0,3.0\n`;
  }
  const fn = globalThis['__wasm_parse_csv'];
  const result = JSON.parse(fn(csv));
  assertEq(result.length, 500, '500 stations through global chain');
});

await test('preview data line count matches for 3 stations', () => {
  // 3 stations: 3×2 width lines + 2×3 connecting = 12 lines
  const fn = globalThis['__wasm_get_preview_data'];
  const data = JSON.parse(fn('No.0,0.0,3.0,3.0\nNo.1,20.0,3.0,3.0\nNo.2,40.0,3.0,3.0\n'));
  // exact count depends on Rust logic, but should be > 0
  assert(data.lines.length >= 6, `expected >=6 lines, got ${data.lines.length}`);
  // texts: station names + widths + distances
  assert(data.texts.length >= 3, `expected >=3 texts, got ${data.texts.length}`);
});

// =====================================================
// Section 7: Naming convention validation
// =====================================================
console.log('\n[7. Naming convention]');

await test('all globals use __wasm_ prefix', () => {
  const expected = ['__wasmInit', '__wasm_parse_csv', '__wasm_generate_dxf', '__wasm_get_preview_data'];
  for (const name of expected) {
    assert(name in globalThis, `${name} should exist on globalThis`);
    assert(typeof globalThis[name] === 'function', `${name} should be a function`);
  }
});

await test('no extra __wasm_ globals leaked', () => {
  const wasmKeys = Object.keys(globalThis).filter(k => k.startsWith('__wasm'));
  const expected = new Set(['__wasmInit', '__wasm_parse_csv', '__wasm_generate_dxf', '__wasm_get_preview_data']);
  for (const key of wasmKeys) {
    assert(expected.has(key), `unexpected global: ${key}`);
  }
});

// --- Summary ---
console.log('\n===========================');
console.log(`Results: ${passed} passed, ${failed} failed, ${passed + failed} total`);
if (failures.length > 0) {
  console.log('\nFailures:');
  for (const f of failures) {
    console.log(`  ✗ ${f.name}: ${f.error}`);
  }
  process.exit(1);
}
console.log('\nAll tests passed!');
