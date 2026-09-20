# Build Instructions — 260917-cli-subcommands

## 依存関係のインストール

- Rust stable（`rustup`）、コンポーネント `rustfmt` / `clippy` / `llvm-tools-preview`
- `cargo install cargo-llvm-cov cargo-audit cargo-deny --locked`（カバレッジ・サプライチェーン検査）
- 追加のクレート依存は本intentでは無し（`Cargo.toml`のバージョン更新のみ）

## 環境設定

- 環境変数・設定ファイルは不要。ビルド／テストは実AWSに接触しない（全テストはトレイト差し替えのモック）。
- `Cargo.lock` はコミット済み。必ず `--locked` を付与する。

## ビルドコマンド

```bash
cargo build --locked
```

## ビルド検証

```bash
cargo run -q --locked -- --version        # cwsweep 0.2.0
cargo run -q --locked -- --help           # scan / clean / audit の3サブコマンドが列挙される
cargo run -q --locked -- --regions x ; echo $?   # 旧フラグ形式は usage エラー（非ゼロ）
```

## トラブルシューティング

- `--locked` で失敗する場合: `Cargo.toml` と `Cargo.lock` の不整合。`cargo update -p cwsweep --offline` で本パッケージのみ同期する（依存のバージョンは変えない）。
- `clippy -D warnings` の `unwrap_used` / `expect_used` 違反: 本番コードでは `?` と `thiserror` の型付きエラーへ置き換える。
