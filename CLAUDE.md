# road-drawing

## 問題領域

アスファルト舗装工事の**出来形管理書類**(面積数量+図面)を、Excel/CSV から DXF に変換する道具。
施工後に発注者へ提出する「どこを舗装したか・面積はいくつか」の書類作成を 1 コマンドに圧縮する。

スコープ:
- **路線展開図**(`road-section`):測点ごとの左右幅員から台形公式で区間面積を積算する「面積展開図」。`区間X,台形計算` ヘッダで複数区間を 1 ファイルに収める慣習に対応。
- **三角形リスト面積計算**(`triangle-core`):路線展開ではすくえない不整形部分(交差点擦り付け、隅切り等)をヘロンの公式で算定。
- **区画線展開図**(`road-marking`):横断歩道・停止線・矢印等の白線位置と寸法。

**将来スコープ(未サポート図面形式)**:縦断図、横断構造図(断面構造)、構造物図(擁壁・側溝・桝等)。
舗装管理という主題には収まるが現状未実装、いずれ追加対象。

**完全対象外**:新設道路の設計・測量自体。これは別ドメインで扱わない。
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
