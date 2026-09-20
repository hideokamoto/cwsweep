# tech-stack-decisions.md — Unit: release-docs

| 領域 | 選択 | 根拠 |
|---|---|---|
| ドキュメント形式 | Markdown（既存 README.md、新規 CHANGELOG.md） | 既存慣行。CHANGELOG は Keep a Changelog 形式（`## [0.2.0]` / `### Changed` / `### Added`）で記述 |
| バージョン管理 | `Cargo.toml` `version = "0.2.0"` + `cargo update -p cwsweep --offline` 相当で `Cargo.lock` 同期 | FR6.3。`--locked` ビルドを維持 |
| 検証 | `cargo build --locked`、README の旧フラグ grep | 文書と実装の乖離検出 |

## 明示的に採用しないもの

- 旧フラグの hidden alias（NFR4）。
- ドキュメント生成ツールの追加。
