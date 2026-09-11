# NFR Design — Questions

NFR Requirementsとdomain-designのcomponents.mdを根拠に、具体的な設計解を提案します。

## Q1. レジリエンスパターン（リトライ・部分障害時の分離）について

以下の設計でよいですか？
- CredentialProvider/LogGroupScanner/ExecutionEngineでのAWS API呼び出しはすべて`aws-config`の標準リトライ（最大3回、指数バックオフ）でラップする
- 1アカウントの処理は独立した`Result<AccountScanOutcome, AccountError>`として扱い、CliApp側でアカウント単位にループしてエラーを収集する（1アカウントの失敗が他アカウントの処理を止めない「バルクヘッド」的分離）
- サーキットブレーカーは導入しない（Organization横断スキャンは都度実行のバッチ処理であり、常時トラフィックに対する保護は不要）

A. はい、この設計でよい
B. 変更したい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この設計でよい

## Q2. スケーラビリティパターンについて

以下の設計でよいですか？
- 横スケール/縦スケールの概念は適用しない（CLIバイナリの単一プロセス内実行）
- アカウント×リージョンの組み合わせをイテレーションのキャッシュ単位とし、`ScanAggregator`内でVec<LogGroupRecord>として保持する
- キャッシュ層・CDN・シャーディングは不要（NFR3.3の規模想定に対し不要と判断）

A. はい、この設計でよい
X. Other (please specify)

[Answer]: A. はい、この設計でよい

## Q3. パフォーマンス最適化について

以下の設計でよいですか？
- 非同期処理は`tokio`のシングルランタイム上で、アカウント×リージョンのスキャンを逐次`await`する（NFR1.2の逐次呼び出し方針に従う）
- `describe-log-groups`は`aws-sdk-cloudwatchlogs`のページネーターAPI（`into_paginator()`）を使い、全ページ取得完了後に`ScanAggregator`へ渡す
- コネクションプーリングは`aws-config`が内部で使用するhyperクライアントのデフォルトに委ねる（独自実装しない）

A. はい、この設計でよい
X. Other (please specify)

[Answer]: A. はい、この設計でよい

## Q4. セキュリティアプローチ（多層防御）について

以下の設計でよいですか？
- 認証情報の保持: `AccountCredentials`の`secret_access_key`/`session_token`は`secrecy::SecretString`型でラップし、`Debug`実装で`[REDACTED]`とマスクする
- Identity検証: `IdentityVerifier`は`Result<(), IdentityMismatchError>`を返す純粋な検証関数とし、呼び出し元（LogGroupScanner起動前、ExecutionEngine実行直前）でエラー時は即座に`?`で伝播させて処理を打ち切る
- 入力値検証: CLI引数（アカウントID形式、リージョン名、retention日数の範囲）は`clap`のvalidatorで構文的に検証する
- シークレット管理: 環境変数・設定ファイルによる認証情報の直接指定はサポートしない（AWS標準の認証チェーン + AssumeRoleのみを経路とする）
- 監査ログ: `AuditLogger`はJSON Linesを1エントリずつ`fsync`付きで追記し、書き込み失敗（`std::io::Error`）は`ExecutionEngine`へ伝播させ操作を中断させる

A. はい、この設計でよい
B. 変更したい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この設計でよい

## Q5. 可観測性アプローチについて

以下の設計でよいですか？
- 構造化ログ: `tracing`クレートを用い、各アカウント×リージョンの処理を`tracing::Span`で囲み、進捗を標準エラー出力に人間可読形式で出す
- SLI/SLO: 定義しない（NFR5.3/5.4の通り対象外）
- 相関ID: 実行1回ごとに`run_id`（UUID）を発行し、AuditEntryの各行に付与して1回の実行内の全操作を後から相関できるようにする
- アラート/ダッシュボード: 対象外

A. はい、この設計でよい
X. Other (please specify)

[Answer]: A. はい、この設計でよい

## Q6. 論理コンポーネント境界（サービス分離・障害ドメイン）について

domain-designのcomponents.mdで定義した12コンポーネントを論理インフラコンポーネントとしてそのまま用い、以下の障害ドメイン分離でよいですか？
- 障害ドメイン1（アカウント単位）: CredentialProvider/IdentityVerifier/LogGroupScannerはアカウントごとに独立して失敗しうる
- 障害ドメイン2（実行単位）: ExecutionEngine/AuditLoggerは1回の`--execute`実行全体で1つの障害ドメインとし、AuditLogger書き込み失敗は実行全体を中断する
- 共有リソース: ローカル監査ログファイルのみ（外部共有リソースなし）

A. はい、この境界でよい
X. Other (please specify)

[Answer]: A. はい、この境界でよい

---

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
