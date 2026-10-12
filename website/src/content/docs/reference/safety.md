---
title: Safety
description: cwsweep's safety mechanisms against accidental deletion and misoperation, and its exit code policy.
sidebar:
  order: 1
---

- Omitting `--regions` never silently scans every region. On a TTY you choose regions interactively; on a non-TTY (CI/pipes) it exits with an error. To target every region you must pass `--regions all` explicitly, and even then the number of targets is shown and, on a TTY, you are asked to confirm (to prevent accidentally scanning unrelated regions).
- Unless you pass `clean --execute` / `retention set --execute`, `delete-log-group` / `put-retention-policy` are never called (dry-run by default). `scan` and `audit` have no code path that reaches these APIs, and `retention set` has no code path that reaches `delete-log-group`.
- The interactive multi-select always starts with every item unchecked.
- Every operation that deletes or changes retention is always recorded in the audit log as JSON Lines — target account ID, region, log group name, execution time, and success/failure (default: `cwsweep-audit.jsonl` in the current directory; override with `--audit-log-path`). There is no option to disable it. If writing to the audit log fails, the operation is aborted. `clean` / `retention set` validate the destination at startup (the parent directory must exist and the path must not be a non-regular file), and the file itself is created on the first write. For each operation, two lines are appended: an `intent` entry before the API call and a `result` entry after it. A dry-run (no `--execute`) appends nothing, and `scan` never creates the file.
- cwsweep does not assume a role into the management account itself and uses your current credentials as is; it assumes a role only into member accounts.
- Immediately before any API call to an AWS account (at scan time, and a second time right before deletion or retention changes), the account ID is verified with `sts:get-caller-identity`. On a mismatch, processing for that account fails immediately.

## Exit code policy (for CI/automation)

For `scan` / `clean`, if there is at least one target account × region combination and the scan fails for all of them, `cwsweep` exits with a non-zero exit code (in addition to warning logs on stderr, it does not print the normal "0 results" message in this case).

If at least one combination succeeds, the exit code is 0 (success) even if others fail; failed accounts/regions are only reported individually in warning logs. This lets callers distinguish "every scan failed" from "there really were zero target log groups" by the exit code alone.
