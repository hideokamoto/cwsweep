# Changelog

このプロジェクトの注目すべき変更を記録する。形式は [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に準拠する。

## [0.2.0] - 2026-09-19

### Changed（破壊的変更）

- CLI をサブコマンド方式（`scan` / `clean` / `audit`）へ移行した。トップレベルの
  `--regions` / `--execute` / `--scan-only` / `--output` / `--audit-log-path` は廃止され、
  互換エイリアスは提供しない。旧形式の呼び出しは clap の usage エラーになる。
- 非TTY環境での `clean` は、スキャン結果を表示したうえで警告を出して正常終了する
  （旧 `--scan-only` 相当の自動フォールバックは行わない。非対話環境では `scan` を使う）。
- 監査ログファイルは `clean` 開始時にオープンされる。`scan` は監査ログを作成・更新しない。

### Added

- `audit` サブコマンド: 監査ログ（JSON Lines）を読み取り専用で全件表示する
  （`--output table|json`）。AWS へは接続しない。ファイル不在は空結果として正常終了し、
  不正行は標準エラーへ警告してスキップする。

### 移行ガイド

| v0.1 | v0.2 |
|---|---|
| `cwsweep --regions R` | `cwsweep clean --regions R` ／ `cwsweep scan --regions R` |
| `cwsweep --regions R --scan-only` | `cwsweep scan --regions R` |
| `cwsweep --regions R --output json` | `cwsweep scan --regions R --output json` |
| `cwsweep --regions R --execute` | `cwsweep clean --regions R --execute` |
| `cwsweep --regions R --audit-log-path P` | `cwsweep clean --regions R --audit-log-path P` |

## [0.1.0]

- 初回リリース: AWS Organization 横断の CloudWatch Logs 棚卸し・対話式削除／retention 変更、
  dry-run 既定、二重 Identity 検証、必須監査ログ。
