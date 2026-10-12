---
title: Logging and troubleshooting
description: How errors and verbose logs are displayed, and how to fix common errors.
sidebar:
  order: 5
---

## Logging and error output

- On a fatal error, cwsweep prints a single line `Error: <message>` to stderr, in red (only when attached to a TTY; no color when piped or redirected). Internal AWS SDK error structures are not dumped.
- With `--verbose` (`--debug` is a synonym), detailed logs from the AWS SDK and internal crates (such as how the credential provider was resolved) are written to stderr as is. Without it, these raw logs are suppressed and only cwsweep's own warnings are shown. `--verbose` / `--debug` work either before or after the subcommand (e.g. `cwsweep scan --verbose` / `cwsweep --debug clean`).
- If the `RUST_LOG` environment variable is set, it takes precedence regardless of `--verbose`.

```bash
# To see the details of an authentication error
cwsweep scan --regions us-west-2 --verbose
```

## Common errors

### `WARN profile [xxx] ignored; sections ... must have a prefix i.e. [profile my-profile]`

A section in `~/.aws/config` is named `[xxx]`. The AWS SDK requires the `profile` prefix for every profile other than `default`, so rename it to `[profile xxx]` (sections in `~/.aws/credentials` can stay as `[xxx]`).

### `ProfileFile provider could not be built: ... the credentials-login feature must be enabled`

Profiles created by `aws login` with a `login_session_arn` require the `credentials-login` feature of `aws-config`. The current source has it enabled, so if you see this error you are running an old binary. Reinstall the latest version.
