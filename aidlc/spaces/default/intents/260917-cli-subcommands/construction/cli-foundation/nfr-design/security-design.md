# security-design.md — Unit: cli-foundation

## D-SEC-1: サブコマンド別のハンドラ・シグネチャによる依存限定（NFR2.3 / NFR2.4）

```text
run_scan(deps: ScanDeps, regions, output)       -> ExitDisposition
run_clean(deps: CleanDeps, regions, execute, stdin_is_tty, prompts) -> ExitDisposition
run_audit(reader: &dyn AuditRead, output)       -> ExitDisposition
```

- `ScanDeps` = CredentialProvider + IdentityCheck + DescribeLogGroupsOperations
  （`AuditWrite` / `ActionApiOperations` を含まない）。
- `CleanDeps` = `ScanDeps` + ActionApiOperations + AuditWrite（既存 `CliApp` 相当）。
- `run_audit` は `AuditRead` トレイトオブジェクト 1 つと `OutputFormat` のみ。
  AWS クライアント・資格情報型は引数にも戻り値にも現れない。
- 上記により「`audit` から破壊的操作に到達できない」「`scan` から監査ログ書き込みに
  到達できない」は **関数シグネチャ＝コンパイル時制約** となる。

## D-SEC-2: clap のバリアント別 `#[arg]` によるオプション所属の宣言（NFR2.4 / NFR5.6）

- `Commands::Scan` に `--audit-log-path` / `--execute` を定義しない、
  `Commands::Clean` に `--output` を定義しない、`Commands::Audit` に `--regions` /
  `--role-name` を定義しない。所属外オプションは clap が未知引数として拒否する。
- 監査ログ無効化フラグは定義しない（存在しない＝無効化不可）。
- 旧フラグ用の `alias` / `hide` 引数は定義しない（NFR4）。

## D-SEC-3: main における配線順序（NFR5.6 / NFR5.4）

`Clean` 分岐: 監査ログ open → AWS 配線 → `run_clean`。open 失敗はスキャンにも進まず
即時エラー。`Audit` 分岐: AWS 配線を一切行わず `AuditReader::new(path)` のみ。

## D-SEC-4: 既存安全機構の再利用（NFR5.4）

`run_clean` の実行フェーズは既存 `CliApp::execute` → `ExecutionEngine::execute_plan`
をそのまま呼ぶ。二重 Identity 検証・dry-run 判定・intent/result 監査記録は
`ExecutionEngine` 内に閉じており、本Unitは `execute_flag` を透過するだけで
ロジックを複製しない。

## D-SEC-5: クレデンシャル・panic 方針（NFR5.5）

新設コードは `AccountCredentials` を既存型のまま受け渡し、ログ・出力に含めない。
`unwrap` / `expect` は `#[cfg(test)]` 内に限定。エラーは `ExitDisposition::Error`
へ集約して main が `Box<dyn Error>` として返す。
