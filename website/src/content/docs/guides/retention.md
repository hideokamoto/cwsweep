---
title: retention set — Bulk retention
description: Set the same retention period on every log group found by the scan (never deletes).
sidebar:
  order: 3
---

```bash
# Dry-run (default). Goes through scan → confirmation, but put-retention-policy is never called
cwsweep retention set --regions us-east-1 --days 30

# Actually set retention (this never happens unless you pass this flag explicitly)
cwsweep retention set --regions us-east-1 --days 30 --execute

# Override the audit log destination (as with clean, there is no option to disable it)
cwsweep retention set --regions all --days 90 --execute --audit-log-path /var/log/cwsweep/audit.jsonl
```

- `--days` is required. Any value other than the discrete values accepted by CloudWatch Logs (1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, 3653) is rejected before connecting to AWS.
- Every log group found by the scan is targeted (there is no interactive selection). On a TTY, the target list is shown and you are asked to confirm before execution. On a non-TTY (CI, etc.) only dry-run is allowed; passing `--execute` exits with an error.
- There is no code path that reaches `delete-log-group`. It reuses the same safety mechanisms as `clean`: dry-run by default, mandatory audit logging, and double identity verification.
