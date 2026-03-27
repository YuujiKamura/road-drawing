# Flutter Web + Rust WASM Migration Plan (Issue #10)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace egui WASM UI with Flutter Web, keeping Rust WASM as the DXF computation backend.

**Architecture:** Flutter Web handles all UI (grid editor, DXF Canvas preview, toolbar). Rust WASM (via wasm-pack) exposes 3 functions: `parse_csv`, `generate_dxf`, `get_preview_data`. Flutter calls these through `dart:js_interop` → JS glue → wasm-bindgen exports.

**Tech Stack:** Flutter 3.27 (Dart), wasm-pack 0.13, wasm-bindgen, Rust workspace crates (road-section, dxf-engine, excel-parser)

**Roles:**
- **flutter-impl-a (59800):** Flutter UI (Tasks 2, 4, 6, 8)
- **flutter-impl-b (75428):** Rust WASM bridge + integration (Tasks 1, 3, 5, 7)
- **flutter-tester (90028):** Testing all tasks (Task 9, plus review each task)

---

## File Structure

### New files (flutter_web/)

```
flutter_web/
├── pubspec.yaml                    # Flutter project config
├── web/
│   ├── index.html                  # Flutter bootstrap + WASM loader
│   └── wasm/                       # wasm-pack output (gitignored, built)
│       ├── road_drawing_wasm_bg.wasm
│       ├── road_drawing_wasm.js
│       └── package.json
├── lib/
│   ├── main.dart                   # App entry, MaterialApp
│   ├── app.dart                    # MainLayout (grid + preview split)
│   ├── wasm_bridge.dart            # dart:js_interop bindings to WASM
│   ├── models/
│   │   ├── station_data.dart       # StationData (name, x, wl, wr)
│   │   └── preview_data.dart       # PreviewData (lines, texts for Canvas)
│   ├── widgets/
│   │   ├── grid_editor.dart        # DataTable CSV editor
│   │   ├── dxf_preview.dart        # CustomPainter DXF preview
│   │   └── toolbar.dart            # Add/Delete/Preview/Download buttons
│   └── services/
│       └── dxf_service.dart        # Orchestrates WASM calls
└── test/
    ├── models/station_data_test.dart
    ├── wasm_bridge_test.dart
    └── widgets/grid_editor_test.dart
```

### New files (wasm crate)

```
crates/road-drawing-wasm/
├── Cargo.toml                      # wasm-pack crate (cdylib)
├── src/
│   └── lib.rs                      # wasm-bindgen exports (3 functions)
└── build.sh                        # wasm-pack build → flutter_web/web/wasm/
```

### Modified files

```
Cargo.toml                          # Add road-drawing-wasm to workspace members
.github/workflows/deploy.yml        # Flutter build web + wasm-pack build
```

---

## Task 1: Rust WASM crate (flutter-impl-b)

**Goal:** Create `crates/road-drawing-wasm/` with wasm-pack, exposing 3 functions.

**Files:**
- Create: `crates/road-drawing-wasm/Cargo.toml`
- Create: `crates/road-drawing-wasm/src/lib.rs`
- Create: `crates/road-drawing-wasm/build.sh`
- Modify: `Cargo.toml` (workspace members)

- [ ] **Step 1: Create Cargo.toml**

```toml
# crates/road-drawing-wasm/Cargo.toml
[package]
name = "road-drawing-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
console_error_panic_hook = "0.1"
road-section = { path = "../road-section" }
dxf-engine = { path = "../dxf-engine" }
excel-parser = { path = "../excel-parser" }
```

- [ ] **Step 2: Add to workspace**

In root `Cargo.toml`, add `"crates/road-drawing-wasm"` to `[workspace] members`.

- [ ] **Step 3: Write lib.rs with 3 exported functions**

```rust
// crates/road-drawing-wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Parse CSV text, return JSON array of {name, x, wl, wr}
/// Called when: file drop, initial load
#[wasm_bindgen]
pub fn parse_csv(csv_text: &str) -> String {
    match road_section::parse_road_section_csv(csv_text) {
        Ok(stations) => {
            let rows: Vec<StationRow> = stations
                .iter()
                .map(|s| StationRow {
                    name: s.name.clone(),
                    x: s.x,
                    wl: s.wl,
                    wr: s.wr,
                })
                .collect();
            serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into())
        }
        Err(e) => format!(r#"{{"error":"{}"}}"#, e),
    }
}

/// Generate DXF string from CSV text
/// Called when: Download button clicked
#[wasm_bindgen]
pub fn generate_dxf(csv_text: &str) -> String {
    let stations = match road_section::parse_road_section_csv(csv_text) {
        Ok(s) => s,
        Err(e) => return format!("ERROR: {}", e),
    };
    let config = road_section::RoadSectionConfig::default();
    let geometry = road_section::calculate_road_section(&stations, &config);
    let (lines, texts) = road_section::geometry_to_dxf(&geometry);
    let writer = dxf_engine::DxfWriter::new();
    writer.write(&lines, &texts)
}

/// Get preview data as JSON: {lines: [{x1,y1,x2,y2,color}], texts: [{text,x,y,rotation,height,color}]}
/// Called when: CSV edited, need to redraw Canvas
#[wasm_bindgen]
pub fn get_preview_data(csv_text: &str) -> String {
    let stations = match road_section::parse_road_section_csv(csv_text) {
        Ok(s) => s,
        Err(_) => return r#"{"lines":[],"texts":[]}"#.into(),
    };
    let config = road_section::RoadSectionConfig::default();
    let geometry = road_section::calculate_road_section(&stations, &config);

    let lines: Vec<PreviewLine> = geometry
        .lines
        .iter()
        .map(|l| PreviewLine {
            x1: l.x1, y1: l.y1, x2: l.x2, y2: l.y2, color: l.color,
        })
        .collect();
    let texts: Vec<PreviewText> = geometry
        .texts
        .iter()
        .map(|t| PreviewText {
            text: t.text.clone(),
            x: t.x, y: t.y,
            rotation: t.rotation,
            height: t.height,
            color: t.color,
        })
        .collect();

    serde_json::to_string(&PreviewData { lines, texts })
        .unwrap_or_else(|_| r#"{"lines":[],"texts":[]}"#.into())
}

#[derive(Serialize, Deserialize)]
struct StationRow {
    name: String,
    x: f64,
    wl: f64,
    wr: f64,
}

#[derive(Serialize)]
struct PreviewData {
    lines: Vec<PreviewLine>,
    texts: Vec<PreviewText>,
}

#[derive(Serialize)]
struct PreviewLine { x1: f64, y1: f64, x2: f64, y2: f64, color: i32 }

#[derive(Serialize)]
struct PreviewText { text: String, x: f64, y: f64, rotation: f64, height: f64, color: i32 }
```

- [ ] **Step 4: Create build script**

```bash
#!/bin/bash
# crates/road-drawing-wasm/build.sh
set -e
cd "$(dirname "$0")"
wasm-pack build --target web --out-dir ../../flutter_web/web/wasm --out-name road_drawing_wasm
echo "WASM built → flutter_web/web/wasm/"
```

- [ ] **Step 5: Build and verify**

```bash
cd ~/road-drawing/crates/road-drawing-wasm
chmod +x build.sh
bash build.sh
# Expected: flutter_web/web/wasm/ contains road_drawing_wasm.js + .wasm
ls -la ../../flutter_web/web/wasm/
```

- [ ] **Step 6: Run existing tests to verify no regression**

```bash
cd ~/road-drawing
cargo test --workspace
# Expected: 768+ tests pass, new crate has no tests yet (that's OK)
```

- [ ] **Step 7: Commit**

```bash
git add crates/road-drawing-wasm/ Cargo.toml
git commit -m "feat(wasm): add road-drawing-wasm crate with wasm-pack (Issue #10)

Exports: parse_csv, generate_dxf, get_preview_data via wasm-bindgen.
Build: wasm-pack build --target web → flutter_web/web/wasm/"
```

---

## Task 2: Flutter project skeleton (flutter-impl-a)

**Goal:** Create `flutter_web/` with Flutter Web project, empty app shell.

**Files:**
- Create: `flutter_web/pubspec.yaml`
- Create: `flutter_web/lib/main.dart`
- Create: `flutter_web/lib/app.dart`
- Create: `flutter_web/web/index.html`

- [ ] **Step 1: Create Flutter project**

```bash
cd ~/road-drawing
~/flutter/bin/flutter create --project-name road_drawing_flutter --platforms web flutter_web
```

- [ ] **Step 2: Clean up generated files, edit pubspec.yaml**

Remove generated test/, replace pubspec.yaml:

```yaml
# flutter_web/pubspec.yaml
name: road_drawing_flutter
description: Road Drawing Web UI - Flutter + Rust WASM
version: 0.1.0
publish_to: none

environment:
  sdk: ^3.6.0

dependencies:
  flutter:
    sdk: flutter
  web: ^1.1.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^5.0.0
```

- [ ] **Step 3: Write main.dart**

```dart
// flutter_web/lib/main.dart
import 'package:flutter/material.dart';
import 'app.dart';

void main() {
  runApp(const RoadDrawingApp());
}

class RoadDrawingApp extends StatelessWidget {
  const RoadDrawingApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Road Drawing',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark(useMaterial3: true).copyWith(
        scaffoldBackgroundColor: const Color(0xFF1A1A1A),
      ),
      home: const MainLayout(),
    );
  }
}
```

- [ ] **Step 4: Write app.dart (split layout placeholder)**

```dart
// flutter_web/lib/app.dart
import 'package:flutter/material.dart';

class MainLayout extends StatefulWidget {
  const MainLayout({super.key});

  @override
  State<MainLayout> createState() => _MainLayoutState();
}

class _MainLayoutState extends State<MainLayout> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          // Left: Grid editor (380px)
          SizedBox(
            width: 380,
            child: Container(
              color: const Color(0xFF1E1E2E),
              child: const Center(child: Text('Grid Editor (Task 4)')),
            ),
          ),
          // Divider
          const VerticalDivider(width: 1, color: Color(0xFF333333)),
          // Right: DXF Preview
          Expanded(
            child: Container(
              color: const Color(0xFF1A1A1A),
              child: const Center(child: Text('DXF Preview (Task 6)')),
            ),
          ),
        ],
      ),
    );
  }
}
```

- [ ] **Step 5: Run flutter and verify blank app**

```bash
cd ~/road-drawing/flutter_web
~/flutter/bin/flutter run -d chrome
# Expected: dark window with "Grid Editor" left, "DXF Preview" right
```

- [ ] **Step 6: Commit**

```bash
git add flutter_web/
git commit -m "feat(flutter): create Flutter Web project skeleton (Issue #10)

MaterialApp with dark theme, Row split layout (380px grid + preview).
Run: flutter run -d chrome"
```

---

## Task 3: WASM bridge in Dart (flutter-impl-b)

**Goal:** Create `wasm_bridge.dart` that loads WASM module and calls the 3 exported functions.

**Files:**
- Create: `flutter_web/lib/wasm_bridge.dart`
- Create: `flutter_web/test/wasm_bridge_test.dart`
- Modify: `flutter_web/web/index.html` (add WASM script tag)

**Depends on:** Task 1 (WASM crate built)

- [ ] **Step 1: Write wasm_bridge.dart**

```dart
// flutter_web/lib/wasm_bridge.dart
import 'dart:js_interop';
import 'package:web/web.dart' as web;

/// Bridge to Rust WASM functions exported by road-drawing-wasm crate.
/// Must call [WasmBridge.init] before using any methods.
class WasmBridge {
  static bool _initialized = false;

  /// Load and initialize the WASM module.
  /// Call once at app startup.
  static Future<void> init() async {
    if (_initialized) return;
    // wasm-pack --target web generates an ES module with default init()
    final module = await _importWasmModule();
    await module.callMethod('default'.toJS).toDart;
    _initialized = true;
  }

  /// Parse CSV text → JSON string of [{name, x, wl, wr}, ...]
  static String parseCsv(String csvText) {
    _ensureInit();
    return _callWasm('parse_csv', csvText);
  }

  /// Generate DXF string from CSV text
  static String generateDxf(String csvText) {
    _ensureInit();
    return _callWasm('generate_dxf', csvText);
  }

  /// Get preview data as JSON: {lines: [...], texts: [...]}
  static String getPreviewData(String csvText) {
    _ensureInit();
    return _callWasm('get_preview_data', csvText);
  }

  static void _ensureInit() {
    if (!_initialized) {
      throw StateError('WasmBridge.init() must be called first');
    }
  }

  static String _callWasm(String funcName, String arg) {
    final result = _wasmExports.callMethod(funcName.toJS, arg.toJS);
    return (result as JSString).toDart;
  }

  static JSObject get _wasmExports => _cachedExports!;
  static JSObject? _cachedExports;

  static Future<JSObject> _importWasmModule() async {
    // Dynamic import of the wasm-pack generated ES module
    final promise = web.window.callMethod(
      'Function'.toJS,
      'return import("./wasm/road_drawing_wasm.js")'.toJS,
    );
    final module = await (promise as JSPromise).toDart;
    _cachedExports = module as JSObject;
    return _cachedExports!;
  }
}
```

**Note:** The exact JS interop API depends on the Dart SDK version. This uses `dart:js_interop` (Dart 3.3+). The implementer should verify against `package:web` docs and adjust if the `callMethod` signature differs.

- [ ] **Step 2: Add WASM script to index.html**

In `flutter_web/web/index.html`, add before `</head>`:

```html
<script type="module">
  import init from './wasm/road_drawing_wasm.js';
  window.__wasmInit = init;
</script>
```

- [ ] **Step 3: Test WASM loading manually**

```bash
# Build WASM first (Task 1)
cd ~/road-drawing/crates/road-drawing-wasm && bash build.sh

# Run Flutter
cd ~/road-drawing/flutter_web
~/flutter/bin/flutter run -d chrome
# Open DevTools console, verify no WASM load errors
```

- [ ] **Step 4: Commit**

```bash
git add flutter_web/lib/wasm_bridge.dart flutter_web/web/index.html
git commit -m "feat(flutter): add WASM bridge with dart:js_interop (Issue #10)

WasmBridge.init() loads WASM module.
parseCsv(), generateDxf(), getPreviewData() call Rust exports."
```

---

## Task 4: Grid editor widget (flutter-impl-a)

**Goal:** Editable DataTable with 4 columns: 測点名, 単延長L, 幅員W, 幅員右.

**Files:**
- Create: `flutter_web/lib/models/station_data.dart`
- Create: `flutter_web/lib/widgets/grid_editor.dart`
- Create: `flutter_web/lib/widgets/toolbar.dart`
- Create: `flutter_web/test/models/station_data_test.dart`
- Modify: `flutter_web/lib/app.dart`

- [ ] **Step 1: Write station_data.dart model**

```dart
// flutter_web/lib/models/station_data.dart
import 'dart:convert';

class StationData {
  String name;
  double x;
  double wl;
  double wr;

  StationData({
    required this.name,
    required this.x,
    required this.wl,
    required this.wr,
  });

  factory StationData.fromJson(Map<String, dynamic> json) {
    return StationData(
      name: json['name'] as String? ?? '',
      x: (json['x'] as num?)?.toDouble() ?? 0.0,
      wl: (json['wl'] as num?)?.toDouble() ?? 0.0,
      wr: (json['wr'] as num?)?.toDouble() ?? 0.0,
    );
  }

  Map<String, dynamic> toJson() => {'name': name, 'x': x, 'wl': wl, 'wr': wr};

  /// Convert list of StationData to CSV text (for WASM input)
  static String toCsv(List<StationData> stations) {
    final buf = StringBuffer('name,x,wl,wr\n');
    for (final s in stations) {
      buf.writeln('${s.name},${s.x},${s.wl},${s.wr}');
    }
    return buf.toString();
  }

  /// Parse JSON array string from WASM parse_csv output
  static List<StationData> fromJsonList(String jsonStr) {
    final list = jsonDecode(jsonStr) as List;
    return list.map((e) => StationData.fromJson(e as Map<String, dynamic>)).toList();
  }
}
```

- [ ] **Step 2: Write station_data_test.dart**

```dart
// flutter_web/test/models/station_data_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:road_drawing_flutter/models/station_data.dart';

void main() {
  test('toCsv generates valid CSV', () {
    final stations = [
      StationData(name: 'No.0', x: 0, wl: 3.45, wr: 3.55),
      StationData(name: 'No.1', x: 20, wl: 3.50, wr: 3.50),
    ];
    final csv = StationData.toCsv(stations);
    expect(csv, contains('name,x,wl,wr'));
    expect(csv, contains('No.0,0.0,3.45,3.55'));
    expect(csv, contains('No.1,20.0,3.5,3.5'));
  });

  test('fromJsonList parses WASM output', () {
    const json = '[{"name":"No.0","x":0.0,"wl":3.45,"wr":3.55}]';
    final stations = StationData.fromJsonList(json);
    expect(stations.length, 1);
    expect(stations[0].name, 'No.0');
    expect(stations[0].wl, 3.45);
  });

  test('roundtrip toCsv consistency', () {
    final original = StationData(name: 'Test', x: 10.5, wl: 2.0, wr: 2.0);
    final csv = StationData.toCsv([original]);
    expect(csv, contains('Test,10.5,2.0,2.0'));
  });
}
```

- [ ] **Step 3: Run test**

```bash
cd ~/road-drawing/flutter_web
~/flutter/bin/flutter test test/models/station_data_test.dart
# Expected: 3 tests pass
```

- [ ] **Step 4: Write grid_editor.dart**

```dart
// flutter_web/lib/widgets/grid_editor.dart
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/station_data.dart';

class GridEditor extends StatefulWidget {
  final List<StationData> stations;
  final ValueChanged<List<StationData>> onChanged;

  const GridEditor({
    super.key,
    required this.stations,
    required this.onChanged,
  });

  @override
  State<GridEditor> createState() => _GridEditorState();
}

class _GridEditorState extends State<GridEditor> {
  late List<StationData> _stations;
  int? _selectedIndex;

  @override
  void initState() {
    super.initState();
    _stations = List.from(widget.stations);
  }

  @override
  void didUpdateWidget(GridEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.stations != widget.stations) {
      _stations = List.from(widget.stations);
    }
  }

  void _notifyChange() {
    widget.onChanged(List.from(_stations));
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      child: DataTable(
        showCheckboxColumn: false,
        headingRowColor: WidgetStateProperty.all(const Color(0xFF2A2A3E)),
        columns: const [
          DataColumn(label: Text('測点名')),
          DataColumn(label: Text('単延長L'), numeric: true),
          DataColumn(label: Text('幅員W'), numeric: true),
          DataColumn(label: Text('幅員右'), numeric: true),
        ],
        rows: List.generate(_stations.length, (i) {
          final s = _stations[i];
          final selected = _selectedIndex == i;
          return DataRow(
            selected: selected,
            onSelectChanged: (_) => setState(() => _selectedIndex = i),
            cells: [
              _editableCell(s.name, (v) { s.name = v; _notifyChange(); }),
              _numericCell(s.x, (v) { s.x = v; _notifyChange(); }),
              _numericCell(s.wl, (v) { s.wl = v; _notifyChange(); }),
              _numericCell(s.wr, (v) { s.wr = v; _notifyChange(); }),
            ],
          );
        }),
      ),
    );
  }

  DataCell _editableCell(String value, ValueChanged<String> onChanged) {
    return DataCell(
      TextFormField(
        initialValue: value,
        style: const TextStyle(fontSize: 13),
        decoration: const InputDecoration(border: InputBorder.none, isDense: true),
        onFieldSubmitted: onChanged,
      ),
    );
  }

  DataCell _numericCell(double value, ValueChanged<double> onChanged) {
    return DataCell(
      TextFormField(
        initialValue: value.toString(),
        style: const TextStyle(fontSize: 13),
        decoration: const InputDecoration(border: InputBorder.none, isDense: true),
        keyboardType: const TextInputType.numberWithOptions(decimal: true),
        inputFormatters: [FilteringTextInputFormatter.allow(RegExp(r'[\d.]'))],
        onFieldSubmitted: (v) => onChanged(double.tryParse(v) ?? 0.0),
      ),
    );
  }
}
```

- [ ] **Step 5: Write toolbar.dart**

```dart
// flutter_web/lib/widgets/toolbar.dart
import 'package:flutter/material.dart';

class Toolbar extends StatelessWidget {
  final VoidCallback onAddRow;
  final VoidCallback onDeleteRow;
  final VoidCallback onPreview;
  final VoidCallback onDownload;

  const Toolbar({
    super.key,
    required this.onAddRow,
    required this.onDeleteRow,
    required this.onPreview,
    required this.onDownload,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: const BoxDecoration(
        border: Border(bottom: BorderSide(color: Color(0xFF333333))),
      ),
      child: Row(
        children: [
          const Text('Grid Editor',
              style: TextStyle(color: Color(0xFFAAAAAA), fontWeight: FontWeight.bold, fontSize: 13)),
          const Spacer(),
          _button('+ Row', onAddRow),
          const SizedBox(width: 6),
          _button('- Row', onDeleteRow),
          const SizedBox(width: 6),
          _button('Preview', onPreview),
          const SizedBox(width: 6),
          _button('DXF', onDownload),
        ],
      ),
    );
  }

  Widget _button(String label, VoidCallback onPressed) {
    return OutlinedButton(
      onPressed: onPressed,
      style: OutlinedButton.styleFrom(
        foregroundColor: const Color(0xFFCCCCCC),
        side: const BorderSide(color: Color(0xFF555555)),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
        textStyle: const TextStyle(fontSize: 13),
      ),
      child: Text(label),
    );
  }
}
```

- [ ] **Step 6: Update app.dart to use real widgets**

```dart
// flutter_web/lib/app.dart
import 'package:flutter/material.dart';
import 'models/station_data.dart';
import 'widgets/grid_editor.dart';
import 'widgets/toolbar.dart';

class MainLayout extends StatefulWidget {
  const MainLayout({super.key});

  @override
  State<MainLayout> createState() => _MainLayoutState();
}

class _MainLayoutState extends State<MainLayout> {
  List<StationData> _stations = [
    StationData(name: 'No.0', x: 0, wl: 3.45, wr: 3.55),
    StationData(name: 'No.1', x: 20, wl: 3.50, wr: 3.50),
    StationData(name: 'No.2', x: 40, wl: 3.55, wr: 3.55),
  ];

  void _addRow() {
    setState(() {
      final lastX = _stations.isNotEmpty ? _stations.last.x : 0.0;
      _stations.add(StationData(
        name: 'No.${_stations.length}',
        x: lastX + 20,
        wl: 3.0,
        wr: 3.0,
      ));
    });
  }

  void _deleteRow() {
    if (_stations.isNotEmpty) {
      setState(() => _stations.removeLast());
    }
  }

  void _onGridChanged(List<StationData> updated) {
    setState(() => _stations = updated);
    // TODO(Task 5): call WasmBridge.getPreviewData() here
  }

  void _onPreview() {
    // TODO(Task 5): trigger WASM preview
  }

  void _onDownload() {
    // TODO(Task 7): trigger WASM DXF download
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          SizedBox(
            width: 380,
            child: Container(
              color: const Color(0xFF1E1E2E),
              child: Column(
                children: [
                  Toolbar(
                    onAddRow: _addRow,
                    onDeleteRow: _deleteRow,
                    onPreview: _onPreview,
                    onDownload: _onDownload,
                  ),
                  Expanded(
                    child: GridEditor(
                      stations: _stations,
                      onChanged: _onGridChanged,
                    ),
                  ),
                ],
              ),
            ),
          ),
          const VerticalDivider(width: 1, color: Color(0xFF333333)),
          Expanded(
            child: Container(
              color: const Color(0xFF1A1A1A),
              child: const Center(child: Text('DXF Preview (Task 6)')),
            ),
          ),
        ],
      ),
    );
  }
}
```

- [ ] **Step 7: Run flutter and verify grid is editable**

```bash
cd ~/road-drawing/flutter_web
~/flutter/bin/flutter run -d chrome
# Expected: left panel shows editable DataTable with 3 rows, buttons work
```

- [ ] **Step 8: Commit**

```bash
git add flutter_web/lib/ flutter_web/test/
git commit -m "feat(flutter): add grid editor + toolbar + station model (Issue #10)

DataTable with 4 editable columns (測点名, 単延長L, 幅員W, 幅員右).
Toolbar: + Row, - Row, Preview, DXF buttons.
Unit test: StationData.toCsv/fromJsonList roundtrip."
```

---

## Task 5: WASM integration - preview data flow (flutter-impl-b)

**Goal:** Wire grid edits → WASM `get_preview_data()` → JSON for Canvas.

**Files:**
- Create: `flutter_web/lib/models/preview_data.dart`
- Create: `flutter_web/lib/services/dxf_service.dart`
- Modify: `flutter_web/lib/app.dart` (call dxf_service on change)

**Depends on:** Task 1, Task 3, Task 4

- [ ] **Step 1: Write preview_data.dart**

```dart
// flutter_web/lib/models/preview_data.dart
import 'dart:convert';

class PreviewLine {
  final double x1, y1, x2, y2;
  final int color;
  PreviewLine({required this.x1, required this.y1, required this.x2, required this.y2, required this.color});

  factory PreviewLine.fromJson(Map<String, dynamic> j) => PreviewLine(
    x1: (j['x1'] as num).toDouble(), y1: (j['y1'] as num).toDouble(),
    x2: (j['x2'] as num).toDouble(), y2: (j['y2'] as num).toDouble(),
    color: j['color'] as int? ?? 7,
  );
}

class PreviewText {
  final String text;
  final double x, y, rotation, height;
  final int color;
  PreviewText({required this.text, required this.x, required this.y,
    required this.rotation, required this.height, required this.color});

  factory PreviewText.fromJson(Map<String, dynamic> j) => PreviewText(
    text: j['text'] as String? ?? '',
    x: (j['x'] as num).toDouble(), y: (j['y'] as num).toDouble(),
    rotation: (j['rotation'] as num?)?.toDouble() ?? 0,
    height: (j['height'] as num?)?.toDouble() ?? 350,
    color: j['color'] as int? ?? 7,
  );
}

class PreviewData {
  final List<PreviewLine> lines;
  final List<PreviewText> texts;
  PreviewData({required this.lines, required this.texts});

  factory PreviewData.empty() => PreviewData(lines: [], texts: []);

  factory PreviewData.fromJson(String jsonStr) {
    final map = jsonDecode(jsonStr) as Map<String, dynamic>;
    return PreviewData(
      lines: (map['lines'] as List).map((e) => PreviewLine.fromJson(e)).toList(),
      texts: (map['texts'] as List).map((e) => PreviewText.fromJson(e)).toList(),
    );
  }
}
```

- [ ] **Step 2: Write dxf_service.dart**

```dart
// flutter_web/lib/services/dxf_service.dart
import '../models/station_data.dart';
import '../models/preview_data.dart';
import '../wasm_bridge.dart';

class DxfService {
  /// Get preview geometry from current stations
  static PreviewData getPreview(List<StationData> stations) {
    final csv = StationData.toCsv(stations);
    final json = WasmBridge.getPreviewData(csv);
    return PreviewData.fromJson(json);
  }

  /// Generate DXF string for download
  static String generateDxf(List<StationData> stations) {
    final csv = StationData.toCsv(stations);
    return WasmBridge.generateDxf(csv);
  }
}
```

- [ ] **Step 3: Update app.dart to call DxfService**

In `_MainLayoutState`, add:

```dart
PreviewData _preview = PreviewData.empty();

void _onGridChanged(List<StationData> updated) {
  setState(() {
    _stations = updated;
    _preview = DxfService.getPreview(_stations);
  });
}

void _onPreview() {
  setState(() => _preview = DxfService.getPreview(_stations));
}
```

Pass `_preview` to the preview widget (Task 6 will consume it).

- [ ] **Step 4: Commit**

```bash
git add flutter_web/lib/models/preview_data.dart flutter_web/lib/services/dxf_service.dart flutter_web/lib/app.dart
git commit -m "feat(flutter): wire grid → WASM → preview data flow (Issue #10)

DxfService.getPreview() converts stations to CSV, calls WASM, parses JSON.
PreviewData model with lines/texts for Canvas rendering."
```

---

## Task 6: DXF Canvas preview (flutter-impl-a)

**Goal:** CustomPainter that renders PreviewData (lines + texts) with viewport transform.

**Files:**
- Create: `flutter_web/lib/widgets/dxf_preview.dart`
- Modify: `flutter_web/lib/app.dart` (replace placeholder with DxfPreview)

**Depends on:** Task 5

- [ ] **Step 1: Write dxf_preview.dart**

```dart
// flutter_web/lib/widgets/dxf_preview.dart
import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../models/preview_data.dart';

class DxfPreview extends StatefulWidget {
  final PreviewData data;
  const DxfPreview({super.key, required this.data});

  @override
  State<DxfPreview> createState() => _DxfPreviewState();
}

class _DxfPreviewState extends State<DxfPreview> {
  Offset _pan = Offset.zero;
  double _zoom = 1.0;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onPanUpdate: (d) => setState(() => _pan += d.delta),
      onScaleUpdate: (d) {
        if (d.scale != 1.0) {
          setState(() => _zoom = (_zoom * d.scale).clamp(0.01, 100.0));
        }
      },
      child: ClipRect(
        child: CustomPaint(
          painter: _DxfPainter(
            data: widget.data,
            pan: _pan,
            zoom: _zoom,
          ),
          size: Size.infinite,
        ),
      ),
    );
  }
}

class _DxfPainter extends CustomPainter {
  final PreviewData data;
  final Offset pan;
  final double zoom;

  _DxfPainter({required this.data, required this.pan, required this.zoom});

  @override
  void paint(Canvas canvas, Size size) {
    if (data.lines.isEmpty && data.texts.isEmpty) return;

    // Calculate bounding box
    double minX = double.infinity, minY = double.infinity;
    double maxX = double.negativeInfinity, maxY = double.negativeInfinity;
    for (final l in data.lines) {
      minX = math.min(minX, math.min(l.x1, l.x2));
      minY = math.min(minY, math.min(l.y1, l.y2));
      maxX = math.max(maxX, math.max(l.x1, l.x2));
      maxY = math.max(maxY, math.max(l.y1, l.y2));
    }
    if (minX == double.infinity) return;

    final dxfW = maxX - minX;
    final dxfH = maxY - minY;
    if (dxfW <= 0 || dxfH <= 0) return;

    // Fit-to-view scale (with 10% margin)
    final margin = 0.9;
    final scaleX = size.width * margin / dxfW;
    final scaleY = size.height * margin / dxfH;
    final baseScale = math.min(scaleX, scaleY) * zoom;

    // Center offset
    final cx = (size.width - dxfW * baseScale) / 2 + pan.dx;
    final cy = (size.height - dxfH * baseScale) / 2 + pan.dy;

    // Transform: DXF Y-up → screen Y-down
    Offset transform(double x, double y) {
      return Offset(
        cx + (x - minX) * baseScale,
        cy + (maxY - y) * baseScale, // flip Y
      );
    }

    // Draw lines
    for (final l in data.lines) {
      final paint = Paint()
        ..color = _dxfColor(l.color)
        ..strokeWidth = 1.0;
      canvas.drawLine(transform(l.x1, l.y1), transform(l.x2, l.y2), paint);
    }

    // Draw texts
    for (final t in data.texts) {
      final textPainter = TextPainter(
        text: TextSpan(
          text: t.text,
          style: TextStyle(color: _dxfColor(t.color), fontSize: math.max(8, t.height * baseScale * 0.003)),
        ),
        textDirection: TextDirection.ltr,
      )..layout();

      final pos = transform(t.x, t.y);
      canvas.save();
      canvas.translate(pos.dx, pos.dy);
      if (t.rotation != 0) {
        canvas.rotate(-t.rotation * math.pi / 180); // DXF rotation is CCW
      }
      textPainter.paint(canvas, Offset.zero);
      canvas.restore();
    }
  }

  Color _dxfColor(int dxfColor) {
    const map = {
      1: Color(0xFFFF0000), // red
      2: Color(0xFFFFFF00), // yellow
      3: Color(0xFF00FF00), // green
      4: Color(0xFF00FFFF), // cyan
      5: Color(0xFF0000FF), // blue
      6: Color(0xFFFF00FF), // magenta
      7: Color(0xFFFFFFFF), // white
    };
    return map[dxfColor] ?? const Color(0xFFCCCCCC);
  }

  @override
  bool shouldRepaint(_DxfPainter old) =>
      old.data != data || old.pan != pan || old.zoom != zoom;
}
```

- [ ] **Step 2: Update app.dart to use DxfPreview**

Replace the placeholder `Center(child: Text('DXF Preview (Task 6)'))` with:

```dart
DxfPreview(data: _preview),
```

Add import: `import 'widgets/dxf_preview.dart';`

- [ ] **Step 3: Test visually**

```bash
cd ~/road-drawing/flutter_web
~/flutter/bin/flutter run -d chrome
# Expected: right panel shows road section lines with station names
# Pan with mouse drag, zoom with scroll
```

- [ ] **Step 4: Commit**

```bash
git add flutter_web/lib/widgets/dxf_preview.dart flutter_web/lib/app.dart
git commit -m "feat(flutter): DXF Canvas preview with pan/zoom (Issue #10)

CustomPainter renders PreviewData lines + texts.
DXF Y-up → screen Y-down transform, auto fit-to-view, DXF color mapping."
```

---

## Task 7: DXF download (flutter-impl-b)

**Goal:** Download button generates DXF via WASM and triggers browser file download.

**Files:**
- Modify: `flutter_web/lib/app.dart` (_onDownload implementation)

**Depends on:** Task 5

- [ ] **Step 1: Implement _onDownload in app.dart**

```dart
import 'dart:convert';
import 'package:web/web.dart' as web;
import 'services/dxf_service.dart';

void _onDownload() {
  final dxf = DxfService.generateDxf(_stations);
  if (dxf.startsWith('ERROR:')) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(dxf)),
    );
    return;
  }
  // Trigger browser download
  final bytes = utf8.encode(dxf);
  final blob = web.Blob([bytes.toJS].toJS, web.BlobPropertyBag(type: 'application/dxf'));
  final url = web.URL.createObjectURL(blob);
  final anchor = web.HTMLAnchorElement()
    ..href = url
    ..download = 'road_section.dxf';
  anchor.click();
  web.URL.revokeObjectURL(url);
}
```

- [ ] **Step 2: Test download**

```bash
cd ~/road-drawing/flutter_web
~/flutter/bin/flutter run -d chrome
# Click DXF button → road_section.dxf downloads
# Open in text editor, verify DXF structure (SECTION/HEADER/ENTITIES/EOF)
```

- [ ] **Step 3: Commit**

```bash
git add flutter_web/lib/app.dart
git commit -m "feat(flutter): DXF download via WASM + Blob (Issue #10)

DXF button → WasmBridge.generateDxf() → browser Blob download.
Error shown via SnackBar if generation fails."
```

---

## Task 8: CSV file drop support (flutter-impl-a)

**Goal:** Drop CSV/Excel file on the app → parse via WASM → populate grid.

**Files:**
- Modify: `flutter_web/lib/app.dart` (add drop target)

**Depends on:** Task 3, Task 4

- [ ] **Step 1: Add drop zone to app.dart**

Wrap the entire `Row` in `app.dart`'s build method:

```dart
import 'dart:convert';
import 'package:web/web.dart' as web;

// In _MainLayoutState, add method:
void _handleFileDrop(String content) {
  // Try parsing as CSV via WASM
  final json = WasmBridge.parseCsv(content);
  if (json.contains('"error"')) return;
  final stations = StationData.fromJsonList(json);
  setState(() {
    _stations = stations;
    _preview = DxfService.getPreview(_stations);
  });
}
```

Use HTML drag-and-drop via `dart:js_interop`:

```dart
@override
void initState() {
  super.initState();
  // Register HTML drop handler
  web.document.body?.addEventListener('drop', _onDrop.toJS);
  web.document.body?.addEventListener('dragover', _onDragOver.toJS);
}

void _onDragOver(web.Event e) {
  e.preventDefault();
}

void _onDrop(web.Event e) {
  e.preventDefault();
  final de = e as web.DragEvent;
  final files = de.dataTransfer?.files;
  if (files == null || files.length == 0) return;
  final file = files.item(0)!;
  final reader = web.FileReader();
  reader.onload = ((web.Event _) {
    final content = reader.result as String;
    _handleFileDrop(content);
  }).toJS;
  reader.readAsText(file);
}
```

**Note:** The exact `dart:js_interop` API for event listeners may need adjustment. The implementer should check `package:web` EventListener registration patterns for their Dart SDK version.

- [ ] **Step 2: Test file drop**

```bash
cd ~/road-drawing/flutter_web
~/flutter/bin/flutter run -d chrome
# Drag a CSV file onto the browser window → grid populates
```

- [ ] **Step 3: Commit**

```bash
git add flutter_web/lib/app.dart
git commit -m "feat(flutter): CSV file drop support (Issue #10)

HTML5 drag-and-drop → FileReader → WasmBridge.parseCsv() → grid update."
```

---

## Task 9: CI + GitHub Pages deploy (flutter-tester)

**Goal:** Update GitHub Actions to build Flutter Web + WASM and deploy to Pages.

**Files:**
- Modify: `.github/workflows/deploy.yml`

- [ ] **Step 1: Update deploy.yml**

```yaml
# .github/workflows/deploy.yml
name: Deploy to GitHub Pages

on:
  push:
    branches: [master, main]

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: "pages"
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Rust + wasm-pack
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      # Build WASM
      - name: Build WASM
        run: |
          cd crates/road-drawing-wasm
          wasm-pack build --target web --out-dir ../../flutter_web/web/wasm --out-name road_drawing_wasm

      # Run Rust tests
      - name: Rust tests
        run: cargo test --workspace

      # Flutter
      - uses: subosito/flutter-action@v2
        with:
          flutter-version: '3.27.3'
          channel: 'stable'

      - name: Flutter build web
        run: |
          cd flutter_web
          flutter pub get
          flutter build web --release --base-href /road-drawing/

      # Upload
      - uses: actions/upload-pages-artifact@v3
        with:
          path: flutter_web/build/web

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/deploy-pages@v4
        id: deployment
```

- [ ] **Step 2: Test CI locally (dry run)**

```bash
cd ~/road-drawing

# 1. Build WASM
cd crates/road-drawing-wasm && bash build.sh && cd ../..

# 2. Build Flutter
cd flutter_web
~/flutter/bin/flutter pub get
~/flutter/bin/flutter build web --release --base-href /road-drawing/
ls build/web/index.html
# Expected: build/web/ contains index.html + main.dart.js + wasm/

# 3. Rust tests still pass
cd ..
cargo test --workspace
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy.yml
git commit -m "ci: update GitHub Pages deploy for Flutter + WASM (Issue #10)

Build pipeline: wasm-pack → flutter build web → deploy to Pages.
Rust tests run before deploy."
```

---

## Dependency Graph

```
Task 1 (WASM crate)  ──→ Task 3 (Dart bridge) ──→ Task 5 (data flow) ──→ Task 7 (download)
                                                         ↓
Task 2 (Flutter skeleton) → Task 4 (grid editor) ──→ Task 6 (Canvas preview)
                                                         ↓
                                              Task 8 (file drop)
                                                         ↓
                                              Task 9 (CI deploy)
```

**Parallelism:**
- Task 1 + Task 2: **parallel** (flutter-impl-b + flutter-impl-a)
- Task 3 + Task 4: **parallel** (after their respective deps)
- Task 5: needs both Task 3 and Task 4
- Task 6 + Task 7: **parallel** (both need Task 5)
- Task 8: after Task 3 + Task 4
- Task 9: after all

**Assignment:**
| Task | Owner | Depends on |
|------|-------|------------|
| 1. WASM crate | flutter-impl-b (75428) | — |
| 2. Flutter skeleton | flutter-impl-a (59800) | — |
| 3. Dart WASM bridge | flutter-impl-b (75428) | Task 1 |
| 4. Grid editor | flutter-impl-a (59800) | Task 2 |
| 5. Preview data flow | flutter-impl-b (75428) | Task 1,3,4 |
| 6. DXF Canvas preview | flutter-impl-a (59800) | Task 5 |
| 7. DXF download | flutter-impl-b (75428) | Task 5 |
| 8. CSV file drop | flutter-impl-a (59800) | Task 3,4 |
| 9. CI deploy | flutter-tester (90028) | All |
