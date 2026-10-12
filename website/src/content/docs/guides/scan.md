---
title: scan — Inventory
description: Inventory log groups in read-only mode.
sidebar:
  order: 1
---

Prints log groups as a table and exits. It does not proceed to interactive selection or confirmation, and it does not write the audit log.

```bash
cwsweep scan --regions us-east-1,us-west-2

# Switch the output format to JSON (for CLI agents such as Cursor or Claude Code)
cwsweep scan --regions us-east-1 --output json

# Override the role name used when assuming into member accounts (default: OrganizationAccountAccessRole)
cwsweep scan --regions us-east-1 --role-name CustomOrgRole

# Target every region in the commercial partition (case-insensitive).
# The number of target regions is printed to stderr, and on a TTY you are asked to confirm first
cwsweep scan --regions all

# If --regions is omitted, on a TTY you pick regions from a multi-select built from account:ListRegions.
# On a non-TTY (CI/pipes) it exits with an error without connecting to AWS
cwsweep scan
```

## Specifying regions

| Input | Behavior |
|---|---|
| `--regions us-east-1,ap-northeast-1` | Targets only the listed regions |
| `--regions all` (`ALL` / `All` also accepted) | Calls `account:ListRegions` with the management account's credentials and targets every enabled region in the commercial partition (`aws`). GovCloud, China regions, and opt-in regions that are not enabled are excluded. If `all` is combined with other values, `all` wins. The count and list are printed to stderr, and on a TTY you are asked to confirm (default: No) |
| Omitted, TTY | Pick regions from the same list via an interactive multi-select. Selecting zero regions is an error |
| Omitted, non-TTY | Exits with an error asking you to specify `--regions` (does not connect to AWS) |

Region selection works the same way for `clean` and `retention set`.
