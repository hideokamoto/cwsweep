# CI Configuration — 260917-cli-subcommands

## CIツール

CircleCI。設定ファイルは既存の `.circleci/config.yml`。**本intentでは CI 設定ファイルを変更しない。**

## 変更不要である根拠

| 本intentの変更 | CI への影響 |
|---|---|
| `src/cli.rs` / `src/main.rs` / `src/output.rs` のサブコマンド化 | `fmt` / `clippy` / `test` / `coverage` が既存のままクレート全体を対象にするため追加ジョブ不要。ディスパッチは lib 側にあり `cargo llvm-cov --lib` の計測対象（team.md Q3） |
| `src/audit.rs` の `AuditReader` | 同上 |
| `README.md` / `CHANGELOG.md` | CI 対象外 |
| `Cargo.toml` 0.2.0 / `Cargo.lock` 同期 | `--locked` を全ジョブで使用しているため、lock 同期済みであることが CI で機械的に検証される |
| 依存追加 | なし。`audit` / `deny` の対象・結果に変化なし |

## トリガー・ジョブ構成

intent 260910-cwsweep-cli の `ci-config.md` のとおり（`build-and-test`: fmt / clippy / test / coverage / audit / deny、`release`: タグ駆動4ターゲット）。

## ローカルでの同等実行（Build and Test で実施済み）

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo build --locked && cargo test --locked
cargo llvm-cov --lib --locked --fail-under-lines 80 --summary-only   # 96.63%
cargo audit && cargo deny check
```

## Assumptions & Open Questions

None.
