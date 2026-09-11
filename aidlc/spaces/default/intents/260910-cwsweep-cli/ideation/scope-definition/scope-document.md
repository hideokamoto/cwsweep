# Scope Document — cwsweep

## In Scope（v1）

Must-have機能:

- `organizations:list-accounts`によるOrganization内対象アカウントの自動列挙
- 各メンバーアカウントへの`AssumeRole`（ロール名は引数で指定可能、デフォルト`OrganizationAccountAccessRole`）。管理アカウント自身は`AssumeRole`せず現在の認証情報をそのまま使用
- API呼び出し直前の`sts:get-caller-identity`による実アカウントID検証。不一致時は警告して続行せず、そのアカウントの処理を即座に失敗させる（構造的な取り違え防止）
- `cloudwatch-logs:describe-log-groups`を全ページ取得してから集計する層（全アカウント×全リージョン）。1ページだけ見て集計する実装が書けない構造的強制
- スキャン結果の対話式マルチセレクト（サイズ・retention・アカウントID・リージョンを表示、チェックボックス、初期状態は全チェックOFF）
- 選択ログループへのアクション選択: 削除（`delete-log-group`）またはretention設定変更（`put-retention-policy`）
- 実行前確認画面（対象アカウントID・リージョン・ログループ名一覧・合計バイト数の再掲）
- dry-runをデフォルトとし、明示的な`--execute`フラグ以外では削除・変更APIを呼ばない
- 削除直前の二重Identity検証（スキャン時点と削除実行時点の再検証）
- 監査ログの必須出力（対象アカウントID・リージョン・ログループ名・実行時刻・成功/失敗。無効化オプションなし）
- スキャン結果の`table`/`json`出力切り替え

[Q2]

## Out of Scope（v1）

- CloudWatch Logs以外のリソース種別（EC2、S3、EBS等）
- 削除条件の自動判定（ルールベースの自動削除）
- CloudWatch OAM自体のセットアップ
- スケジュール実行・常駐化

[Q2]

## MVPスコープの境界

初回リリース（MVP）は、PRDのマイルストーンM1〜M5（Organizations一覧取得+AssumeRole+Identity検証、全ページ取得の集計、table/json出力、対話式マルチセレクトUI、dry-run表示）までとし、M6（`--execute`フラグによる実削除・retention変更）は別段階として扱う。実Org環境での削除候補確認を経てからM6に着手する。[Q1]

## シーケンシング方針

リスク優先で進める。まずPRD背景にある2つの構造的事故（`AWS_PROFILE`によるクレデンシャル取り違え、`describe-log-groups`のページネーション未対応）を構造的に解消する設計・実装（M1・M2相当）を最優先し、その基盤の上に価値提供機能（出力形式・対話式UI・削除実行）を積み上げる。[Q4]

## 依存関係

M1（認証+Identity検証）→ M2（全ページ集計）→ M3（table/json出力）→ M4（対話式マルチセレクトUI）→ M5（dry-run表示）→ M6（実削除・retention変更、本スコープ外）の順に依存する。[Q3]

## デッドライン

機能個別のハードデッドラインはなし。PRDに記載の90日後判定（実運用で3回以上使われたか、撤退条件に該当しないか）が唯一の目安。[Q5]

## Assumptions & Open Questions

None.
