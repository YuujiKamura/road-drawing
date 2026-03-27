# road-drawing ロードマップ

## テスト総数: 768 (全パス)

| Crate | Unit | Integration | Total |
|-------|------|-------------|-------|
| dxf-engine | 238 | 10 (compat) + 3 (golden) | 251 |
| triangle-core | 124 | 19 (integration) + 8 (property) | 151 |
| excel-parser | 82 | 7 (calamine) + 1 (real_file) + 10 (stress) | 100 |
| road-marking | 88 | — | 88 |
| road-drawing-web | 64 | 9 (e2e_pipeline) + 4 (hotswap) | 77 |
| road-section | 44 | 20 (e2e, 1 ignored) | 64 |
| cli | — | 11 (cli_test) + 24 (fixture_validation) | 35 |
| doctests | 2 | — | 2 |
| **Total** | **642** | **126** | **768** |

## Crate構成

```
crates/
├── dxf-engine/     — DXFエンティティ生成・リーダー・バリデーション・Lint・空間インデックス
├── road-section/   — 路線展開図ジオメトリ計算 + CSVパーサー
├── excel-parser/   — Excel/CSV入力パース (section_detector/station_name/distance/transform)
├── triangle-core/  — 三角形リスト計算エンジン (triangle/connection/csv_loader)
├── road-marking/   — 区画線生成 (crosswalk + command)
cli/                — road-drawing CLIツール
web/                — Web UI (egui native DXFビューワ + WASM + DXFエクスポート)
  ├── src/app.rs          — CSV D&D + Shift_JIS自動検出 + プレビュー描画
  ├── src/renderer.rs     — Viewport座標変換 (DXF Y-up → screen Y-down)
  ├── src/dxf_export.rs   — stations_to_dxf() + カスタムスケール
  ├── src/dxf_viewer.rs   — DXFホットスワップビューワ (notify file watcher)
  └── src/bin/dxf_viewer.rs — ビューワバイナリ
```

## Phase完了状況

| Phase | Status | 概要 |
|-------|--------|------|
| **1** | ✅ 完了 | dxf-engine + road-section + CLI基盤 |
| **2** | ✅ 完了 | excel-parser (section_detector/station_name/distance/transform) |
| **2.5** | 未着手 | LLMによる勝手書式→マスタ書式変換 |
| **3** | ✅ 完了 | triangle-core + road-marking + dxf-engine reader/index |
| **4 (egui)** | ✅ 完了 | egui WASM + DXFプレビュー + GitHub Pages CI |
| **4.5** | 🔄 方針転換 | HTML/JS + Tabulator + Rust WASMバックエンドへ移行 (Issue #6) |
| **5** | 未着手 | trianglelist依存切替 + crates.io publish |

### Issues 完了状況

| Issue | Title | Status |
|-------|-------|--------|
| #1 | Phase 2: Excel入力パーサー | ✅ 完了 |
| #2 | Phase 3: Kotlin区画線ロジック移植 | ✅ 完了 |
| #3 | Phase 4: Web UI (egui WASM) | ✅ 完了 |
| #4 | Phase 5: trianglelist依存切替 | 未着手 |
| #5 | Phase 2.5: LLM書式変換 | 未着手 |
| #6 | Phase 4.5: Web UIエディタ選定 | OPEN — 方針決定済み |
| #7 | Golden DXF regression testing | ✅ 完了 |
| #8 | テストフィクスチャ自己完結化 | ✅ 完了 |
| #9 | DXFホットスワップビューワ基盤整備 | ✅ 完了 |

---

## Phase 4.5: HTML/JS + Rust WASM バックエンド (Issue #6)

### 方針転換の理由
Phase 4のegui WASMは描画プレビューとして動くが、CSVエディタやコードエディタのようなリッチUI部品との統合が困難（egui CanvasとDOM要素のz-index/イベント共存問題）。

### 新アーキテクチャ
```
[HTML/JS フロント]
├── Tabulator (99KB, MIT) — CSV 4列グリッドエディタ
├── CodeMirror 6 (124KB, MIT) — JSON/DXFコードエディタ
├── Canvas/SVG — DXFプレビュー描画
└── wasm-bindgen ↔ Rust WASM バックエンド
    ├── excel-parser — Excel/CSVパース
    ├── road-section — ジオメトリ計算
    ├── dxf-engine — DXF生成
    └── triangle-core — 三角形計算
```

### ワークフロー
```
[Excel D&D] or [手入力]
    ↓
[Tabulator 4列グリッド] ← ユーザーが直接編集可能
    ↓ (onEdit → WASM呼び出し)
[Rust WASM: excel-parser → road-section → dxf-engine]
    ↓
[Canvas/SVG プレビュー] ← リアルタイム更新
    ↓
[DXFダウンロード] ← Blob生成
```

### 既存egui資産の扱い
- `web/src/app.rs` (CSV D&D + プレビュー): HTML/JSに置き換え
- `web/src/renderer.rs` (Viewport/座標変換): ロジックはWASMに残す
- `web/src/dxf_viewer.rs` (ホットスワップビューワ): nativeバイナリとして維持 (#9)
- `web/src/dxf_export.rs` (DXF生成): WASM経由で呼び出し

### タスク
- [ ] Tabulator + CodeMirror のPoC作成
- [ ] wasm-bindgen経由のexcel-parser/road-section呼び出しPoC
- [ ] LLM整形フロー + ルール蓄積の設計詳細

---

## 現状 (Phase 1 完了)

初期コミット `ff6cd91` で以下を確立:

- **dxf-engine**: DXF エンティティ生成・バリデーション (Line, Text, Circle, LwPolyline, Handle, Linter, Writer) — 64テスト
- **road-section**: 路線展開図ジオメトリ計算 + CSV パーサー (測点名/延長/左幅員/右幅員)
- **cli**: `road-drawing generate` コマンド (CSV→DXF)

ソース元:
- `trianglelist/rust-dxf/` → dxf-engine
- `trianglelist-web/road_section.rs` → road-section

---

## Phase 2: Excel入力パーサー + AIスキル層 (Issue #1) ✅ 完了

32テスト全パス。section_detector / station_name / distance / transform の4モジュール実装済み。CLI に `--section` / `--list-sections` オプション追加済み。

### 目的
Python版 `csv_to_dxf/src/processing.py` のパイプラインをRust移植。多様な書式のExcel/CSVから構造化データへ変換。

### タスク

#### 2-1: `crates/excel-parser/` 作成
**新規ファイル:**
- `crates/excel-parser/Cargo.toml`
- `crates/excel-parser/src/lib.rs`
- `crates/excel-parser/src/section_detector.rs` — セクション検出 (`区間X,台形計算` ヘッダ)
- `crates/excel-parser/src/station_name.rs` — 測点名自動生成 (20mピッチ, `No.0`/`0+10.5` 形式)
- `crates/excel-parser/src/distance.rs` — 単延長↔累積距離変換 (中央値16m判定)
- `crates/excel-parser/src/transform.rs` — パイプライン統合 (extract → to_cumulative → fill_station_names)

**依存:**
- `calamine` — Excel (.xlsx/.xls/.ods) 読み込み
- `encoding_rs` — Shift_JIS CSV 対応

**移植元 (Python):**
| Rust モジュール | Python ソース | 主要関数 |
|---|---|---|
| `section_detector` | `src/processing.py` | `extract_section_data()`, `get_available_sections()` |
| `station_name` | `src/station_name_utils.py` | `fill_station_names()`, PITCH_M=20, Decimal丸め |
| `distance` | `src/processing.py` | `to_cumulative()`, median(diffs)<16m 判定 |
| `transform` | `src/processing.py` | `transform_section()`, ROUND_N=2 |

**重要定数 (Python版から):**
```
PITCH_M = 20.0    # 測点間隔 (m)
ROUND_N = 2       # 小数点桁数
SPAN = 20         # 測点名ピッチ
scale = 1000.0    # DXF座標スケール
text_height = 350.0  # デフォルトテキスト高さ
```

#### 2-2: CLI拡張
**変更ファイル:**
- `cli/src/main.rs` — `--section`, `--list-sections` オプション追加
- `cli/Cargo.toml` — `excel-parser` 依存追加

#### 2-3: AIスキル定義
**新規ファイル:**
- `~/.claude/skills/road-drawing.md` — CLIの使い方・入出力仕様をスキルとして定義

### 検証
1. `csv_to_dxf/data/` のサンプルCSV 6ファイルで Python版と出力DXF を diff 比較
2. セクション検出: `面積計算書...csv` で区間リスト一致確認
3. 測点名生成: `No.0`, `0+10.5` 形式の正確性
4. 累積距離変換: 単延長→累積の往復一致
5. `cargo test` 全パス

### リスク
- **calamine WASM対応**: Phase 4 で WASM ビルドする際に calamine が動くか未確認。fallback は SheetJS (JS側)
- **Shift_JIS CSV**: Windows環境の業務CSVはShift_JIS多い。encoding_rs でデコード必須
- **浮動小数点丸め**: Python の `Decimal` と Rust の `f64` で丸め結果が微妙にずれる可能性。ROUND_N=2 で比較検証

### 依存関係
- Phase 1 (完了) の road-section の `StationData` 型を再利用
- excel-parser → road-section への変換レイヤーが必要

---

## Phase 2.5: LLMによる勝手書式→マスタ書式変換層 (Issue #5)

### 目的
現場のExcelは書式がバラバラ。excel-parserが受け付けるマスタ書式に、LLMで整形する前処理層を追加する。

### パイプライン
```
[勝手書式Excel] → calamine全セルダンプ → テキスト化 → LLM → [マスタ書式CSV] → excel-parser → DXF
```

### マスタ書式定義
```csv
測点名,単延長L,幅員W,幅員右
No.0,0.00,0.80,0.00
```
または複数区間: `区間X,台形計算` ヘッダ付き

### タスク

#### 2.5-1: calamine全セルダンプ
**新規ファイル:**
- `crates/excel-parser/src/cell_dump.rs` — calamine でシート全セルをテキスト化

**依存:**
- `calamine` (excel-parser に追加)

#### 2.5-2: LLMプロンプト + cli-ai-analyzer連携
**新規ファイル:**
- `crates/excel-parser/src/ai_convert.rs` — プロンプト構築 + cli-ai-analyzer 呼び出し + 出力CSV検証

**依存:**
- `cli-ai-analyzer` crate (既存、~/cli-ai-analyzer)
- Gemini/Claude バックエンド

**プロンプト設計:**
- マスタ書式仕様をプロンプトに埋め込み
- セルダンプをコンテキストとして渡す
- 出力: マスタ書式CSV文字列

#### 2.5-3: CLI `--ai-parse` フラグ
**変更ファイル:**
- `cli/src/main.rs` — `--ai-parse` オプション追加。フラグあり→LLM変換→excel-parser、なし→直接excel-parser

### 検証
1. 既知書式CSV: `--ai-parse` なしで従来通り動くこと (回帰テスト)
2. 未知書式Excel: `--ai-parse` ありでマスタ書式に変換 → DXF生成
3. LLM出力のCSVが excel-parser で正常パースされること
4. `cargo test` 全パス

### リスク
- **LLM出力の安定性**: 同じ入力でも毎回微妙に異なるCSVを出す可能性。出力検証+リトライが必要
- **calamine依存追加**: Phase 4 の WASM ビルドに影響する可能性 (Phase 2 で識別済みのリスク)
- **cli-ai-analyzer の外部依存**: Gemini/Claude APIキーが必要。オフライン動作不可

### 依存関係
- Phase 2 (完了) の excel-parser マスタ書式パーサーを再利用
- cli-ai-analyzer crate への依存追加

---

## Phase 3: Kotlin区画線ロジックのRust移植 (Issue #2) ✅ 完了

131テスト全パス (triangle-core 112 + integration 19)。road-marking 44テスト。
triangle-core: triangle/connection/csv_loader の3モジュール。MIN/CONN/FULL 28列対応。
road-marking: crosswalk + command の2モジュール。JSON→描画コマンド実行エンジン。
dxf-engine: reader/index 追加 (228テスト)。Writer出力のラウンドトリップ検証済み。

### 目的
trianglelist (Kotlin) の三角形リスト展開図・区画線生成ロジックをRust移植。DXFリーダーも追加。

### タスク

#### 3-1: `crates/road-marking/` 作成
**新規ファイル:**
- `crates/road-marking/Cargo.toml`
- `crates/road-marking/src/lib.rs`
- `crates/road-marking/src/crosswalk.rs` — 横断歩道生成
- `crates/road-marking/src/command.rs` — JSONコマンド実行エンジン

**移植元 (Kotlin):**
| Rust モジュール | Kotlin ソース | 概要 |
|---|---|---|
| `crosswalk` | `common/.../CrosswalkGenerator.kt` (推定) | 横断歩道パターン生成 |
| `command` | `common/.../CommandExecutor.kt` (推定) | JSON→描画コマンド変換 |

#### 3-2: `crates/triangle-core/` 作成 — 三角形リスト計算エンジン
**新規ファイル:**
- `crates/triangle-core/Cargo.toml`
- `crates/triangle-core/src/lib.rs`
- `crates/triangle-core/src/triangle.rs` — Triangle構造体 + ジオメトリ計算
- `crates/triangle-core/src/csv_loader.rs` — 三角形CSV読み込み (4/6/28列対応)
- `crates/triangle-core/src/dims.rs` — 寸法表示・自動配置ロジック
- `crates/triangle-core/src/connection.rs` — 親子接続・座標検証

**移植元 (Kotlin + 既存Rust):**
| Rust モジュール | 移植元 | 概要 |
|---|---|---|
| `triangle` | `rust-trilib/src/model/triangle.rs` + `editmodel/Triangle.kt` | 三角形計算 (Heron面積, 余弦定理, 接続座標) |
| `csv_loader` | `datamanager/CsvLoader.kt` | CSV解析 (MIN4/CONN6/FULL28列) |
| `dims` | `editmodel/Dims.kt` | 寸法自動配置 (鋭角検出, 接続辺除外) |
| `connection` | `Triangle.kt` calculate_points_connected() | 親子接続座標計算 |

**三角形CSV列構造:**
```
MIN (4列):  NUMBER, LENGTH_A, LENGTH_B, LENGTH_C
CONN (6列): + PARENT_NUMBER, CONNECTION_TYPE (-1/1/2)
FULL (28列): + NAME, POINT位置, COLOR, DIM配置, ANGLE, ...
```

**コア計算:**
- 面積: Heron公式 `s=(a+b+c)/2, area=√(s(s-a)(s-b)(s-c))`, 小数2桁丸め
- 頂点: CA=原点, AB=x軸上(距離=c), BC=余弦定理
- 接続: 子のA辺 = 親のB辺(type=1) or C辺(type=2)
- 検証: `child.length_a == parent.length_{b|c}`, 座標距離 < 0.01

#### 3-3: DXFパーサー追加
**変更ファイル:**
- `crates/dxf-engine/Cargo.toml` — `dxf` crate v0.6 を optional 依存に追加
- `crates/dxf-engine/src/dxf/reader.rs` — 新規。DXFファイル読み込み
- `crates/dxf-engine/src/dxf/index.rs` — 空間インデックス (`DxfIndex.kt` 移植)

**判断ポイント:** 外部 `dxf` crate を Reader として採用し、Writer は自前を維持。理由: Writer は CAD互換性のため細かい制御が必要だが、Reader は標準パーサーで十分。

#### 3-4: CLI拡張
**変更ファイル:**
- `cli/src/main.rs` — `--type marking`, `analyze` サブコマンド追加
- `cli/Cargo.toml` — `road-marking`, `triangle-core` 依存追加

### 検証
1. `trianglelist/app/src/test/` のテストCSV (`minimal.csv`, `connected.csv`, `4.11.csv`) で計算結果一致
2. 面積計算: Kotlin版と小数2桁まで完全一致
3. 接続座標: 親子間の座標差 < 0.01
4. 寸法配置: 鋭角検出→反対辺配置の自動ロジック検証
5. DXF読み込み→書き出しラウンドトリップ

### リスク
- **Kotlin→Rust移植の精度**: 浮動小数点演算の差異 (f32 vs f64)。Kotlin版は Float (f32相当) だが Rust は f64 推奨→精度向上方向なので問題少
- **28列CSV後方互換**: フル形式CSVの列マッピングが複雑。テストデータ `4.11.csv` で検証必須
- **DxfIndex空間検索**: Kotlin版の実装詳細未調査。Phase 3-2 着手時に `DxfIndex.kt` を精読する

### 依存関係
- Phase 1 の dxf-engine (Writer) を再利用
- triangle-core は dxf-engine に依存 (DXF出力用)
- road-marking は triangle-core + dxf-engine に依存

---

## Phase 4: Web UI層 — egui WASM (Issue #3) ✅ 完了

77テスト全パス。web/ crate作成済み (eframe 0.29, egui 0.29)。
app.rs: CSV D&D + Shift_JIS自動検出 + road-section プレビュー描画。
renderer.rs: Viewport座標変換 (DXF Y-up → screen Y-down) + DXFカラーマッピング。
dxf_export.rs: stations_to_dxf() + カスタムスケール対応 + ラウンドトリップ検証済み。
dxf_viewer.rs: DXFホットスワップビューワ (notify file watcher + auto-reload) (#9)。
WASM ビルド: `trunk build --release` 通過済み。calamine も WASM で動作確認済み。
GitHub Pages デプロイ: `.github/workflows/deploy.yml` 設定済み (push to master → trunk build → Pages)。

### Issue #7: Golden DXF regression testing ✅ 完了
DXF構造比較 (ハンドル/タイムスタンプ無視)。golden_lint_test + comparator。

### Issue #8: テストフィクスチャ自己完結化 ✅ 完了
外部リポ依存を解消。24 fixture validation tests。

### Issue #9: DXFホットスワップビューワ基盤整備 ✅ 完了
- `dxf-viewer` バイナリ: `cargo run --bin dxf-viewer -- <path.dxf>`
- notify crate によるファイル変更検知 + 自動リロード
- LINE/TEXT/CIRCLE/LWPOLYLINE 全エンティティ描画
- テスト: BBox (7) + DXFエンティティ→画面座標変換 (6) + ファイル監視 (4) + E2Eパイプライン (9)

---

## Phase 5: trianglelist 依存切り替え + crates.io publish (Issue #4)

### 目的
trianglelist と csv_to_dxf のコード重複を解消し、road-drawing を single source of truth にする。

### タスク

#### 5-1: trianglelist の依存切り替え
**変更ファイル (trianglelist 側):**
- `rust-dxf/Cargo.toml` — `dxf-engine = { git = "<road-drawing repo URL>" }` に切替
- `rust-trilib/Cargo.toml` — `triangle-core` への依存に切替
- 旧 `rust-dxf/src/` 配下のソースを削除
- `desktop/build.gradle.kts`, `settings.gradle.kts` — Rust ビルド設定更新

**段階:**
1. git 依存で切替 → CI 通るまで調整
2. 旧コード削除
3. trianglelist 側テスト全パス確認

#### 5-2: csv_to_dxf/web の DXF ビューアロジック吸収
**変更ファイル:**
- `crates/dxf-engine/src/dxf/viewer.rs` — 新規。DXF→Canvas描画ロジック
- `csv_to_dxf/web/src/lib.rs` → road-drawing の web/ に統合

#### 5-3: crates.io publish 準備
**新規/変更ファイル:**
- 各 crate に `README.md` 追加 (crates.io 表示用)
- `LICENSE` ファイル (MIT)
- 各 `Cargo.toml` に `description`, `keywords`, `categories` 追加
- `cargo publish --dry-run` で各 crate 検証

**publish 順序** (依存の葉から):
1. `dxf-engine` (依存なし)
2. `road-section` (→ dxf-engine)
3. `excel-parser` (→ calamine, encoding_rs)
4. `triangle-core` (→ dxf-engine)
5. `road-marking` (→ triangle-core, dxf-engine)
6. `road-drawing` CLI (bin crate, publish 不要)

### 検証
1. trianglelist: `./gradlew test` 全パス (Kotlin テスト)
2. trianglelist: Rust テスト (`cargo test --manifest-path rust-trilib/Cargo.toml`)
3. `cargo publish --dry-run` 各 crate エラーなし
4. crate 名の空き確認: `cargo search dxf-engine` → 取られていたら `road-dxf-engine` に変更

### リスク
- **パッケージ名衝突**: `dxf-engine` が crates.io で取られている可能性。代替名: `road-dxf-engine`, `dxf-gen`
- **trianglelist CI の git 依存**: GitHub Actions で private repo の git 依存が認証エラーになる可能性。対策: deploy key or crates.io publish 後に切替
- **API 破壊変更**: trianglelist が使う API を Phase 2-4 で変更すると切替時に大量修正。Phase 3 完了時に API を freeze する
- **csv_to_dxf Python版の継続**: Rust移植完了後も Python GUI は当面維持 (ユーザーがいる場合)

### 依存関係
- Phase 2, 3, 4 すべて完了後に実行
- trianglelist リポジトリへの書き込みアクセス必要

---

## アーキテクチャ全体図

```
                    ┌──────────────────┐
                    │  HTML/JS フロント │  Phase 4.5 (Tabulator + CodeMirror 6)
                    │  + Rust WASM     │
                    └────────┬─────────┘
                             │ wasm-bindgen
┌──────────┐    ┌────────────┴────────────┐    ┌──────────────┐
│  cli/    │    │    crates/              │    │ trianglelist │
│(Phase 1) │────│                         │────│  (Phase 5)   │
└──────────┘    │  excel-parser (P2)      │    └──────────────┘
   │            │  road-section (P1)      │
   │ --ai-parse │  triangle-core (P3)     │
   │            │  road-marking  (P3)     │
   ▼            │  dxf-engine    (P1)     │
┌──────────┐    └─────────────────────────┘
│ LLM層    │  calamine dump → LLM → マスタCSV (Phase 2.5)
│(P2.5)    │
└──────────┘

Native tools:
  dxf-viewer — DXFホットスワップビューワ (egui native, Issue #9)
```

**レイヤー依存:**
```
road-marking → triangle-core → dxf-engine
road-section → dxf-engine
excel-parser → (standalone, calamine)
LLM層 (P2.5) → excel-parser + cli-ai-analyzer
cli → excel-parser + road-section + triangle-core + road-marking
cli --ai-parse → LLM層
web (WASM) → excel-parser + road-section + dxf-engine + triangle-core
web (native) → dxf-engine + road-section (dxf-viewer)
```

---

## 優先度とスケジュール感

| Phase | 優先度 | 前提 | 規模感 |
|-------|--------|------|--------|
| **2** | ✅ 完了 | Phase 1 | 中 (4モジュール, 100テスト) |
| **2.5** | 高 — 未知書式対応 | Phase 2 | 中 (calamine dump + LLM + CLI) |
| **3** | ✅ 完了 | Phase 1 | 大 (triangle-core 151 + road-marking 88 + dxf-engine 251テスト) |
| **4** | ✅ 完了 | Phase 2+3 | 中 (Web 77テスト, WASM build + CI deploy済) |
| **4.5** | 次 — UI刷新 | Phase 4 | 中 (HTML/JS + Tabulator + WASM バインディング) |
| **5** | P3 — 安定後 | Phase 2+3+4 | 小 (依存切替 + publish) |

---

## 横断的な注意事項

1. **テストデータは既存リポジトリから借用**: `csv_to_dxf/data/`, `trianglelist/app/src/test/resources/`
2. **DXF座標系**: Y上向き (CAD標準)。スクリーン座標 (Y下向き) との変換に注意
3. **文字エンコーディング**: DXF出力は UTF-8、入力CSVは Shift_JIS/UTF-8 両対応
4. **浮動小数点**: 面積は小数2桁丸め、座標比較は 0.01 以下で一致判定
5. **ハンドル**: DXF ハンドルは 0x100 開始、16進大文字
6. **public リポジトリ注意**: 業務データ (工事CSV) をコミットしない。テスト用サンプルデータのみ
