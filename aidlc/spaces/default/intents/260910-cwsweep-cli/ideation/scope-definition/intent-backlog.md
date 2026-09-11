# Intent Backlog — cwsweep（Proto-Units, MoSCoW優先度）

優先順位はリスク優先のシーケンシング方針に従い、PRDのマイルストーンM1〜M5の順で並べる（[Q1][Q3][Q4]）。

| # | Proto-Unit | 内容 | MoSCoW | 対応マイルストーン |
|---|---|---|---|---|
| 1 | Organization探索 + Identity検証基盤 | `organizations:list-accounts`、`AssumeRole`（明示的CredentialsProvider）、API呼び出し直前の`sts:get-caller-identity`検証・不一致時の即時失敗 | Must | M1 |
| 2 | ページネーション完全対応の集計層 | `describe-log-groups`の全ページ取得後にメモリ上で集計する独立層 | Must | M2 |
| 3 | 出力形式（table/json） | スキャン結果の`table`/`json`出力切り替え | Must | M3 |
| 4 | 対話式マルチセレクトUI | サイズ・retention・アカウントID・リージョン表示、初期状態全チェックOFF | Must | M4 |
| 5 | dry-run表示 | `--execute`未指定時は削除予定一覧のみ表示、書き込みAPIを呼ばない | Must | M5 |
| 6 | 実削除・retention変更（`--execute`） | `delete-log-group`/`put-retention-policy`の実行、削除直前の二重Identity検証、実行前確認画面 | Must（別段階） | M6 |
| 7 | 監査ログ出力 | 実行結果（アカウントID・リージョン・ログループ名・時刻・成功/失敗）の必須ファイル出力 | Must（別段階、M6と同時） | M6 |
| 8 | CloudWatch Logs以外のリソース対応 | EC2/S3/EBS等への拡張 | Won't（v1） | Out of Scope |
| 9 | 削除条件の自動判定 | ルールベース自動削除 | Won't（v1） | Out of Scope |
| 10 | OAMセットアップ支援 | CloudWatch OAM自体の設定 | Won't（v1） | Out of Scope |
| 11 | スケジュール実行・常駐化 | 定期実行・デーモン化 | Won't（v1） | Out of Scope |

[Q2]

## Assumptions & Open Questions

None.
