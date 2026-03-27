# Test Audit Report — road-drawing

**Date**: 2026-03-27
**Total tests**: 674
**All passing**: Yes

## 1. Test Distribution by Module

| Module | Crate | Tests | Coverage |
|--------|-------|------:|----------|
| dxf (entities/writer/linter/handle/reader/index) | dxf-engine | 228 | Excellent |
| crosswalk | road-marking | 46 | Excellent |
| tests (e2e/integration) | road-section, road-marking, triangle-core | 44 | Good |
| connection | triangle-core | 42 | Excellent |
| csv_loader | triangle-core | 40 | Excellent |
| triangle | triangle-core | 38 | Excellent |
| command | road-marking | 37 | Excellent |
| renderer | web | 29 | Good |
| section_detector | excel-parser | 25 | Good |
| station_name | excel-parser | 24 | Good |
| distance | excel-parser | 16 | Good |
| dxf_export | web | 15 | Good |
| transform | excel-parser | 12 | Adequate |
| app | web | 7 | Minimal |

## 2. PLAN.md Phase 2 Requirements Coverage

### 2-1: excel-parser (77 tests)

| Requirement | Status | Tests |
|-------------|--------|-------|
| セクション検出 (`区間X,台形計算` ヘッダ) | ✅ Covered | `test_extract_section_block`, `test_multi_section`, `test_get_sections_from_body` |
| 測点名自動生成 (20mピッチ, `No.0`/`0+10.5`) | ✅ Covered | `test_name_main_station`, `test_name_sub_station`, `test_fill_*` (8 tests) |
| 単延長↔累積距離変換 (中央値16m判定) | ✅ Covered | `test_is_cumulative_*`, `test_to_cumulative_*`, `test_median_boundary` |
| パイプライン統合 | ✅ Covered | `test_transform_span_data`, `test_transform_cumulative_data` |
| Shift_JIS CSV 対応 | ✅ Covered | `test_extract_shift_jis_csv_file`, `test_get_sections_shift_jis_file` |
| calamine Excel読み込み | ✅ Covered | `calamine_test.rs` (separate integration test with xlsx roundtrip) |

### Verification criteria from PLAN.md:

| Criterion | Status | Notes |
|-----------|--------|-------|
| 1. サンプルCSV 6ファイルでPython版とdiff比較 | ❌ **NOT DONE** | Real file test reads 区間1.csv but no Python output comparison |
| 2. セクション検出: 面積計算書CSV区間リスト一致 | ✅ Covered | `test_get_sections_from_body` |
| 3. 測点名生成: No.0, 0+10.5 形式 | ✅ Covered | 24 station_name tests |
| 4. 累積距離変換: 往復一致 | ⚠️ Partial | Cumsum tested but no reverse (cumulative→span) test |
| 5. cargo test 全パス | ✅ Covered | 674 tests pass |

### 2-2: CLI拡張

| Requirement | Status | Notes |
|-------------|--------|-------|
| `--section` | ✅ Implemented | No CLI-level test (manual verification only) |
| `--list-sections` | ✅ Implemented | No CLI-level test |

### 2-3: AIスキル定義

| Requirement | Status | Notes |
|-------------|--------|-------|
| `road-drawing.md` skill | ❌ **NOT DONE** | No skill file created |

## 3. PLAN.md Phase 3 Requirements Coverage

### 3-1: road-marking (83 tests)

| Requirement | Status | Tests |
|-------------|--------|-------|
| crosswalk stripe生成 | ✅ Covered | 46 crosswalk tests (count, symmetry, angled, layers) |
| JSONコマンド実行 | ✅ Covered | 37 command tests (parse, execute, edge cases) |

### 3-2: triangle-core (112 + 19 integration tests)

| Requirement | Status | Tests |
|-------------|--------|-------|
| Heron面積計算 | ✅ Covered | `test_area_*` (7 tests including edge cases) |
| 余弦定理頂点計算 | ✅ Covered | `test_vertex_*` (6 tests at various angles) |
| 接続座標計算 | ✅ Covered | 42 connection tests (type1/2, chains, errors) |
| CSV 4/6/28列対応 | ✅ Covered | 40 csv_loader tests (MIN/CONN/FULL) |
| 4.11.csv実データ | ✅ Covered | 19 integration tests |

### 3-3: DXFパーサー (dxf-engine reader/index)

| Requirement | Status | Tests |
|-------------|--------|-------|
| LINE/TEXT/CIRCLE/LWPOLYLINE読み込み | ✅ Covered | 37 roundtrip tests |
| ステーション座標検索 | ✅ Covered | `test_get_station_coord_*` (5 tests) |
| レイヤーフィルタ | ✅ Covered | `test_lines_on_layer_*`, `test_texts_on_layer` |
| バウンディングボックス | ✅ Covered | Including circle extent, polyline vertices |

### 3-4: CLI拡張

| Requirement | Status | Notes |
|-------------|--------|-------|
| `--type marking` | ✅ Implemented | No CLI-level test |
| `--type triangle` | ✅ Implemented | No CLI-level test |

## 4. Edge Case Gap Analysis

### Missing edge cases by module:

#### excel-parser
- ❌ **No Excel (.xlsx) real file test** — calamine_test uses in-memory xlsx but no disk file
- ❌ **No multi-sheet Excel** — only single-sheet tested
- ⚠️ **No BOM (Byte Order Mark) handling test** — some Windows CSVs have UTF-8 BOM
- ⚠️ **No very large CSV test** — only small datasets

#### triangle-core
- ❌ **No negative coordinate test** — all test triangles start at origin with positive coords
- ❌ **No very thin triangle test** — near-degenerate but valid (e.g., sides 100, 100, 0.1)
- ⚠️ **No duplicate triangle number test** — what happens with `1,6,5,4 \n 1,3,4,5`?

#### road-marking
- ❌ **No multi-segment centerline crosswalk test** — all tests use single-line centerlines
- ❌ **No very short centerline test** — centerline shorter than crosswalk width
- ⚠️ **No polyline-to-lines conversion test** — `polylineToLines()` exists in Kotlin but not ported

#### dxf-engine
- ⚠️ **No real-world DXF file roundtrip** — only self-generated DXF tested
- ⚠️ **No malformed DXF recovery test** — reader error paths minimally tested

## 5. Integration/Cross-crate Test Gaps

| Integration Path | Status | Notes |
|-----------------|--------|-------|
| excel-parser → road-section (RawRow → StationData) | ❌ **NO TEST** | CLI does the conversion but no unit test for the mapping |
| excel-parser → road-section → dxf-engine (full pipeline) | ⚠️ Partial | web/dxf_export tests cover road-section→dxf-engine but not excel-parser→road-section |
| triangle-core → dxf-engine (triangle DXF output) | ❌ **NO TEST** | CLI `--type triangle` renders triangles but no test for the rendering |
| road-marking → dxf-engine (crosswalk DXF output) | ⚠️ Partial | `test_crosswalk_generate_lint_roundtrip` validates DXF lint but not reader roundtrip |
| excel-parser + road-section + dxf-engine E2E | ❌ **NO TEST** | No test that reads CSV → transforms → generates DXF → reads back → validates |

## 6. Priority Recommendations

### P0 — Required for production confidence
1. **excel-parser → road-section integration test**: Verify `RawRow` → `StationData` conversion preserves all fields
2. **Full pipeline E2E test**: CSV file → excel-parser → road-section → DxfWriter → parse_dxf → validate line/text counts
3. **Python output comparison**: At least 1 sample CSV with known Python DXF output for diff comparison

### P1 — Should add before Phase 5 (crates.io publish)
4. **CLI integration tests**: `cargo run -- generate --input ... --output ... --type X` for each type
5. **Multi-segment crosswalk test**: Centerline with 2+ segments, verify stripe placement at bend
6. **Real xlsx file test**: Use csv_to_dxf's data files if any .xlsx exist, or create programmatic test

### P2 — Nice to have
7. **Degenerate input resilience**: Empty files, binary garbage, truncated CSV
8. **Property-based tests**: Random triangle side lengths → area ≥ 0, angles sum = 180°
9. **Performance test**: 1000 triangles, 100 road sections — not for speed but for correctness at scale
