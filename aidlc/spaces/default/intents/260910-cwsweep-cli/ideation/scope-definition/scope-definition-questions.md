# Scope Definition — Questions

## Q1. 価値を提供する最小限のスコープ（MVP）は何ですか？

A. PRDのM1〜M5相当: Organizations一覧取得+AssumeRole+Identity検証、全ページ取得の集計、table/json出力、対話式マルチセレクトUI、dry-run表示（削除予定一覧のみ・実行はしない）まで。実削除（M6）は別段階とする。
B. M6（実削除）まで含めて初回リリースとする
X. Other (please specify)

[Answer]: A. PRDのM1〜M5相当: Organizations一覧取得+AssumeRole+Identity検証、全ページ取得の集計、table/json出力、対話式マルチセレクトUI、dry-run表示（削除予定一覧のみ・実行はしない）まで。実削除（M6）は別段階とする。

## Q2. Must-have と Nice-to-have の機能は何ですか？

Must-have（PRD In Scope）:
- Organizations横断アカウント自動列挙、AssumeRole（管理アカウントは現在の認証情報を使用）
- API呼び出し直前の`sts:get-caller-identity`による実アカウントID検証（不一致は即失敗）
- `describe-log-groups`の全ページ取得後の集計（1ページのみの集計を構造的に禁止する設計）
- スキャン結果の対話式マルチセレクト（初期状態全チェックOFF）
- 削除（`delete-log-group`）またはretention変更（`put-retention-policy`）のアクション選択
- 実行前確認画面（アカウントID・リージョン・ログループ名一覧・合計バイト数の再掲）
- dry-runをデフォルトとし、`--execute`明示時のみ書き込みAPIを呼ぶ
- 削除直前の二重Identity検証
- 監査ログの必須出力（無効化オプションなし）
- `table`/`json`出力切り替え

Nice-to-have（Out of Scope, v1では対象外）:
- CloudWatch Logs以外のリソース種別（EC2/S3/EBS等）
- 削除条件の自動判定（ルールベース自動化）
- CloudWatch OAM自体のセットアップ
- スケジュール実行・常駐化

A. 上記の通りで正しい
B. 変更したい項目がある（自由記述で補足する）
X. Other (please specify)

[Answer]: A. 上記の通りで正しい

## Q3. 機能間の依存関係は何ですか？

A. PRDのマイルストーン順（M1: 認証+Identity検証 → M2: 全ページ集計 → M3: 出力形式 → M4: 対話式UI → M5: dry-run → M6: 実削除）が自然な依存順序であり、それに従う
B. 別の順序にしたい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. PRDのマイルストーン順（M1: 認証+Identity検証 → M2: 全ページ集計 → M3: 出力形式 → M4: 対話式UI → M5: dry-run → M6: 実削除）が自然な依存順序であり、それに従う

## Q4. シーケンシング（リスク優先・価値優先・依存関係優先）の希望は？

A. リスク優先: PRD背景にある2つの構造的事故（認証取り違え、ページネーション未対応）を最初に構造的に解消する設計（M1・M2）を最優先し、その上に価値提供機能（対話式UI・削除実行）を積み上げる
B. 価値優先: 対話式削除機能を早期に動かせる形にすることを優先する
X. Other (please specify)

[Answer]: A. リスク優先: PRD背景にある2つの構造的事故（認証取り違え、ページネーション未対応）を最初に構造的に解消する設計（M1・M2）を最優先し、その上に価値提供機能（対話式UI・削除実行）を積み上げる

## Q5. 特定機能に紐づくハードデッドラインはありますか？

A. なし。90日後（成功指標・撤退条件の判定タイミング）が唯一の目安であり、機能個別のデッドラインではない
B. None
X. Other (please specify)

[Answer]: A. なし。90日後（成功指標・撤退条件の判定タイミング）が唯一の目安であり、機能個別のデッドラインではない

---

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
