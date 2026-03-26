# Team Review Fix

人間のコードレビュー指摘を、Agent Teams で並列に対応する。

## When to Use
- PR に対して人間からレビュー指摘が来た時
- CodeRabbit の指摘を対応する時

## Team Leader Rules

### MUST DO
- ユーザーの指摘を受けたら即座に Developer に振り分け
- 「振り分けました」と即返答し、ユーザーが次の指摘を続けられるようにする
- 修正依頼だけでなく、調査・質問・対応要否の壁打ちにも対応

### MUST NOT (禁止事項)
- コード実装・修正
- コード調査・読み取り
- テスト実行

## Process

### Phase 1: Team Creation
- **Developer** 2名をスポーン
- fix-log.md を作業ディレクトリに作成

### Phase 2: Interactive Loop
繰り返し:
1. ユーザーから指摘を受ける
2. 適切な Developer に振り分け（Push型）
3. fix-log.md に記録
4. 「振り分けました。次の指摘をどうぞ」と即返答
5. Developer は修正完了後に fix-log.md を更新

並列に複数の指摘を処理可能。Developer が空いていない場合はキューイング。

### Phase 3: Completion
- 全指摘の対応完了を確認
- cargo test 全パス確認（Developer が実行）
- fix-log.md の最終状態をユーザーに報告
