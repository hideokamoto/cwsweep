# Domain Design — Questions

要件分析（requirements-analysis）とユニット生成（units-generation）はスコープ上SKIPされているため、PRD・Scope Document・Intent Backlogを根拠として、以下のコンポーネント分解を提案します。

## Q1. コンポーネント境界について

以下の12コンポーネントに分解する案でよいですか？

1. **CliApp** — clapベースのエントリポイント。サブコマンド解析、全体のオーケストレーション
2. **OrgDiscovery** — `organizations:list-accounts`によるアクティブアカウント列挙
3. **CredentialProvider** — アカウントごとの明示的CredentialsProvider構築（AssumeRole、管理アカウントは現在の認証情報）
4. **IdentityVerifier** — API呼び出し直前の`sts:get-caller-identity`検証、不一致時の即時失敗
5. **LogGroupScanner** — `describe-log-groups`の全ページ取得後集計（1ページのみの集計を構造的に禁止する独立層）
6. **ScanAggregator** — 全アカウント×全リージョンのスキャン結果をメモリ上で保持・集約するモデル
7. **OutputFormatter** — table/json出力切り替え
8. **InteractiveSelector** — 対話式マルチセレクト（初期状態全チェックOFF）
9. **ActionPlanner** — 選択されたログループと選択アクション（削除/retention変更）から実行計画を構築
10. **ConfirmationPresenter** — 実行前確認画面（アカウントID・リージョン・ログループ名一覧・合計バイト数の再掲）
11. **ExecutionEngine** — dry-run既定、`--execute`時のみ書き込みAPI（`delete-log-group`/`put-retention-policy`）を呼び出し、削除直前の二重Identity検証を実施
12. **AuditLogger** — 実行結果の監査ログ必須出力、書き込み失敗時は実行中の操作を中断

A. はい、この12コンポーネント分解でよい
B. 変更したい点がある（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この12コンポーネント分解でよい

## Q2. エンティティ所有権について

- **ScanAggregator**が`LogGroupRecord`（アカウントID・リージョン・ログループ名・サイズ・retention日数）を所有する
- **ActionPlanner**が`PlannedAction`（対象LogGroupRecord・アクション種別・実行前確認状態）を所有する
- **AuditLogger**が`AuditEntry`（実行時刻・アカウントID・リージョン・ログループ名・アクション・成功/失敗）を所有する

A. はい、この所有権でよい
B. 変更したい点がある（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この所有権でよい

## Q3. コンポーネント間の相互作用について

CliApp起点で「OrgDiscovery→CredentialProvider→IdentityVerifier→LogGroupScanner→ScanAggregator→OutputFormatter/InteractiveSelector→ActionPlanner→ConfirmationPresenter→ExecutionEngine（内部でIdentityVerifierを再度呼び出し二重検証）→AuditLogger」という一方向の同期パイプラインでよいですか？

A. はい、この一方向パイプラインでよい
B. 変更したい点がある（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この一方向パイプラインでよい

## Q4. UI構造について（CLIのため簡略）

InteractiveSelectorとConfirmationPresenterはターミナルUI（`inquire`/`dialoguer`クレート想定）として、UXデザイナーの観点から「初期状態は全チェックOFF、選択なしで確定操作に進めない」という制約を明示する程度でよいですか？

A. はい、それで十分
X. Other (please specify)

[Answer]: A. はい、それで十分

---

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
