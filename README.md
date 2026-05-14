# road-drawing

アスファルト舗装工事の出来形管理書類(面積数量計算+図面)を、Excel/CSV から DXF 図面に変換する道具。
同時に、過去に別言語・別リポに散らばっていた図面生成系の機能を **Rust の crate 群として再利用可能な形に揃える** 統合プロジェクトでもある。

## なぜこのリポがあるか(起源)

舗装管理に使ってきた図面生成ツールが、これまで 3 つの別プロジェクトに分散していた:

| 元リポ | 言語 | 担当 | 移植先 crate |
|---|---|---|---|
| `csv_to_dxf` | Python | 路線展開図 + CSV/xlsx パーサ | `road-section` + `excel-parser` + `dxf-engine` |
| `trianglelist` | Kotlin | 三角形リスト面積計算 | `triangle-core` |
| `trianglelist-web` | Rust(`rust-dxf` 依存) | WASM ビューワ | `web/`(`dxf-engine` ベースに再構築) |

別言語・別ランタイム・別 DXF 実装が並走していた状態を、Rust の単一ワークスペースに**統合し再利用可能な crate に揃える**ことで:

- DXF 出力ロジックを `dxf-engine` 1 つに集約(リンタ・空間インデックス・読み書きを共有)
- 図面種別ごとの crate を独立に呼べる(他プロジェクトから crate 単位で再利用可能)
- 最終的に元リポ側(`trianglelist` 等)が逆方向に依存するよう、Phase 5 で `triangle-core` を crates.io 公開し、`rust-dxf` からの依存を切替える計画

つまりこのリポは「**舗装管理用の道具**」(ドメイン目的)と「**散在していた図面生成資産の統合基盤**」(技術目的)の 2 層で同時に動いている。

## 何のためにあるか(問題領域)

アスファルト舗装工事では、施工完了後に発注者へ「どこを舗装したか・面積はいくつか」を出来形として提出する義務がある。担当者の手元には測点ごとの幅員や三角形分割の辺長を記録した Excel(または CSV)があり、それを発注者様式に貼り込める CAD 図面(DXF)に変換する必要がある。

このリポジトリは、その変換工程を Excel/CSV → DXF の 1 コマンドに圧縮するためのもの。あくまでアスファルト舗装の**面積管理(出来形)**にスコープを絞る ── 新設道路の設計・測量自体は別ドメインで扱わない。

## 扱う 3 種類の図面

| 図面種別 | 担当 crate | 何を描くか |
|---|---|---|
| **路線展開図** | `road-section` | 測点ごとの左右幅員を並べた図。隣接 2 測点の幅員平均 × 距離 = 台形面積。これを全測点間で積算する「面積展開図」。`区間X,台形計算` ヘッダで複数区間を 1 ファイルにまとめる慣習に対応。 |
| **三角形リスト面積計算** | `triangle-core` | 路線展開ではすくえない不整形部分(交差点擦り付け、隅切り等)を三角形に分割し、ヘロンの公式で面積を出す図。Kotlin の trianglelist から移植。 |
| **区画線展開図** | `road-marking` | 横断歩道・停止線・矢印など、舗装面に塗る白線の位置と寸法を展開した図。 |

### 将来スコープ(未サポート図面形式)

舗装管理という主題には収まるが、まだ実装していない図面形式:

- **縦断図**:道路中心線の縦方向(進行方向)に沿った高さ・勾配の図。
- **横断図(断面構造図)**:測点ごとの路床・路盤・表層を断面で見せる図(現在の路線展開図は「上から見た面積展開」、こちらは「横から見た構造断面」)。
- **構造物図**:擁壁・側溝・集水桝など、舗装に付随する構造物の図。

新設道路の設計・測量自体は別ドメインで対象外。

## アーキテクチャ

```
crates/
├── dxf-engine/      DXF 読み書き・リンタ・空間インデックス (238 tests)
├── road-section/    路線展開図ジオメトリ計算 (101 tests)
├── triangle-core/   三角形リスト面積計算 (147 tests)
├── excel-parser/    Excel/CSV 入力パース・台形計算セクション検出 (85 tests)
├── road-marking/    区画線(横断歩道等)生成 (44 tests)
├── file-watcher/    ファイル更新監視
└── road-drawing-wasm/  Web 版エンジン (51 tests)
cli/                 コマンドラインフロントエンド (6 tests)
web/                 egui WASM Web アプリ
```

依存方向(下流が上流を呼び返さない):

```
cli  ─→ excel-parser ─→ (calamine, encoding_rs)
     ─→ road-section ─→ dxf-engine
     ─→ road-marking ─→ triangle-core ─→ dxf-engine
web  ─→ dxf-engine + road-section + excel-parser + triangle-core
```

## ビルド

```bash
cargo build
cargo test    # 全 674 テスト
```

## CLI 使い方

```bash
# 路線展開図(複数区間 Excel から「区間1」を抜き出して DXF)
road-drawing generate --input 面積計算書.xlsx --output 区間1.dxf --type road-section --section "区間1"

# 縮尺指定(デフォルト 1:1000)
road-drawing generate --input data.csv --output out.dxf --scale 500

# 区間一覧の確認
road-drawing generate --input data.xlsx --output /dev/null --list-sections

# 三角形リスト面積計算
road-drawing generate --input triangles.csv --output triangle.dxf --type triangle

# 区画線(JSON コマンド)
road-drawing generate --input commands.json --output marking.dxf --type marking

# 出来形検証(発注者支給のゴールデン DXF と生成物の照合)
road-drawing verify --golden 様式.dxf --actual 区間1.dxf
```

## Web 版

```bash
# 開発時(ネイティブウィンドウ)
cargo run -p road-drawing-web

# WASM ビルド
cd web && trunk build --release

# ローカルプレビュー
cd web && trunk serve
```

master への push で GitHub Actions が trunk build + GitHub Pages デプロイを実行。
公開 URL: `https://yujikamura.github.io/road-drawing/`
有効化には Settings → Pages → Source: GitHub Actions の手動切替えが必要。

## 入力フォーマットの慣習

- **路線展開図 CSV**: 1 行 = 1 測点。列は「測点名・累積延長(m)・左幅員(m)・右幅員(m)」。日本語ヘッダ検出あり。
- **複数区間 Excel/CSV**: `区間X,台形計算` のヘッダで区切る。`--list-sections` で一覧化、`--section "区間2"` で抜き出し。
- **距離の累計/単スパン**: median heuristic で自動判定し、必要なら累計に変換。
- **エンコーディング**: Shift_JIS / UTF-8 自動検出(`encoding_rs`)。
- **三角形 CSV**: MIN(4 列)・CONN(6 列)・FULL(28 列)の 3 形式。FULL は現状 1〜6 列のみ使用、7〜28 列は無音で破棄(既知の制限)。
- **スケール**: 内部は m、DXF 出力は m → mm(×1000)変換が既定。

## 既知の制限(2026-05 時点)

1. **FULL 28 列 CSV**: 1〜6 列のみ使用、7〜28 列(name, color, 寸法アラインメント, 角度)は無音で破棄。
2. **DXF リンタ warnings 未発火**: `LintResult.warnings` フィールドは構造体に存在するが、書き込むコードパスがない。全部 error 扱い。
3. **WASM サイズ**: debug 6.2 MB。release で `wasm-opt` を回せば 2 MB 未満になるはずだが CI に未組込み。
4. **Phase 2.5(LLM 形式変換)未着手**: 任意 Excel 形式 → 正規 CSV の LLM 変換は未実装。
5. **Phase 5(crates.io 公開)保留**: triangle-core の crates.io 公開と trianglelist 依存切替が未着手。
6. **GitHub Pages**: 手動で Settings → Pages → Source: GitHub Actions に切替える必要あり。

## 開発ルール

- DXF 出力は必ず `DxfLinter::is_valid()` で検証
- `cargo test` 全パスが必須
- スケール変換は m → mm(×1000)で統一
- 測点名ラベルは DXF color 5(blue)
- GitHub Issues に対応するブランチで作業し PR を作成
