# security-design.md — Unit: release-docs

## D-SEC-1: README 構成（NFR4.1 / NFR5.1 / NFR5.2）

「使い方」節を `scan` / `clean` / `audit` の 3 小節に分割し、各小節の先頭コマンド例は
最も安全な形（`scan`、`clean` は `--execute` なし）とする。`--execute` 付きの例は
「このフラグを明示的に渡さない限り絶対に実行されない」の注記と同じ行ブロックに置き、
単独でコピーされにくくする。`audit` 小節では「読み取り専用・AWS 非接続・ファイル不在は
空結果」を明記する。既存の箇条書き（`--regions` 必須、dry-run 既定、監査ログ必須、
管理アカウント非 AssumeRole、二重 Identity 検証、終了コード方針）はサブコマンド名を
補ったうえで全件維持する。

## D-SEC-2: 移行ガイド（NFR4.1）

README「v0.1 からの移行」節と CHANGELOG `## [0.2.0]` に同一の対応表を置く：

| v0.1 | v0.2 |
|---|---|
| `cwsweep --regions R` | `cwsweep clean --regions R`（対話継続）／ `cwsweep scan --regions R`（表示のみ） |
| `cwsweep --regions R --scan-only` | `cwsweep scan --regions R` |
| `cwsweep --regions R --output json` | `cwsweep scan --regions R --output json` |
| `cwsweep --regions R --execute` | `cwsweep clean --regions R --execute` |
| `cwsweep --regions R --audit-log-path P` | `cwsweep clean --regions R --audit-log-path P` |
| （なし） | `cwsweep audit [--audit-log-path P] [--output json]` |

旧形式はエイリアスなしで clap の usage エラーになる旨を併記する。非 TTY で `clean` を
実行した場合はスキャン結果表示後に警告して正常終了し、`scan` の利用を促す点も記載する。

## D-SEC-3: バージョン（NFR5.3）

`Cargo.toml` の `version = "0.2.0"`。`Cargo.lock` の `cwsweep` エントリを同期し、
`cargo build --locked` を検証手段とする。依存追加なし。
