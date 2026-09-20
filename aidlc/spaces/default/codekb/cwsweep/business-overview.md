# business-overview.md — cwsweep

## ビジネスドメイン

`cwsweep` は、AWS Organizations 配下の複数メンバーアカウント・複数リージョンに
またがる **Amazon CloudWatch Logs のロググループ棚卸しと削除・保持期間（retention）
変更**を安全に行うための、スタンドアロン配布の Rust 製 CLI ツールである。

対象ドメインは「AWS コスト最適化・ログライフサイクル運用」で、具体的には
組織横断で放置された/肥大化した CloudWatch ロググループを可視化し、破壊的操作
（削除・retention変更）を人手の確認を経て安全に実行することを目的とする。

## 目的

- AWS Organization 全体（管理アカウント＋複数メンバーアカウント）× 複数リージョンの
  CloudWatch ロググループを一括スキャンし、サイズ順に集計・可視化する。
- 対話式マルチセレクトで対象ロググループをユーザー自身に選ばせ、削除または
  retention 変更のアクションプランを生成する。
- 実行前に対象一覧・合計バイト数を再掲した確認画面を提示し、明示的な承認を得た
  上でのみ実行する。
- すべての破壊的操作を JSON Lines 形式の監査ログに記録し、誰が・いつ・どの
  ロググループに対して・成功したか失敗したかを追跡可能にする。

## 主要機能

1. **スキャン**: Organizations の `list-accounts`（アクティブアカウントのみ）を起点に、
   各メンバーアカウントへ `AssumeRole`（既定ロール名 `OrganizationAccountAccessRole`）
   した上で、指定リージョンごとに CloudWatch Logs `describe-log-groups` を
   ページネーション完了まで実行し集計する。
2. **表示**: table（既定、`comfy-table`）または JSON（CLI エージェント向け）で
   スキャン結果を出力する。
3. **選択**: 対話式マルチセレクト（`inquire`、初期状態は常に全チェック OFF）で
   削除/retention変更の対象ロググループをユーザーが選ぶ。
4. **プラン生成・確認**: 選択結果からアクションプラン（削除 or retention 日数設定）
   を組み立て、対象アカウントID・リージョン・ログループ名一覧・合計バイト数を
   再掲した確認画面を提示する。
5. **実行**: `--execute` を明示した場合のみ、二重目の Identity 検証を経て
   `delete-log-group` / `put-retention-policy` を実行する（既定は dry-run）。
6. **監査**: 対象アカウントID・リージョン・ログループ名・実行時刻・成功/失敗を
   JSON Lines で記録し、無効化オプションを設けない。

## 想定利用者・利用シーン

- AWS Organization を運用する SRE/プラットフォームチームによる、定期的な
  CloudWatch Logs コスト棚卸し・クリーンアップ作業。
- CI/自動化パイプラインからの `--output json` / `--scan-only` によるスキャン専用実行
  （非TTY環境では自動的に scan-only 相当のフォールバックが働く）。

## 本 intent (`260917-cli-subcommands`) との関係

現状の CLI は `clap` によるフラットなフラグ構成（`--scan-only` と `--execute` が
相互排他）の単一コマンドであり、`scan` / `clean` / `audit` といったサブコマンドへの
再構成が本 intent のスコープである。詳細は `architecture.md` の
「サブコマンド化に向けた設計上の要点」および `api-documentation.md` を参照。
