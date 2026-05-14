# road-drawing

## 起源と目的

このリポは 2 層の目的で動いている:

1. **技術目的(なぜこのリポが生まれたか)**:過去 3 つの別言語・別リポに散らばっていた図面生成機能(`csv_to_dxf` Python / `trianglelist` Kotlin / `trianglelist-web` Rust+rust-dxf)を、Rust の単一ワークスペースに**統合し再利用可能な crate 群として揃える**。DXF 出力ロジックを `dxf-engine` 1 つに集約し、図面種別ごとの crate を他プロジェクトから単体で呼べるようにする。最終 Phase 5 で `triangle-core` を crates.io 公開、`trianglelist` 等が逆向きに本リポへ依存する形へ移行。
2. **ドメイン目的(何のために使うか)**:アスファルト舗装工事の**出来形管理書類**(面積数量+図面)を、Excel/CSV から DXF に変換する道具として日々使う。施工後に発注者へ提出する「どこを舗装したか・面積はいくつか」の書類作成を 1 コマンドに圧縮する。

## 現スコープ(扱う図面)

- **路線展開図**(`road-section`):測点ごとの左右幅員から台形公式で区間面積を積算する「面積展開図」。`区間X,台形計算` ヘッダで複数区間を 1 ファイルに収める慣習に対応。
- **三角形リスト面積計算**(`triangle-core`):路線展開ではすくえない不整形部分(交差点擦り付け、隅切り等)をヘロンの公式で算定。
- **区画線展開図**(`road-marking`):横断歩道・停止線・矢印等の白線位置と寸法。

**将来スコープ(未サポート図面形式)**:
- **縦断図**:道路中心線を進行方向に沿って側面から切った図(地盤高・計画高・勾配、土工量・排水計画用)
- **横断図**:縦断方向に直角に切った断面図(舗装の表層・基層・路盤厚、路面・路肩・法面勾配、品質確認用)

**完全対象外**:新設道路の設計・測量自体、および擁壁・側溝・集水桝などの構造物図(これらは別ドメインの個別構造物設計)。
あくまで既に施工した(または施工する)アスファルト舗装の面積管理が主題。

## Architecture

```
crates/
├── dxf-engine/     — DXFエンティティ生成・バリデーション・Lint
├── road-section/   — 路線展開図ジオメトリ計算
├── road-marking/   — 区画線生成（Phase 3で追加予定）
└── excel-parser/   — Excel入力パース（Phase 2で追加予定）
cli/                — road-drawing CLIツール（AIスキル層）
web/                — egui WASMアプリ（Phase 4で追加予定）
```

## Build

```bash
cargo build
cargo test
```

## CLI Usage

```bash
road-drawing generate --input data.csv --output output.dxf --type road-section
```

## Development Process

### Agent Teams 構成
- `.claude/agents/` にレビュワー定義
- `.claude/skills/` に3つの開発スキル:
  - `plan-and-review`: 計画→AIレビュー（サブエージェント）
  - `team-impl`: 実装→レビュー→修正サイクル（Agent Teams）
  - `team-review-fix`: 人間レビュー指摘の並列対応（Agent Teams）

### Team Leader は作業を持たない
リーダーの禁止事項: コード実装、コード調査、コードレビュー、テスト実行。
リーダーの仕事: タスク割り当て、進捗管理、ユーザーとの窓口。

### 共有ファイルでタスク可視化
- `status-board.md`: メンバーのステータス・担当ファイル
- `task-log.md`: タスク一覧・担当・結果

## Rules

- DXF出力は必ず `DxfLinter::is_valid()` で検証
- テストは `cargo test` 全パスが必須
- スケール変換は m→mm (×1000) で統一
- 測点名ラベルは DXF color 5 (blue)
- GitHub Issues に対応するブランチで作業し、PRを作成
