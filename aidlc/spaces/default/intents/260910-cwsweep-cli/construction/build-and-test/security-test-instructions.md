# Security Test Instructions — cwsweep

Standard戦略ではセキュリティテストファイルの生成は必須ではないが、本プロジェクトは破壊的操作（ログ削除・retention変更）を扱い、project.md/team.mdに詳細なセキュリティ関連Forbidden/Mandated条項（クレデンシャル非ログ出力、unsafe禁止、Identity二重検証等）が存在するため、ソフトガイドラインの例外としてセキュリティテスト観点を独立文書化する。

## 実行コマンド（静的検査）

リポジトリルート（`Cargo.toml`が存在するディレクトリ）で実行する。

```bash
# unsafeコード禁止の確認（コンパイル時に#![forbid(unsafe_code)]で強制されるため、grep は補助確認）
grep -n "forbid(unsafe_code)" src/main.rs src/lib.rs

# 本番コードパスでのunwrap/expect/panic禁止の確認（#[deny(...)]がコンパイル時に強制）
grep -n "deny(clippy::unwrap_used" src/main.rs src/lib.rs

# clippyによる静的解析（unwrap_used/expect_used/panic includedのdeny設定込み）
cargo clippy --all-targets -- -D warnings
```

## 実行コマンド（クレデンシャルマスキング・動的検証）

`src/credentials.rs`の既存ユニットテストが以下を検証済み（`cargo test --lib credentials::`）:
- `AccountCredentials`の`Debug`実装が`secret_access_key`/`session_token`の実値を含まず`[REDACTED]`のみを出力すること

```bash
cargo test --lib credentials::
```

## 実行コマンド（Identity検証・dry-run既定の安全パス）

```bash
cargo test --lib identity::
cargo test --lib execution::
```

`identity::tests`はアカウントID不一致時の即時失敗と境界値（不一致ID、リージョン跨ぎ）を、`execution::tests`は`--execute`未指定時のdry-run既定と削除直前の独立した二重目Identity検証を検証する。

## カバレッジ目標

上記の安全パスはteam.mdの要求により80%floorとは別枠で100%パスカバレッジ＋境界値テストが要求されている。本stage実行時点の実測（`cargo llvm-cov --lib --summary-only`）は`execution.rs` 93.66%・`identity.rs` 99.22%であり、未達分はCode Generationステージの承認ゲートで人間により明示的にAccepted riskとして記録済み（R-01、`.aidlc-reviews/code-generation/stage/df820a4cc21bd395/1.review.md`）。本stageではこのギャップを新規の欠陥として扱わず、既知の受容済みリスクとして`test-results.md`のTarget Verification Matrixに転記する。

## CI Pipelineステージが所有する項目（本stageでは実行しない）

- NFR2.6（`cargo audit`による依存脆弱性スキャン）
- NFR2.7（`cargo deny check`によるライセンス・サプライチェーンチェック）

いずれも`cargo-audit`/`cargo-deny`のインストールを要し、CI環境でのゲート化が本来の所在（project.md Mandated）であるため、CI Pipelineステージの成果物として明示的に委譲する。

## Assumptions & Open Questions

None.
