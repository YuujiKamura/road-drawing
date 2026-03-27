# SekouTaiseiMaker エディタ調査レポート

調査日: 2026-03-27
対象: `~/SekouTaiseiMaker/pdf-editor.js`, `src/views/pdf_editor.rs`
目的: road-drawing Phase 4 egui DXFビューアで流用可能なUI設計パターンの特定

## 1. エディタ機能一覧

### 実装一覧（3系統）

| 実装 | ファイル | フレームワーク |
|------|---------|-------------|
| JS版 (原型) | `pdf-editor.js` | Vanilla JS IIFE |
| React版 | `react-app/src/components/PdfEditor.tsx` | React/TypeScript |
| Rust版 (本番) | `src/views/pdf_editor.rs` | Leptos/WASM |

### 注釈機能

| 機能 | 説明 | コード参照 |
|------|------|-----------|
| テキスト配置 | クリック位置にテキスト挿入。フォントサイズ(10-24pt)、書体(明朝/ゴシック)選択可 | `pdf_editor.rs:610-733` |
| 矩形描画 | ドラッグで矩形作成。カラーピッカーで色選択。5x5px未満は棄却 | `pdf_editor.rs:606-708` |
| 選択→移動 | クリックで選択(AABB判定)→ドラッグで移動。差分計算方式 | `pdf_editor.rs:542-672` |
| 削除 | 選択中のオブジェクトをDeleteキーまたはボタンで削除 | `pdf_editor.rs:797-810` |
| Undo | スナップショット方式。全注釈のclone保存、最大50エントリ。Ctrl+Z | `pdf_editor.rs:507-511` |
| プロパティ編集 | 選択中テキストのフォント/サイズ/内容をツールバーから即時更新 | `pdf_editor.rs:950-966` |
| ピンチズーム | 2本指でズーム(0.25x-4.0x)。初期距離比でスケール計算 | `pdf_editor.rs:741-795` |
| PDF保存 | pdf-libで注釈をPDFに書き込み→ダウンロード。フォントサブセット対応 | `pdf_editor.rs:1105-1274` |

### EditorMode

```rust
enum EditorMode { Text, Rect, Select }
```
3つの排他モード。mousedown/move/upの挙動がモード別に分岐 (`pdf_editor.rs:67-72`)。

## 2. 依存ライブラリ

| ライブラリ | 用途 | ロード方法 |
|-----------|------|-----------|
| PDF.js | PDFページをCanvasにレンダリング | CDN (`js/pdf-libs-loader.js`) |
| pdf-lib | 注釈をPDFに書き込み(保存) | CDN |
| @pdf-lib/fontkit | カスタムフォント埋め込み | CDN (global `fontkit`) |
| font-subset.js | フォントサブセット生成(使用文字のみ) | `wasm_bindgen(module)` |
| Leptos | リアクティブUI (create_signal, create_effect, spawn_local) | Cargo依存 |

### レンダリングアーキテクチャ

デュアルCanvas方式:
```
<canvas class="pdf-canvas">      ← PDF.js が背景ページを描画 (タッチ不可)
<canvas class="overlay-canvas">  ← 注釈をここに描画 (全イベント受信)
```
- オーバーレイは状態変更のたびに全消去→全再描画
- `create_effect` でリアクティブに再描画トリガー

## 3. 座標系と変換

### 2つの座標空間

| 空間 | 原点 | Y方向 | 用途 |
|------|------|-------|------|
| Screen | Canvas左上 | 下向き | マウスイベント、描画 |
| Document (PDF) | ページ左下 | 上向き | オブジェクト保存、PDF書き出し |

### 変換規則

**配置時 (Screen→Document):**
```
ann.x = screen_x / zoom
ann.y = screen_y / zoom
```
`pdf_editor.rs:712-713`

**描画時 (Document→Screen):**
```
draw_x = ann.x * zoom
draw_y = ann.y * zoom
```
`pdf_editor.rs:413-416`

**PDF書き出し (回転対応):**
```
0°:   (x, page_height - y)
90°:  (y, x)
180°: (page_width - x, y)
270°: (page_height - y, page_width - x)
```
`pdf_editor_interop.rs:214-245`

### DPI対応
JS版: `devicePixelRatio` でCanvas内部解像度を倍率、CSS論理サイズは固定 (`pdf-editor.js:233-249`)
Rust版: PDF.js viewportに委任

## 4. road-drawing Phase 4 egui DXFビューアへの流用

### 直接移植可能なパターン (5つ)

#### 4.1 座標空間分離 (最優先)

**原理**: オブジェクトはドキュメント座標(DXF世界単位)で保存。描画時にzoom倍。

road-drawingの `web/src/renderer.rs` の `Viewport::to_screen()` が既にこれを実装済み:
```rust
// DXF Y-up → screen Y-down
screen_y = origin_y + (max_dxf_y - dxf_y) * scale
```

**流用方法**: SekouTaiseiMakerの `zoom * ann.x` パターンと同一。Viewport構造体にzoom操作を追加すればエディタのパン/ズームが完成。

#### 4.2 EditorMode enum + イベントディスパッチ

```rust
// SekouTaiseiMaker
enum EditorMode { Text, Rect, Select }

// road-drawing DXFエディタ用に拡張
enum DxfEditorTool { Select, Move, AddLine, AddDimension, AddText }
```

mousedown/move/up のモード別分岐パターン (`pdf_editor.rs:537-616`) をeguiの `Response` ベースに変換:
```rust
let response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());
if response.clicked() { match tool { ... } }
if response.dragged() { match tool { ... } }
```

#### 4.3 Undo スナップショット

```rust
struct HistoryEntry {
    entities: Vec<DxfEntity>,  // 全エンティティのclone
}
let mut history: Vec<HistoryEntry> = Vec::new();  // max 50
```
`pdf_editor.rs:75-79, 507-511` — 変更前にpush、Ctrl+Zでpop。egui側も同一実装。

#### 4.4 AABB ヒットテスト + 線分近接テスト

SekouTaiseiMaker: 矩形判定のみ (`pdf_editor.rs:544-572`)
DXF用追加: `point_to_line_distance(p, line_start, line_end) < threshold`

```rust
// SekouTaiseiMaker のAABB判定そのまま
fn hit_test_rect(p: Pos2, rect: &DxfRect) -> bool {
    p.x >= rect.x && p.x <= rect.x + rect.w
    && p.y >= rect.y && p.y <= rect.y + rect.h
}
// DXF用追加: 線分近接
fn hit_test_line(p: Pos2, a: Pos2, b: Pos2, tolerance: f32) -> bool {
    point_to_segment_distance(p, a, b) < tolerance
}
```

#### 4.5 ドラッグ状態管理

```rust
// SekouTaiseiMaker
drag_state: Option<(String, f64, f64)>  // (id, last_x, last_y)

// egui版
struct DragState { entity_id: usize, last_pos: Pos2 }
drag: Option<DragState>
```
`pdf_editor.rs:600-672` の差分計算パターンをそのまま使用。

### 不要なもの (流用しない)

| SekouTaiseiMaker | 理由 |
|------------------|------|
| PDF.js / pdf-lib | DXFビューアにPDFは不要 |
| フォントサブセット | DXF書き出しに不要 |
| デュアルCanvas | egui は即時モード描画。Painter 1パスで背景+オーバーレイ |
| Leptos create_effect | egui は毎フレーム再描画。リアクティブ不要 |
| localStorage永続化 | egui State + serde で代替 |

### road-drawingに既にある基盤

| 既存コード | 場所 | 状態 |
|-----------|------|------|
| Viewport変換 | `web/src/renderer.rs` | 完成 |
| DXF色マッピング | `web/src/renderer.rs:93-104` | 完成 |
| 線分/テキスト描画 | `web/src/renderer.rs` | 完成 |
| DXFエンティティモデル | `crates/dxf-engine/src/dxf/entities.rs` | 完成 |
| DXF書き出し | `web/src/dxf_export.rs` | 完成 |
| egui WASM基盤 | `web/src/lib.rs` + `Trunk.toml` | 完成 |

## 5. 結論

**新規実装が必要なのは「エディタ操作層」のみ。**

描画・データモデル・WASM基盤は既存。SekouTaiseiMakerから以下5パターンを移植すればDXFプレビューエディタの骨格が完成:

1. 座標空間分離 (zoom * document_coord)
2. EditorTool enum + イベントディスパッチ
3. Undo スナップショット (Vec<HistoryEntry>, max 50)
4. AABB + 線分近接ヒットテスト
5. Option<DragState> ドラッグ管理

推定工数: エディタ操作層の実装 = 500-800行 (renderer.rs + app.rs への追加)
