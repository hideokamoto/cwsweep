---
title: clean — Select, delete, and change retention
description: Delete or change the retention of interactively selected log groups.
sidebar:
  order: 2
---

`clean` proceeds through scan → interactive selection → confirmation. **Dry-run is the default**: nothing is deleted and no retention is changed unless you explicitly pass `--execute`.

```bash
# Dry-run (default). Goes through scan → interactive selection → confirmation, but never deletes or changes retention
cwsweep clean --regions us-east-1

# Actually delete / change retention (this never happens unless you pass this flag explicitly)
cwsweep clean --regions us-east-1 --execute

# Override the audit log destination (default: cwsweep-audit.jsonl in the current directory.
# There is no option to disable it)
cwsweep clean --regions us-east-1 --audit-log-path /var/log/cwsweep/audit.jsonl
```

- The interactive multi-select always starts with every item unchecked. After selecting, you choose the action: `delete`, or `set-retention` (you then enter the number of days).
- The confirmation defaults to No. Before execution, a confirmation screen restates the target account IDs, regions, log group names, and total bytes.
- When stdin is not a TTY (pipes, CI, etc.), `clean` prints the scan results, warns that it cannot proceed to interactive selection, and exits successfully. Use [`scan`](/guides/scan/) for inventories in non-interactive environments.

- With `--execute`, the account ID is verified again with `sts:GetCallerIdentity` immediately before each operation, and the audit log gets an `intent` line before the API call and a `result` line after it. A dry-run writes nothing to the audit log.

See [Safety](/reference/safety/) for the full set of safety mechanisms.
