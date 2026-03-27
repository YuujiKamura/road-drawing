/**
 * WASM JS Interop Tests — テスト役B (76884)
 *
 * Node.js環境でWASMモジュールを直接ロードし、
 * Flutter Web index.htmlから呼ばれるのと同等のJS関数を検証する。
 *
 * 実行: node --experimental-wasm-modules crates/road-drawing-wasm/tests/wasm_js_interop_test.mjs
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
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

// --- Load WASM ---
const pkgDir = join(__dirname, '..', 'pkg');
const wasmBytes = readFileSync(join(pkgDir, 'road_drawing_wasm_bg.wasm'));

// We need to import the JS glue that wasm-pack generated.
// For Node.js, use initSync with raw bytes.
// On Windows, dynamic import needs file:// URL
import { pathToFileURL } from 'url';
const glueUrl = pathToFileURL(join(pkgDir, 'road_drawing_wasm.js')).href;
const glue = await import(glueUrl);
glue.initSync({ module: wasmBytes });

const { parse_csv, generate_dxf, get_preview_data, init } = glue;

console.log('WASM JS Interop Tests');
console.log('=====================');

// --- init() ---
console.log('\n[init]');
await test('init does not throw', () => {
  // init() sets panic hook — should not throw
  init();
});

// --- parse_csv ---
console.log('\n[parse_csv]');

await test('parse_csv valid CSV returns JSON array', () => {
  const result = parse_csv('No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n');
  const arr = JSON.parse(result);
  assert(Array.isArray(arr), 'result should be array');
  assertEq(arr.length, 2, 'station count');
  assertEq(arr[0].name, 'No.0', 'first station name');
  assertEq(arr[0].x, 0.0, 'first station x');
  assertEq(arr[0].wl, 3.0, 'first station wl');
  assertEq(arr[0].wr, 3.0, 'first station wr');
});

await test('parse_csv with header', () => {
  const result = parse_csv('測点名,延長,左幅員,右幅員\nNo.0,0.0,3.45,3.55\n');
  const arr = JSON.parse(result);
  assertEq(arr.length, 1, 'should have 1 station');
  assertEq(arr[0].wl, 3.45, 'wl value');
});

await test('parse_csv empty string returns error JSON', () => {
  const result = parse_csv('');
  const parsed = JSON.parse(result);
  assert(parsed.error !== undefined, 'should have error field');
});

await test('parse_csv returns valid JSON for all inputs', () => {
  for (const input of ['', 'garbage', 'a,b', 'No.0,0.0,3.0,3.0\n']) {
    const result = parse_csv(input);
    JSON.parse(result); // should not throw
  }
});

await test('parse_csv handles Japanese station names', () => {
  const result = parse_csv('起点,0.0,3.0,3.0\n終点,100.0,3.0,3.0\n');
  const arr = JSON.parse(result);
  assertEq(arr[0].name, '起点', 'Japanese name preserved');
  assertEq(arr[1].name, '終点', 'Japanese name preserved');
});

await test('parse_csv large input (100 stations)', () => {
  let csv = '';
  for (let i = 0; i < 100; i++) {
    csv += `No.${i},${i * 20.0},3.0,3.0\n`;
  }
  const arr = JSON.parse(parse_csv(csv));
  assertEq(arr.length, 100, '100 stations');
});

// --- generate_dxf ---
console.log('\n[generate_dxf]');

await test('generate_dxf produces valid DXF', () => {
  const dxf = generate_dxf('No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n');
  assert(!dxf.startsWith('ERROR'), `should not be error: ${dxf.slice(0, 80)}`);
  assert(dxf.includes('SECTION'), 'should contain SECTION');
  assert(dxf.includes('ENTITIES'), 'should contain ENTITIES');
  assert(dxf.includes('EOF'), 'should contain EOF');
  assert(dxf.includes('LINE'), 'should contain LINE entities');
});

await test('generate_dxf empty CSV returns ERROR', () => {
  const dxf = generate_dxf('');
  assert(dxf.startsWith('ERROR'), 'empty CSV should return ERROR');
});

await test('generate_dxf preserves station names', () => {
  const dxf = generate_dxf('Alpha,0.0,3.0,3.0\nBeta,20.0,3.5,3.5\n');
  assert(dxf.includes('Alpha'), 'should contain Alpha');
  assert(dxf.includes('Beta'), 'should contain Beta');
});

await test('generate_dxf output is ASCII-safe DXF', () => {
  const dxf = generate_dxf('No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n');
  // DXF group codes: "0\nSECTION", "0\nLINE" etc.
  assert(dxf.includes('0\nSECTION'), 'DXF group codes present');
  assert(dxf.includes('0\nLINE'), 'DXF LINE group codes present');
});

await test('generate_dxf large input does not crash', () => {
  let csv = '';
  for (let i = 0; i < 200; i++) {
    csv += `No.${i},${i * 20.0},3.0,3.0\n`;
  }
  const dxf = generate_dxf(csv);
  assert(!dxf.startsWith('ERROR'), 'large input should work');
  assert(dxf.length > 1000, 'DXF should have substantial content');
});

// --- get_preview_data ---
console.log('\n[get_preview_data]');

await test('get_preview_data returns valid JSON with lines and texts', () => {
  const json = get_preview_data('No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n');
  const data = JSON.parse(json);
  assert(Array.isArray(data.lines), 'should have lines array');
  assert(Array.isArray(data.texts), 'should have texts array');
  assert(data.lines.length > 0, 'lines should not be empty');
  assert(data.texts.length > 0, 'texts should not be empty');
});

await test('get_preview_data line has correct shape', () => {
  const data = JSON.parse(get_preview_data('No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n'));
  const line = data.lines[0];
  assert(typeof line.x1 === 'number', 'x1 is number');
  assert(typeof line.y1 === 'number', 'y1 is number');
  assert(typeof line.x2 === 'number', 'x2 is number');
  assert(typeof line.y2 === 'number', 'y2 is number');
  assert(typeof line.color === 'number', 'color is number');
});

await test('get_preview_data text has correct shape', () => {
  const data = JSON.parse(get_preview_data('No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n'));
  const text = data.texts[0];
  assert(typeof text.text === 'string', 'text is string');
  assert(typeof text.x === 'number', 'x is number');
  assert(typeof text.y === 'number', 'y is number');
  assert(typeof text.rotation === 'number', 'rotation is number');
  assert(typeof text.height === 'number', 'height is number');
  assert(typeof text.color === 'number', 'color is number');
});

await test('get_preview_data empty CSV returns empty arrays', () => {
  const data = JSON.parse(get_preview_data(''));
  assertEq(data.lines.length, 0, 'lines should be empty');
  assertEq(data.texts.length, 0, 'texts should be empty');
});

await test('get_preview_data coordinates are scaled (m→mm)', () => {
  const data = JSON.parse(get_preview_data('No.0,0.0,3.0,3.0\nNo.1,20.0,3.0,3.0\n'));
  // With default scale 1000, x=20m → some coordinate ≥ 1000
  const hasScaled = data.lines.some(l =>
    Math.abs(l.x1) >= 1000 || Math.abs(l.x2) >= 1000
  );
  assert(hasScaled, 'coordinates should be scaled');
});

// --- Cross-function consistency ---
console.log('\n[consistency]');

await test('parse_csv and generate_dxf agree on same input', () => {
  const csv = 'No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\n';
  const stations = JSON.parse(parse_csv(csv));
  assertEq(stations.length, 2, 'parse found 2 stations');
  const dxf = generate_dxf(csv);
  assert(!dxf.startsWith('ERROR'), 'dxf should succeed');
  assert(dxf.includes('No.0'), 'DXF has station No.0');
  assert(dxf.includes('No.1'), 'DXF has station No.1');
});

await test('parse_csv and get_preview_data agree on same input', () => {
  const csv = 'A,0.0,2.0,2.0\nB,10.0,3.0,3.0\n';
  const stations = JSON.parse(parse_csv(csv));
  assertEq(stations.length, 2, 'parse found 2 stations');
  const preview = JSON.parse(get_preview_data(csv));
  assert(preview.lines.length > 0, 'preview has lines');
});

// --- Edge cases for Flutter integration ---
console.log('\n[Flutter integration edge cases]');

await test('string with only whitespace', () => {
  const result = parse_csv('   \n  \n  ');
  const parsed = JSON.parse(result);
  assert(parsed.error !== undefined || Array.isArray(parsed) && parsed.length === 0,
    'whitespace-only should return error or empty');
});

await test('CSV with Windows line endings (CRLF)', () => {
  const result = parse_csv('No.0,0.0,3.0,3.0\r\nNo.1,20.0,3.5,3.5\r\n');
  const arr = JSON.parse(result);
  assertEq(arr.length, 2, 'CRLF should work');
});

await test('CSV with mixed line endings', () => {
  const result = parse_csv('No.0,0.0,3.0,3.0\nNo.1,20.0,3.5,3.5\r\n');
  const arr = JSON.parse(result);
  assertEq(arr.length, 2, 'mixed line endings should work');
});

await test('CSV with trailing comma', () => {
  // Extra columns should be ignored
  const result = parse_csv('No.0,0.0,3.0,3.0,extra\nNo.1,20.0,3.5,3.5,\n');
  const arr = JSON.parse(result);
  assertEq(arr.length, 2, 'extra columns should be ignored');
});

await test('rapid successive calls (simulating Flutter updates)', () => {
  for (let i = 0; i < 50; i++) {
    const csv = `No.0,0.0,${3.0 + i * 0.01},3.0\nNo.1,20.0,3.5,3.5\n`;
    const result = parse_csv(csv);
    JSON.parse(result); // should not crash
    get_preview_data(csv); // should not crash
  }
});

// --- Summary ---
console.log('\n=====================');
console.log(`Results: ${passed} passed, ${failed} failed, ${passed + failed} total`);
if (failures.length > 0) {
  console.log('\nFailures:');
  for (const f of failures) {
    console.log(`  ✗ ${f.name}: ${f.error}`);
  }
  process.exit(1);
}
console.log('\nAll tests passed!');
