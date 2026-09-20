# reliability-requirements.md — Unit: cli-foundation

## 詳細要件

| ID | 要件 | 目標 | 検証 |
|---|---|---|---|
| NFR5.10 | 終了コード契約（R-03）を維持: 対象 1 件以上かつ全件失敗のみ非ゼロ、部分失敗は 0 終了＋警告ログ、対象 0 件は 0 終了 | `scan` / `clean` 共通 | `scan_fully_failed` の既存 lib テストを維持し、`run_scan` 経由でも同一判定であることをテスト |
| NFR5.11 | 引数誤用（サブコマンド未指定・所属外オプション・`--regions` 欠落・旧フラグ）は AWS I/O 開始前に使用方法エラーで終了する | 誤用でネットワーク呼び出し 0 | lib テストで `parse_from_args` が `Err` を返すことを固定化 |
| NFR5.12 | 監査ログ出力先のオープン失敗は `clean` の即時失敗とし、スキャンにも進まない | フェイルクローズ | main 配線順序（open → run_clean）をコードレビューで確認 |
| NFR5.13 | `audit` は監査ログ不在を空結果として正常終了し、読み取り I/O エラーは非ゼロ終了する | Contract 1 継承 | audit-reader 既存テスト＋`run_audit` の Err 伝播テスト |
| NFR5.14 | 非 TTY 環境の `clean` は破壊的操作へ進まず正常終了する（フェイルセーフ） | 誤用時に API 呼び出し 0 | 設計（functional-spec ワークフロー2 ステップ4）・コードレビュー（TTY 判定は main 側の入力で lib へ渡す） |

## 障害時の振る舞い

- AssumeRole 失敗・describe 失敗は該当アカウント×リージョンのみ `Failed` とし、
  他は継続（既存バルクヘッド分離、`rules.md` BR2.3 / BR5.1）。
- 実行フェーズの資格情報解決失敗も該当アクションのみ失敗として記録し、他は継続
  （既存 `CliApp::execute` の方針を `run_clean` がそのまま利用）。
