# Build Instructions — cwsweep

## 依存関係のインストール

Rustツールチェーン（stable、`rustfmt`・`clippy`コンポーネント込み）が必要。

```bash
rustup component add rustfmt clippy
```

依存クレートは`Cargo.lock`にピン留め済み。ネットワークアクセスがある環境では`cargo build`が自動的に`~/.cargo/registry`へフェッチする。

## 環境設定

- 環境変数・設定ファイルは不要（実AWS認証情報は標準の`aws-config`認証チェーンに委譲。本ビルド・テストはモックのみで完結し、実AWS呼び出しを行わない）。
- カバレッジ計測には`cargo-llvm-cov`（別途`cargo install cargo-llvm-cov`が必要、CI専用ツール）。
- 依存監査には`cargo-audit`・`cargo-deny`（別途インストールが必要。CI Pipelineステージで導入・実行する）。

## ビルドコマンド

```bash
cd /home/user/cwsweep
cargo build --locked
```

`--locked`は`Cargo.lock`と`Cargo.toml`の不一致時にビルドを失敗させる（team.md: 依存関係はCargo.lockをコミットし`--locked`でビルド・テストする）。

## ビルド検証

```bash
cargo build --locked 2>&1 | tail -5
```

`Finished`行が出力され、非ゼロ終了コードでないことを確認する。

## トラブルシューティング

| 症状 | 原因 | 対処 |
|---|---|---|
| `error: the lock file ... needs to be updated` | `Cargo.toml`変更後に`Cargo.lock`を再生成していない | `cargo update -p <crate>`のうえ`Cargo.lock`をコミット |
| `error[E0433]: failed to resolve` | AWS SDKクレートのバージョン不整合 | `Cargo.lock`をリセットし`cargo build`を再実行 |
| クレートのダウンロードに失敗 | ネットワーク未接続環境 | `cargo build --offline`（事前にvendorされたキャッシュが必要） |

## Assumptions & Open Questions

None.
