# Contract Design — 質問

本intentはUnits Generationで確定した3 Unit（audit-reader → cli-foundation → release-docs、
一方向の依存チェーン）を持つ。統合点はすべて同一Cargoパッケージ内の型シグネチャであり、
ネットワークAPI・非同期イベントは存在しない（unit-of-work-dependency.md参照）。

## Q1. Public/external API surface

```question
prompt: "このintentのいずれかのUnitは、システム外部（パートナー・公開インターネット等）に消費されるAPIを公開しますか?"
header: "外部API有無"
multiSelect: false
options:
  - label: "A. 外部APIは公開しない（CLIの引数・終了コード・標準出力/標準エラーの形式は既にrequirements.mdのFR群で規定済みであり、本ステージで別途OpenAPI/AsyncAPI契約を新設する対象ではない）"
    description: "cwsweepはスタンドアロンのCLIバイナリであり、ネットワーク越しに消費されるAPIを持たない。CLIの引数・終了コード契約はrequirements.md FR2.4(R-03)等で既に規定済み"
  - label: "B. CLIの引数・出力形式を公式な外部契約として本ステージでも別途文書化する"
    description: "スクリプト・自動化ツールからの呼び出しを想定し、CLI引数・終了コード・出力フォーマットをcontract-summary.mdにも契約として明記する"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 外部APIは公開しない（推奨どおり確定）

## Q2. Unit間統合の契約形式

```question
prompt: "3件のUnit間統合点（audit-reader→cli-foundation、cli-foundation→release-docs、audit-reader→release-docs）を、どの契約形式で記述しますか?"
header: "契約形式"
multiSelect: false
options:
  - label: "A. shared-schemaブロック（Rustの型シグネチャをYAML形式で記述）"
    description: "OpenAPI/AsyncAPIはHTTP/イベント駆動システム向けであり、同一クレート内の関数呼び出し・trait契約には過剰。トレイト・構造体のシグネチャをshared-schema形式のYAMLで表現する"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. shared-schemaブロック（単一の妥当な選択肢のため人間確認なしで確定）

## Q3. 契約の所有権

```question
prompt: "各契約（インターフェース仕様）の所有Unitはどう決めますか?"
header: "契約所有権"
multiSelect: false
options:
  - label: "A. 提供側（プロバイダ）Unitが所有する（例: AuditRead traitはaudit-readerが所有）"
    description: "一般的なプロバイダ/コンシューマ契約と同じ考え方。単一開発者体制のため所有権の衝突は生じない"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 提供側Unitが所有（単一の妥当な選択肢のため人間確認なしで確定）

## Q4. バージョニング・破壊的変更ポリシー

```question
prompt: "Unit間契約の破壊的変更ポリシーはどうしますか?"
header: "バージョニング方針"
multiSelect: false
options:
  - label: "A. v1未リリース・単一開発者体制のため、正式なバージョニングは設けず、契約変更はコードレビュー（自己レビュー）とCI green（既存スイート）で担保する"
    description: "team.md Way of Workingの通り、複数人承認は求めない体制。Unit間の型シグネチャ変更はコンパイルエラーとして即座に検出されるため、追加のバージョニング機構は過剰と判断"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 正式バージョニングなし・コードレビュー+CI greenで担保（単一の妥当な選択肢のため人間確認なしで確定）

## Q5. エラー・タイムアウト・リトライの振る舞い

```question
prompt: "Unit間境界でのエラー処理方針はどうしますか?"
header: "エラー処理方針"
multiSelect: false
options:
  - label: "A. 全境界が同期的なクレート内関数呼び出しのため、タイムアウト・リトライの概念はなく、エラーは`Result<T, E>`で呼び出し元に伝播させる（project.md Forbidden: unwrap/expect/panic禁止の既存方針をそのまま適用）"
    description: "ネットワーク境界がないため分散システム的なエラー処理（サーキットブレーカー等）は不要。既存のRustエラー伝播規約をそのまま契約とする"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. Result<T,E>伝播・タイムアウト/リトライ概念なし（単一の妥当な選択肢のため人間確認なしで確定）

## Consolidated Summary Confirmation

- 外部API: 公開しない（Q1:A）
- 契約形式: shared-schemaブロック（Q2:A）
- 契約所有権: 提供側Unitが所有（Q3:A）
- バージョニング: 正式なものは設けず、コードレビュー+CI greenで担保（Q4:A）
- エラー処理: Result<T,E>伝播、タイムアウト/リトライ概念なし（Q5:A）
- 対象契約は3件: audit-reader→cli-foundation（AuditRead trait）、cli-foundation→release-docs（Commands::Audit仕様）、audit-reader+cli-foundation→release-docs（AuditEntry出力仕様）

- Looks correct
- Request changes

[Answer]: Looks correct
