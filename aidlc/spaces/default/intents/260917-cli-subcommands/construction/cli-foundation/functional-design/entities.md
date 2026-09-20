# entities.md — Unit: cli-foundation

`cli-foundation` Unit（`Cli` / `Commands` / `CliApp` ハンドラ群）が所有する
エンティティ。技術非依存の論理モデルであり、`entities.md` はデータ形状の
source of truth、ワークフローは `functional-spec.md`、決定ロジックは
`rules.md` を参照する。

```yaml
entities:
  - name: Cli
    description: >
      コマンドライン全体の解析結果。必ずちょうど1つのサブコマンド（Commands）を
      保持する。グローバルオプションは持たない（全オプションはサブコマンドに所属）。
    attributes:
      - name: command
        type: Commands
        required: true
        constraints: "サブコマンド未指定はパースエラー（BR1.1）"
    relationships:
      - target: Commands
        cardinality: "1..1"
        direction: "Cli -> Commands"

  - name: Commands
    description: >
      3つのサブコマンドのいずれか。バリアントごとに受け付けるオプション集合が
      異なり、所属しないオプションの指定はパースエラーとなる。
    attributes:
      - name: variant
        type: enum
        required: true
        allowed_values: [Scan, Clean, Audit]
    variants:
      - name: Scan
        attributes:
          - { name: regions,   type: "list<string>", required: true,  constraints: "1件以上、重複は初出順で除去" }
          - { name: role_name, type: string,          required: false, default: "OrganizationAccountAccessRole" }
          - { name: output,    type: OutputFormat,    required: false, default: table }
      - name: Clean
        attributes:
          - { name: regions,        type: "list<string>", required: true,  constraints: "1件以上、重複は初出順で除去" }
          - { name: role_name,      type: string,          required: false, default: "OrganizationAccountAccessRole" }
          - { name: execute,        type: boolean,         required: false, default: false }
          - { name: audit_log_path, type: path,            required: false, default: "cwsweep-audit.jsonl" }
      - name: Audit
        attributes:
          - { name: audit_log_path, type: path,         required: false, default: "cwsweep-audit.jsonl" }
          - { name: output,         type: OutputFormat, required: false, default: table }

  - name: OutputFormat
    description: 標準出力の整形方式（既存エンティティ、変更なし）。
    attributes:
      - name: value
        type: enum
        required: true
        allowed_values: [table, json]
        default: table

  - name: ScanReport
    description: >
      run_scan / run_clean 共通のスキャン結果。ScanAggregator（集約済みレコード）と
      AccountRegionOutcome の一覧（成功/失敗のバルクヘッド分離）から成る。
      既存エンティティであり形状は変更しない。
    attributes:
      - { name: aggregator, type: ScanAggregator,             required: true }
      - { name: outcomes,   type: "list<AccountRegionOutcome>", required: true }
    relationships:
      - target: AccountRegionOutcome
        cardinality: "0..N"
        direction: "ScanReport -> AccountRegionOutcome"

  - name: ExitDisposition
    description: >
      ハンドラが main へ返す終了方針。プロセス終了コードへ写像される。
    attributes:
      - name: kind
        type: enum
        required: true
        allowed_values: [Success, ScanFullyFailed, Error]
      - name: message
        type: string
        required: false
        constraints: "kind != Success のとき必須"

relationships_summary:
  - "Cli 1..1 Commands"
  - "Commands::Scan / Commands::Clean → ScanReport を生成する（run_scan / run_clean）"
  - "Commands::Audit → AuditReadOutcome（audit-reader Unit所有、Contract 1）を消費する"
  - "各ハンドラ → ExitDisposition を返す"
```

## サマリー

- `Cli` は必ず1つの `Commands` を持ち、グローバルオプションは存在しない。
  旧トップレベルフラグ（`--regions` / `--execute` / `--scan-only` /
  `--output` / `--audit-log-path`）はすべてサブコマンド配下へ移動または廃止
  される（`--scan-only` は `scan` サブコマンドそのものに置換され消滅）。
- `Scan` は `--output` を持ち `--execute` / `--audit-log-path` を持たない。
  `Clean` は `--execute` / `--audit-log-path` を持ち `--output` を持たない。
  `Audit` は `--audit-log-path` / `--output` のみを持ち、`--regions` /
  `--role-name` を持たない（AWSへ一切アクセスしない）。
- `ScanReport` / `OutputFormat` / `AccountRegionOutcome` は既存の形状を
  そのまま再利用し、本Unitで形状変更は行わない。
- `AuditReadEntry` / `AuditReadOutcome` / `SkippedLine` は audit-reader Unit
  の所有エンティティ（Contract 1）であり、本Unitは消費のみを行う。
