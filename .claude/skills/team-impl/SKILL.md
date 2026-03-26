# Team Implementation

承認済み計画に基づき、Agent Teams で並列実装→AIレビュー→修正サイクルを回す。

## When to Use
- 計画が承認された後の実装フェーズ
- 複数ファイルにまたがる変更

## Team Leader Rules

### MUST DO
- ユーザーの要望を汲み取りメンバーにタスクをPush割り当て
- status-board.md と task-log.md を管理
- メンバーが困っていたらサポート
- 判断が必要なものはユーザーに確認
- コンフリクト発生時にメンバー間のタスク調整

### MUST NOT (禁止事項)
- コード実装・修正
- コード調査・読み取り
- コードレビュー
- テスト実行
- cargo build / cargo test の直接実行

## Process

### Phase 1: Preparation
1. 計画ファイルを読み込む
2. 並列度を分析（独立して実装可能な単位を特定）
3. 作業ディレクトリに共有ファイルを作成:
   - `status-board.md` — 各メンバーのステータス・現在タスク・触っているファイル
   - `task-log.md` — 全タスク一覧、担当、ステータス、結果

### Phase 2: Team Creation
メンバー構成:
- **Developer** (1-2名): 計画の並列度に応じて動的決定
- **dxf-reviewer** (1名): DXF仕様準拠チェック
- **test-reviewer** (1名): テスト品質チェック

スキーマ/API変更がある場合は追加レビュワーを検討。

### Phase 3: Implementation
- Developer(s) が計画に沿って並列実装
- 各 Developer は担当ファイルを status-board.md に記録してから作業開始
- 完了時に task-log.md を更新しリーダーに報告

### Phase 4: AI Review
- Reviewer(s) が観点別に並列レビュー
- レビュー結果は review-log.md に記録
- 全レビュワーの指摘を task-log.md に集約

### Phase 5: Fix Cycle
- 指摘を Developer に振り分けて修正
- 修正完了 → 再レビュー
- 全員承認まで繰り返し（最大3サイクル）

### Phase 6: Completion
- cargo test 全パス確認（Developer が実行）
- ユーザーに完了報告:
  - 変更ファイル一覧
  - テスト結果
  - 残存する人間判断が必要な事項
