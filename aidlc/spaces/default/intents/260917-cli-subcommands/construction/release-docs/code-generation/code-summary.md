# Code Summary — Unit: release-docs

## 実装したもの

- `README.md`: 「使い方」を `scan` / `clean` / `audit` 小節へ分割。各小節の先頭例は安全な形。
  「安全機構」節に既存の箇条書きをサブコマンド名付きで維持し、`scan`/`audit` が破壊的 API に
  到達しないこと、監査ログが `clean` 開始時にオープンされることを追記。「v0.1 からの移行」節に対応表。
- `CHANGELOG.md`: 新規。`[0.2.0]` に Changed（破壊的変更）/ Added / 移行ガイド、`[0.1.0]` 初回。
- `Cargo.toml`: `version = "0.2.0"`。`Cargo.lock` 同期。

## デビエーション

なし。

## テストの実行方法

```bash
cargo build --locked && cargo run -q -- --version
```

結果: `cwsweep 0.2.0`、ビルド成功。README の `--scan-only` 出現は移行表の 1 行のみ。

## 変更しなかったもの

`src/**`、CI 設定、依存関係。
