# contract-summary.md — Unit間契約（260917-cli-subcommands）

本intentには外部（パートナー・公開インターネット等）に消費されるAPIは存在しない
（cwsweepはスタンドアロンCLIバイナリであり、CLIの引数・終了コード契約は
requirements.md FR2.1-FR2.4/FR3.1-FR3.5/FR6.1で既に規定済み）。以下はすべて
同一Cargoパッケージ内のUnit間境界（型シグネチャレベルの契約）である。

## Contracts

| # | Provider Unit | Consumer | Mechanism | Owner |
|---|---|---|---|---|
| 1 | audit-reader (U1) | cli-foundation (U2) | shared-schema（クレート内trait/型シグネチャ） | audit-reader (U1) |
| 2 | audit-reader (U1) + cli-foundation (U2, OutputFormatter) | release-docs (U3) | shared-schema（出力フォーマット仕様） | audit-reader (U1) / cli-foundation (U2) |
| 3 | cli-foundation (U2) | release-docs (U3) | shared-schema（CLI引数・終了コード仕様） | cli-foundation (U2) |

## Contract 1: AuditRead trait / AuditEntry型（audit-reader → cli-foundation）

`cli-foundation`の`run_audit`ハンドラが`audit-reader`の読み取り専用APIを呼び出すための契約。
`project.md`のForbidden（`run_audit`ハンドラは削除・retention変更・監査ログ書き込み系の型に
コンパイル時点で到達不能な構造とする）を型レベルで担保する境界でもある。

```yaml
shared-schema:
  name: AuditReadContract
  owner: audit-reader (U1)
  consumer: cli-foundation (U2)
  types:
    - name: AuditRead
      kind: trait
      methods:
        - signature: "fn entries(&self) -> Result<Vec<AuditEntry>, AuditReadError>"
          description: >
            監査ログファイルを行単位でストリーミング読み取りし、パース済みの
            AuditEntry一覧を返す。不正フォーマット行はスキップし、呼び出し元が
            警告として扱えるよう並行してSkippedLine一覧も返す（下記
            AuditReadOutcome参照）。
      forbidden_dependencies:
        - ExecutionEngine
        - AuditWrite
      note: >
        AuditReadの実装（および実装が依存する型グラフ）はExecutionEngine・
        AuditWrite等の削除・retention変更・監査ログ書き込み系の型に一切
        依存しない。この非依存性は本契約の中核であり、実装時に構造的
        回帰テスト（コンパイル成立を前提とした型/モジュール境界テスト）で
        検証する（NFR2）。
    - name: AuditEntry
      kind: struct
      identifier: "(timestamp, account_id, region, log_group_name)の組（一意性は記録順序で担保、厳密なユニーク制約はない）"
      fields:
        - "timestamp: String (ISO 8601)"
        - "account_id: String"
        - "region: String"
        - "log_group_name: String"
        - "action: String"
        - "result: String"
    - name: AuditReadOutcome
      kind: struct
      fields:
        - "entries: Vec<AuditEntry>"
        - "skipped_lines: Vec<SkippedLine>"
      description: >
        不正フォーマット行のスキップ+警告出力（FR4.4）と、対象ファイル不在時の
        空扱い正常終了（FR4.6）を型で表現する。skipped_linesが空でも処理全体は
        中断しない。
    - name: SkippedLine
      kind: struct
      fields:
        - "line_number: usize"
        - "reason: String"
    - name: AuditReadError
      kind: enum
      variants:
        - "Io(std::io::Error)"
      description: >
        ファイルI/O自体の失敗（権限エラー等）のみをエラーとして扱う。個々の行の
        パース失敗はSkippedLineとして扱い、AuditReadErrorには昇格させない
        （FR4.4の「処理全体を中断しない」契約と整合）。
  error_handling: >
    Result<T, E>で呼び出し元(cli-foundation)に伝播させる。タイムアウト・リトライの
    概念はない（同期的なクレート内関数呼び出しのため）。project.mdのForbidden
    （unwrap/expect/panic禁止）を実装が遵守する。
  versioning: >
    正式なバージョニング機構は設けない。単一開発者体制であり、trait/型シグネチャの
    変更はコンパイルエラーとして即座に検出されるため、コードレビュー(自己レビュー)+
    CI green（既存スイート含む）で契約変更の安全性を担保する。
```

## Contract 2: AuditEntry出力仕様（audit-reader + cli-foundation → release-docs）

`release-docs`が移行ガイド・CHANGELOGを記述する際の入力となる、`audit`サブコマンドの
出力仕様。`audit-reader`がデータを供給し、`cli-foundation`の`OutputFormatter`が整形する
横断的関心事（unit-of-work.md参照）。

```yaml
shared-schema:
  name: AuditOutputContract
  owner: "audit-reader (U1, データ供給) / cli-foundation (U2, OutputFormatter整形)"
  consumer: release-docs (U3)
  formats:
    - name: table
      description: >
        人間可読のテーブル形式。既存のScanAggregator向けtable出力と同じ書式ルールを
        踏襲する（列幅・区切り文字等）。既定フォーマット（FR4.3）。
    - name: json
      description: >
        CLI/エージェント向けの構造化出力。AuditEntry構造体をそのままJSON配列として
        シリアライズする。
  malformed_line_behavior: >
    不正フォーマット・欠落フィールドを持つ行はスキップし、標準エラーへ警告を出力した
    上で、残りの正常な行の表示を継続する（FR4.4）。処理全体は中断しない。
  missing_file_behavior: >
    対象ファイルが存在しない場合は空扱い（0件表示）として正常終了する（FR4.6）。
  versioning: >
    Contract 1と同様、正式なバージョニングは設けずコードレビュー+CI greenで担保する。
```

## Contract 3: CLI引数・終了コード仕様（cli-foundation → release-docs）

`release-docs`が移行ガイド（旧`--scan-only`/`--execute`フラグ → 新`scan`/`clean`/`audit`
サブコマンドの対応表）を記述するための入力。既存のCLI外部インターフェース仕様
（requirements.md FR1-FR6）をそのまま引用する契約であり、新規の技術的取り決めは
生じない。

```yaml
shared-schema:
  name: CliInterfaceContract
  owner: cli-foundation (U2)
  consumer: release-docs (U3)
  reference: >
    requirements.md FR1.1(サブコマンド未指定時のエラー)、FR2.1-FR2.4(scan)、
    FR3.1-FR3.5(clean)、FR4.1-FR4.6(audit)、FR6.1(旧フラグ廃止・後方互換なし)。
    cli-foundationの実装がこれらのFRを満たすことを前提に、release-docsは
    実装確定後の最終仕様（旧フラグとの正確な対応表）を記述する。
  versioning: >
    Contract 1と同様。CLI引数の破壊的変更自体はFR6.1で既に決定済みであり、
    本契約が扱うのは「その決定をドキュメントへどう反映するか」の入力仕様のみ。
```

## Contract Ownership Rules

- 各契約は提供側（プロバイダ）Unitが所有する。Contract 2のみ、データ供給元
  （audit-reader）と整形実装（cli-foundation）の共同所有とする（unit-of-work.mdの
  FR4.3横断的関心事の扱いと整合）。
- 破壊的変更（trait/型シグネチャの削除・シグネチャ変更）は、単一開発者体制のため
  正式な承認プロセスを設けない。Rustのコンパイラが型不整合を即座に検出するため、
  変更者自身がコンパイル成立とCI green（既存スイート）を確認すれば契約変更として
  有効とする。
- 加法的変更（新規フィールド追加・新規enumバリアント追加等）は、既存の消費側
  コードを壊さない限り安全に追加できる。消費側は`#[non_exhaustive]`等の明示がない
  限り、将来のフィールド追加に備えてワイルドカードパターン(`..`)での分解を推奨する。

## Open Questions

| Contract | Question | Blocks |
|---|---|---|
| なし | なし（本ステージで解決すべき論点はすべて確認済み） | — |
