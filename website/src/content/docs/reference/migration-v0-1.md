---
title: Migrating from v0.1
description: Mapping from the v0.1 flag-based interface to the v0.2.0 subcommand-based interface.
sidebar:
  order: 3
---

v0.2.0 removed the top-level flag-based interface in favor of subcommands. The old forms have no compatibility aliases and result in a clap usage error.

| v0.1 | v0.2 |
|---|---|
| `cwsweep --regions R` | `cwsweep clean --regions R` (proceeds to interactive selection as before) / `cwsweep scan --regions R` (display only) |
| `cwsweep --regions R --scan-only` | `cwsweep scan --regions R` |
| `cwsweep --regions R --output json` | `cwsweep scan --regions R --output json` |
| `cwsweep --regions R --execute` | `cwsweep clean --regions R --execute` |
| `cwsweep --regions R --audit-log-path P` | `cwsweep clean --regions R --audit-log-path P` |
| (none) | `cwsweep audit [--audit-log-path P] [--output json]` |
