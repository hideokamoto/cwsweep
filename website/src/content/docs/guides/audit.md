---
title: audit — View the audit log
description: Display the deletion and retention-change audit log in read-only mode.
sidebar:
  order: 4
---

```bash
# Show every entry of the audit log at the default path (cwsweep-audit.jsonl) as a table
cwsweep audit

# Specify the path and output format
cwsweep audit --audit-log-path /var/log/cwsweep/audit.jsonl --output json
```

`audit` does not connect to AWS and never modifies the audit log. If the file does not exist, it exits successfully with an empty result. Malformed lines are skipped with a warning on stderr (valid entries before and after them are still shown).

Each deletion or retention change produces two entries: an `intent` entry written before the API call and a `result` entry written after it. The `success` value of an `intent` entry is a placeholder (the outcome is not yet known) and is shown as `-` in the table; use the `result` entry to see whether the operation succeeded.
