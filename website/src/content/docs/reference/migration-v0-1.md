---
title: v0.1 からの移行
description: v0.2.0 でのフラグ方式からサブコマンド方式への移行対応表。
sidebar:
  order: 3
---

v0.2.0 でトップレベルのフラグ方式を廃止し、サブコマンド方式へ移行しました。旧形式は互換エイリアスなしに clap の usage エラーとなります。

| v0.1 | v0.2 |
|---|---|
| `cwsweep --regions R` | `cwsweep clean --regions R`（従来どおり対話へ進む）／ `cwsweep scan --regions R`（表示のみ） |
| `cwsweep --regions R --scan-only` | `cwsweep scan --regions R` |
| `cwsweep --regions R --output json` | `cwsweep scan --regions R --output json` |
| `cwsweep --regions R --execute` | `cwsweep clean --regions R --execute` |
| `cwsweep --regions R --audit-log-path P` | `cwsweep clean --regions R --audit-log-path P` |
| （なし） | `cwsweep audit [--audit-log-path P] [--output json]` |
