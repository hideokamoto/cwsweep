# security-requirements.md — Unit: cli-foundation

Inception の NFR2（構造的安全性）・NFR5（既存NFR継続）を本Unitの詳細要件に具体化する。

## NFR2: 構造的安全性（run_audit の依存限定）

### NFR2.3 run_audit の依存注入限定（呼び出し側）

- **要件**: `run_audit` の引数型は `AuditRead` 実装と `OutputFormat` のみとし、
  `ExecutionEngine` / `AuditWrite` / `ActionApiOperations` / `CredentialProvider` /
  `IdentityCheck` のいずれも受け取らない。`Commands::Audit` バリアントは
  `--regions` / `--role-name` を持たず、main の `Audit` 分岐は AWS SDK クライアントを
  実体化しない。
- **脅威モデル（STRIDE: 権限昇格）**: 読み取り専用であるべき `audit` から
  破壊的操作・監査ログ改ざんへ到達する経路の混入。
- **検証**: audit-reader Unit が実装した構造的回帰テスト（`src/audit.rs` の
  AuditRead 領域が execution/write に非依存）に加え、`run_audit` の
  シグネチャを固定する lib 単体テスト（`AuditRead` のテストダブルのみで
  呼び出せること）を追加する。
- **出典**: NFR2、FR4.5、`rules.md` BR4.2、Contract 1。

### NFR2.4 scan の書き込み非到達

- **要件**: `run_scan` は `AuditWrite` / `ActionApiOperations` を要求しない。
  `Commands::Scan` は `--audit-log-path` を受理しない。
- **検証**: `scan --audit-log-path x` がパースエラーになる lib テスト、
  `run_scan` が `AuditWrite` なしで呼び出せるシグネチャテスト。
- **出典**: FR2.2、`rules.md` BR2.2。

## NFR5: 既存セキュリティ機構の継続

### NFR5.4 dry-run 既定と二重 Identity 検証の維持

- **要件**: `clean --execute` 未指定時は `delete-log-group` /
  `put-retention-policy` を呼ばない。実行直前の二重目 Identity 検証、
  メンバーアカウント API 呼び出し前の Identity 検証、管理アカウントへの
  AssumeRole 非実施は既存の `ExecutionEngine` / `LogGroupScanner` /
  `CredentialProvider` をそのまま利用することで維持する。
- **検証**: 既存の `execution.rs` / `scanner.rs` / `credentials.rs` テストを
  変更なく green に保つ。`clean` の既定 `execute=false` を lib テストで固定化。
- **出典**: FR3.3、FR5.1〜FR5.4、`rules.md` BR3.2 / BR5.1。

### NFR5.5 クレデンシャル非露出・unsafe 禁止・panic 排除

- **要件**: 新設コードは `SecretString` 経由以外でクレデンシャルを扱わず、
  `unsafe` を用いず、本番コードパスに `unwrap` / `expect` / `panic!` を置かない。
  `cargo audit` / `cargo deny check` の CI ゲートは変更しない。
- **検証**: `cargo clippy -D warnings`、コードレビュー、既存 CI ジョブ。
- **出典**: NFR5、project.md Forbidden。

### NFR5.6 監査ログ必須（clean）

- **要件**: `clean` は `--audit-log-path`（既定 `cwsweep-audit.jsonl`）を必ず
  オープンし、オープン失敗は clean 全体の失敗とする。無効化オプションを
  設けない。
- **検証**: `Commands::Clean` の既定値テスト、無効化フラグが存在しないこと。
- **出典**: FR3.5、`rules.md` BR3.3。
