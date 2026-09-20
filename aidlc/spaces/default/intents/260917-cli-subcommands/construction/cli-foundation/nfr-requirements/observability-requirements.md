# observability-requirements.md — Unit: cli-foundation

## ログ標準

| ID | 要件 | 検証 |
|---|---|---|
| NFR5.15 | 診断ログ（`tracing`）は標準エラーへ、機械可読出力（`--output json`）は標準出力へ。`json` 出力時に標準出力へ人間向け文言を混ぜない | `run_scan` / `run_audit` の json 出力が単一 JSON 文書としてパース可能であることをテスト（`rules.md` BR2.4） |
| NFR5.16 | スキャン失敗はアカウント × リージョン単位で `warn` レベルにフィールド付き（`account_id`, `region`, `error`）で記録する（現行維持） | 既存挙動、コードレビュー |
| NFR5.17 | `clean` の監査ログ（intent / result の 2 行記録、`run_id` 付与）は現行 `AuditLogger` の書式を変更しない。`audit` サブコマンドはそれを読むだけで書き込まない | audit-reader Contract 1 と `run_audit` の依存限定（NFR2.3） |
| NFR5.18 | 非 TTY の `clean`、監査ログ不正行スキップなど「静かに何かをしなかった」事象は必ず標準エラーへ警告を出す | `rules.md` BR3.4 / BR4.4（audit-reader 側で出力済みの警告は再出力しない） |

## 非目標

- メトリクス・分散トレーシング・アラート・ダッシュボードは CLI ツールとして対象外。
