# Testing Policy

## 方針
全ての公開関数にユニットテストを書く。E2Eだけでは不十分。内部の期待するふるまい1つにつき1テスト。

## テスト命名規約
```
test_{module}_{function}_{scenario}
```
例: `test_triangle_area_equilateral`, `test_crosswalk_zero_stripes`, `test_reader_malformed_header`

## 名前衝突防止（複数エージェント並列時）
- テスト追加前に `grep -r 'fn test_関数名' crates/` で既存名を確認
- 同名テストが存在したら別名にする（`_v2` ではなくシナリオ名を変える）
- 同一ファイルに複数エージェントが書き込む場合はgit pullしてから作業

## カバレッジ基準
各公開関数について最低限:
1. **正常系**: 代表的な入力
2. **境界値**: 0, 1, 空, 最大値
3. **エラー系**: 不正入力、存在しないデータ、型不一致
4. **エッジケース**: ドメイン固有の特殊ケース

## crate別テスト要件

### dxf-engine
- `entities`: 全ビルダーメソッド、デフォルト値、Clone/PartialEq
- `writer`: 全エンティティ型のDXF出力、ハンドル一意性、セクション構造
- `reader`: Writer出力のラウンドトリップ、不正DXFのエラー処理、部分的DXF
- `linter`: 全LintErrorCode、警告/エラー分類、エッジケース
- `index`: 測点座標抽出、レイヤーフィルタ、バウンディングボックス、空DXF
- `handle`: 連番性、開始値カスタム、1000個生成の一意性

### triangle-core
- `triangle`: Heron面積(正三角形/直角/鋭角/鈍角/微小)、頂点配置(角度0/90/180)、不正三角形(辺長0/負/不等式違反)
- `connection`: type1/type2接続、3段以上チェーン、不正親参照、自己参照、辺長不一致
- `csv_loader`: MIN/CONN/FULL形式、ヘッダー有無、空行・コメント行、不正数値、文字化け

### road-section
- `calculate_road_section`: 1測点/2測点/多数測点、幅員0/片側のみ/左右非対称、スケール変換精度
- `parse_road_section_csv`: ヘッダー有無、日本語ヘッダー、列不足、空行
- `geometry_to_dxf`: テキスト回転(-90°)、色(blue=5)、alignment

### road-marking
- `crosswalk`: ストライプ数0/1/奇数/偶数、オフセット0/負/中心線超過、アンカー全種類
- `command`: JSON parse成功/失敗/不完全、未知type、station-based配置、パラメータ欠落

### excel-parser
- `section_detector`: 複数区間/区間なし/重複区間名、Shift_JIS、列名バリエーション
- `station_name`: ピッチ20m境界、小数点丸め、既存名保持+ギャップ
- `distance`: 累積判定の境界(中央値ちょうど16m)、1行/2行データ、降順データ
- `transform`: パイプライン全段通過、各段の独立テスト

## テスト実行
```bash
cargo test                    # 全テスト
cargo test -p <crate>         # crate単位
cargo test <test_name>        # 個別テスト
```

## DXF出力の検証
テストで生成したDXFは必ず `DxfLinter::is_valid()` で検証すること。
