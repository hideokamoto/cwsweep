# dependencies.md — cwsweep

外部クレート依存の一覧・バージョン・用途は `technology-stack.md` を正とし、
ここでは重複を避けて **内部クロスモジュール依存関係**（"どのコンポーネントが
どのコンポーネントに依存するか"）のみを記録する。

## 内部モジュール間依存グラフ（`src/lib.rs` の `pub mod` 宣言順、`main.rs` からの配線含む）

- `main.rs`（バイナリ、合成ルート） → `cli`, `aggregator`, `audit`, `confirmation`,
  `credentials`, `error`, `execution`, `identity`, `org_discovery`, `planner`,
  `scanner`, `selector`（全コンポーネントを配線するエントリポイント）
- `cli`（`CliApp`） → `aggregator`, `confirmation`, `credentials`, `error`,
  `execution`, `identity`, `org_discovery`, `output`, `planner`, `scanner`,
  `selector`（オーケストレーション層。ほぼ全モジュールに依存する唯一のハブ）
- `scanner`（`LogGroupScanner`） → `aggregator`（`LogGroupRecord` 生成）,
  `credentials`, `error`, `identity`
- `execution`（`ExecutionEngine`） → `audit`, `credentials`, `error`, `identity`,
  `planner`
- `confirmation`（`ConfirmationPresenter`） → `planner`
- `output`（`OutputFormatter`） → `aggregator`
- `selector`（`InteractiveSelector`） → `aggregator`
- `credentials`（`CredentialProvider`） → `error`
- `identity`（`IdentityVerifier`） → `credentials`, `error`
- `org_discovery`（`OrgDiscovery`） → `error`
- `audit`（`AuditLogger`） → `error`, `planner`（`ActionKind` を監査エントリに埋め込む）
- `planner`（`ActionPlanner`） → `aggregator`（`LogGroupRecord` から `PlannedAction`
  を構築）, `error`
- `aggregator`, `error` — 他の内部モジュールに依存しない葉ノード

依存の可視化は `architecture.md`（Component Relationships の Mermaid 図）を参照。

## 外部サービス依存（呼び出し先 AWS サービス）

STS（`AssumeRole`, `GetCallerIdentity`）、Organizations（`ListAccounts`）、
CloudWatch Logs（`DescribeLogGroups`, `DeleteLogGroup`, `PutRetentionPolicy`）。
詳細な呼び出し契約は `api-documentation.md` の「3. 外部 AWS API」を参照。

## サプライチェーン管理

- `Cargo.lock` はコミット済み、CI は `--locked` でビルド・テスト。
- `cargo audit`（既知脆弱性スキャン、週次 CI ジョブ）と `cargo deny check`
  （`deny.toml` の advisories/licenses/bans/sources ポリシー）が CI 必須ゲート
  （通常実行は `bans licenses sources`、週次に `advisories` 追加）。
- ライセンス情報は `licenses/` ディレクトリに保持（サードパーティ通知は
  `THIRD_PARTY_NOTICES.md`）。
