# Plan and Review

計画を作成し、サブエージェントでAIレビューを通してからユーザーに提示する。

## When to Use
- 新しいPhaseやIssueに着手する前
- 大きな設計変更を行う前

## Process

### Phase 1: Context Gathering
1. GitHub Issue を読む (`gh issue view <number>`)
2. 関連する既存コードを調査
3. PLAN.md の既存内容を確認

### Phase 2: Plan Drafting
ユーザーと1対1で壁打ちしながら計画を固める。以下を含む:
- 変更対象ファイル一覧
- 新規作成ファイル一覧
- 依存クレートの追加
- 実装順序（依存関係を考慮）
- テスト戦略
- リスクと対策

計画は `.claude/plans/` に Markdown で書く。

### Phase 3: AI Review (Background Sub-agent)
plan-reviewer エージェントをバックグラウンドで起動し、計画をレビューさせる。
レビュー中もユーザーとの会話は続行可能。

### Phase 4: Triage
レビュー結果を仕分け:
- **Must Fix**: 計画に反映して修正
- **Optional**: ユーザーに判断を委ねる
- **Dismissed**: 理由を添えて却下

### Phase 5: Final Confirmation
修正済みの計画をユーザーに提示。承認を得たら実装フェーズへ。
