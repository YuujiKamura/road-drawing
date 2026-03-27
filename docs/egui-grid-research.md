# egui Grid/Table Research for CSV Editor

**Target**: dxf_viewer.rs (native egui app) に編集可能なCSVグリッドを追加
**egui version**: 0.29.1
**egui_extras version**: 0.29.1

## 1. egui_extras::TableBuilder API (0.29.1)

### Column Sizing

```rust
use egui_extras::{TableBuilder, Column};

// Fixed width
Column::exact(100.0)

// Initial width, resizable
Column::initial(120.0).resizable(true)

// Auto-size based on content
Column::auto()

// Take remaining space
Column::remainder()

// With constraints
Column::initial(100.0).range(50.0..=300.0).resizable(true).clip(true)
```

### Basic Table

```rust
TableBuilder::new(ui)
    .id_salt("csv_grid")
    .striped(true)
    .resizable(true)
    .column(Column::initial(90.0).resizable(true))   // 測点名
    .column(Column::initial(80.0).resizable(true))    // 単延長L
    .column(Column::initial(80.0).resizable(true))    // 幅員W
    .column(Column::initial(80.0).resizable(true))    // 幅員右
    .header(20.0, |mut header| {
        header.col(|ui| { ui.label("測点名"); });
        header.col(|ui| { ui.label("単延長L"); });
        header.col(|ui| { ui.label("幅員W"); });
        header.col(|ui| { ui.label("幅員右"); });
    })
    .body(|mut body| {
        // Virtual scrolling: only visible rows rendered
        body.rows(25.0, data.len(), |mut row| {
            let i = row.index();
            row.col(|ui| { ui.text_edit_singleline(&mut data[i].name); });
            row.col(|ui| { ui.text_edit_singleline(&mut data[i].x_str); });
            row.col(|ui| { ui.text_edit_singleline(&mut data[i].wl_str); });
            row.col(|ui| { ui.text_edit_singleline(&mut data[i].wr_str); });
        });
    });
```

### Key Methods

| Method | 説明 |
|--------|------|
| `body.row(height, callback)` | 単一行（非仮想スクロール） |
| `body.rows(height, count, callback)` | 仮想スクロール（大量行OK） |
| `body.heterogeneous_rows(heights, callback)` | 可変高さ行 |
| `row.col(callback)` | セル描画。`(Rect, Response)` を返す |
| `row.index()` | 現在の行インデックス |

### Scroll Options

```rust
TableBuilder::new(ui)
    .vscroll(true)                    // 縦スクロール有効
    .max_scroll_height(600.0)         // デフォルト800px。明示設定推奨
    .stick_to_bottom(false)           // 末尾追従
    .scroll_to_row(target, Some(Align::Center))  // 特定行にジャンプ
```

## 2. 編集可能セルの実装パターン

### パターンA: 常時TextEdit（シンプル、推奨）

```rust
struct CsvRow {
    name: String,
    x_str: String,   // f64ではなくStringで保持（入力中の"3."を壊さない）
    wl_str: String,
    wr_str: String,
}

// セル描画
row.col(|ui| {
    let response = ui.add(
        egui::TextEdit::singleline(&mut data[i].x_str)
            .desired_width(f32::INFINITY)  // セル幅いっぱい
            .horizontal_align(egui::Align::RIGHT)  // 数値は右寄せ
    );
    if response.lost_focus() {
        // フォーカス喪失時にバリデーション
        if let Err(_) = data[i].x_str.parse::<f64>() {
            data[i].x_str = "0.0".to_string();
        }
        trigger_preview_update = true;
    }
});
```

### パターンB: クリックで編集モード切替

```rust
row.col(|ui| {
    if editing_cell == Some((i, col)) {
        let r = ui.text_edit_singleline(&mut data[i].field);
        if r.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            editing_cell = None;
        }
    } else {
        let r = ui.label(&data[i].field);
        if r.double_clicked() {
            editing_cell = Some((i, col));
        }
    }
});
```

### パターンC: egui::Grid（小規模向け、仮想スクロールなし）

```rust
egui::Grid::new("csv_grid")
    .striped(true)
    .num_columns(4)
    .min_col_width(60.0)
    .show(ui, |ui| {
        for row in &mut data {
            ui.text_edit_singleline(&mut row.name);
            ui.text_edit_singleline(&mut row.x_str);
            ui.text_edit_singleline(&mut row.wl_str);
            ui.text_edit_singleline(&mut row.wr_str);
            ui.end_row();
        }
    });
```

**推奨: パターンA** — 常時TextEditが最もシンプルで、4列×100行以下なら十分。

## 3. 必要な依存

### Cargo.toml変更

```toml
[dependencies]
# 既存
egui = "0.29"
eframe = { version = "0.29", ... }

# 追加
egui_extras = "0.29"   # TableBuilder
```

`egui_extras` の features は不要（テーブルはデフォルトで含まれる）。
`image` feature は画像ローダー用なので、テーブルだけなら不要。

### 代替・補助crate

| Crate | 用途 | 状態 |
|-------|------|------|
| `egui_extras` 0.29 | TableBuilder（公式、安定） | **推奨** |
| `egui::Grid` | 小規模グリッド（egui本体に含む） | 100行以下なら可 |
| `egui_tabular` | カスタムセルエディタ、Undo/Redo | 実験的 |
| `egui-data-table` | CSV import/export、キーボードナビ | 重い |

## 4. 制限事項・注意点

### 既知の問題

1. **max_scroll_height デフォルト無制限** (0.29): 明示的に `.max_scroll_height()` を設定しないとウィンドウ全体を占有する可能性
2. **id_salt 必須**: 同一Ui内に複数TableBuilderがある場合、`id_salt()` で区別しないとスクロール位置が共有される
3. **水平スクロール非対応**: TableBuilder自体は横スクロールを持たない。カラム幅を `resizable(true)` にして対応
4. **rows() のクロージャ**: `FnMut(TableRow)` なので `&mut data[i]` への可変参照は問題ない
5. **TextEdit + 数値**: `f64` を直接バインドする `DragValue` もあるが、日本語入力との相性が悪い。`String` 保持 + `lost_focus` でパースが安全

### パフォーマンス

| 方法 | 100行 | 1000行 | 10000行 |
|------|-------|--------|---------|
| `body.rows()` (仮想スクロール) | 60fps | 60fps | 60fps |
| `body.row()` ループ | 60fps | 30fps | 5fps |
| `egui::Grid` | 60fps | 15fps | 使用不可 |

## 5. dxf_viewer.rsへの統合プラン

### レイアウト案

```
┌──────────────────────────────────────┐
│ [Open CSV] [Save DXF] [Auto-reload] │  ← ToolBar
├──────────────┬───────────────────────┤
│  CSV Grid    │                       │
│  (TableBld)  │    DXF Preview        │
│  4列×N行     │    (Canvas)           │
│  編集可能     │                       │
├──────────────┤                       │
│  Status Bar  │                       │
└──────────────┴───────────────────────┘
```

### 実装ステップ

1. `web/Cargo.toml` に `egui_extras = "0.29"` 追加（済み）
2. `dxf_viewer.rs` の `update()` で `SidePanel::left` を追加
3. パネル内に `TableBuilder` で4列グリッド描画
4. セル編集時に `road_section::parse_road_section_csv` → `calculate_road_section` → プレビュー更新
5. ファイルウォッチャーと排他：CSV編集中はウォッチャーを一時停止

### データフロー

```
[TableBuilder セル編集]
    ↓ lost_focus or Enter
[String → f64 バリデーション]
    ↓
[Vec<StationData> 再構築]
    ↓
[calculate_road_section()]
    ↓
[geometry_to_dxf() → DxfDocument]
    ↓
[Canvas再描画]
```
