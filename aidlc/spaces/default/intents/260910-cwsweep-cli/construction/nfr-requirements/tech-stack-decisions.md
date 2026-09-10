# Tech Stack Decisions

| 領域 | 選定 | 根拠 |
|---|---|---|
| 言語 | Rust (2021 edition以降) | PRDで確定。単一バイナリ配布、メモリ安全性、AWS SDK for Rustの提供 |
| AWS SDK | `aws-config`, `aws-sdk-organizations`, `aws-sdk-sts`, `aws-sdk-cloudwatchlogs` | 各AWSサービスへの公式SDKアクセス。`aws-config`で明示的CredentialsProviderを構築する（環境変数チェーンへの暗黙依存を避けるため、CredentialProviderコンポーネントの設計と整合）。リトライ設定は`RetryMode::Standard`・最大3回（NFR1.4） |
| 非同期ランタイム | `tokio` | AWS SDK for RustはTokioベースの非同期APIを前提とするため |
| CLI引数 | `clap`（derive API） | Rustエコシステムで標準的、サブコマンド・フラグの型安全な定義が可能 |
| 対話式選択UI | `inquire` | マルチセレクト（初期全OFF）・確認プロンプトをサポート |
| シリアライズ | `serde` / `serde_json` | json出力・監査ログ（JSON Lines）のシリアライズ |
| テーブル表示 | `comfy-table` | table出力の整形 |
| エラー型 | `thiserror`（ライブラリ的エラー定義） | `unwrap`/`expect`/`panic`禁止方針（Practices Discovery確定）と整合させ、Result型でのエラー伝播を徹底する。`anyhow`は使用しない（型付きエラーの明示性を優先） |
| クレデンシャルマスキング | `secrecy` | 一時クレデンシャルの非ログ出力保証（NFR2.1） |
| カバレッジ計測 | `cargo llvm-cov` | Practices Discoveryで確定 |
| Lint | `clippy`（`-D warnings`段階的厳格化） | Practices Discoveryで確定 |
| 依存監査 | `cargo audit`, `cargo deny` | Practices Discoveryで確定（NFR2.6, NFR2.7） |

## Assumptions & Open Questions

None.
