# Delivery Planning — 質問

本ステージは、Units Generation(2.7)が生成した依存DAG（audit-reader→cli-foundation→
release-docsの一方向チェーン）を使って、Construction段階でUnit群をどの順序・単位で
実施するか（Bolt——1回のビルドパスで実行可能な状態になる、DAGの一部を切り出した
作業単位）を決める。DAGのトポロジー自体はここでは変更しない。

## 戦略的質問（プロジェクト全体で1つの答え）

### Q1. 何を最初に作るか

```question
prompt: "最初に着手すべきはどれですか？（リスクが高い部分／価値が高い部分／アーキテクチャ全層を貫く薄い一枚岩（ウォーキングスケルトン）／その混合）"
header: "着手順の方針"
multiSelect: false
options:
  - label: "A. DAGのトポロジカル順そのまま（audit-reader→cli-foundation→release-docs）"
    description: "team.mdはWalking Skeletonを作らない方針（skeleton: off）であり、DAG自体が単一チェーンのため、リスク優先・価値優先いずれの観点からも独立して並べ替える余地が乏しい。新設のaudit-readerは既存の破壊的操作系コンポーネントに一切依存しない独立実装であり最も低リスクなため最初に、cli-foundationは既存12コンポーネントの再編＋破壊的操作(clean)を含むため2番目に、release-docsは両者の確定を前提とするため最後に置く"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. DAGのトポロジカル順そのまま（推奨どおり確定）

## Q2. スコアリングモデル（WSJFの採用可否）

```question
prompt: "Bolt順序をWSJF（価値・緊急度・規模による正式なスコアリング）で決めますか？"
header: "スコアリング方針"
multiSelect: false
options:
  - label: "A. 正式なスコアリングモデルは採用しない"
    description: "Bolt数がわずか3件、依存関係が単一チェーンで実質的に順序の自由度がないため、WSJF等の形式的スコアリングを導入するコストに見合う判断の複雑さがない。順序の根拠はrisk-and-sequencing-rationale.mdにDAG制約＋リスク論として記述する"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 正式なスコアリングモデルは採用しない（推奨どおり確定）

## Q3. Boltの粒度

```question
prompt: "1 Boltの規模はどうしますか？（単一Unit／複数Unitの束ね／Unit横断の薄いスライス）"
header: "Bolt粒度"
multiSelect: false
options:
  - label: "A. 1 Bolt = 1 Unit（3 Bolt構成）"
    description: "audit-reader・cli-foundation・release-docsそれぞれを独立したBoltとする。Unit分割自体が既にunit-of-work.mdで機能ストリーム別・粗粒度に設計されているため、Bolt粒度とUnit粒度を一致させるのが最もシンプル"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 1 Bolt = 1 Unit、3 Bolt構成（推奨どおり確定）

## Q4. Bolt並行実施の可否

```question
prompt: "複数のBoltを同時に進められますか、それとも1つずつ順番に進める必要がありますか？"
header: "並行実施"
multiSelect: false
options:
  - label: "A. 1つずつ順番に進める"
    description: "team.mdのWay of Working（単一開発者体制、Bolt = 1コミット）およびConstruction StaffingがAI単独セッション（aidlc-developer-agent）である前提から、実務上並行実施する体制がない。DAG上はaudit-readerとcli-foundationの一部が並行実施可能な余地があるが（unit-of-work-dependency.md「並行開発の余地」参照）、本Bolt計画では採用しない"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 1つずつ順番に進める（推奨どおり確定）

## Q5. 外部依存（このチーム外の要因でブロックされるもの）

```question
prompt: "このチーム外の要因（外部API・データ・承認・他チームの引き渡し等）でブロックされるものはありますか？"
header: "外部依存"
multiSelect: false
options:
  - label: "A. なし（外部依存は存在しない）"
    description: "cwsweepは単一Cargoパッケージのスタンドアロンバイナリであり、AWS Organizations/CloudWatch Logs/STS APIへの依存は既存実装のまま変更されない。新規の外部承認・他チームハンドオフ・データ提供待ちは生じない。唯一の human gateはM6（--execute実装Bolt）の人間承認であり、これは外部依存ではなく既存の内部プロセス"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. なし（推奨どおり確定）

## Q6. 最も懸念される点

```question
prompt: "この開発で最も懸念される点は何ですか？早期に着手すべき対象を教えてください"
header: "最大の懸念"
multiSelect: false
options:
  - label: "A. audit-readerの構造的分離（project.md Forbidden制約）とcli-foundationの既存回帰リスクの2点"
    description: "①audit-readerがExecutionEngine/AuditWrite等に型レベルで到達不能であることの実装・検証（NFR2）、②cli-foundationが既存12コンポーネントの呼び出し順序を再編する際に既存の安全機構（Identity二重検証・dry-run既定・監査ログ必須）を壊さないこと（既存テストスイートを緑に保つ）。Bolt順序（Q1）で両方とも早期（Bolt 1・2）に配置済み"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. audit-readerの構造的分離とcli-foundationの既存回帰リスク（推奨どおり確定）

## Per-Bolt質問

### Bolt 1: audit-reader

```question
prompt: "Bolt 1（audit-reader）の詳細を確認してください"
header: "Bolt 1詳細"
multiSelect: false
options:
  - label: "A. 以下の内容で確定する"
    description: "含むUnit: U1 audit-reader。ウォーキングスケルトンではない（skeleton: off）。完了条件: AuditReader/AuditEntry実装、不正行スキップ+警告テスト、NFR2構造的回帰テスト、既存スイートgreen。証明すること: project.md Forbidden制約（削除・retention変更系への非依存）を型レベルで満たせるか。担当: aidlc-developer-agent"
  - label: "C. 自由記述で伝える"
    description: "別の内容を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. この内容で確定（推奨どおり）

### Bolt 2: cli-foundation

```question
prompt: "Bolt 2（cli-foundation）の詳細を確認してください"
header: "Bolt 2詳細"
multiSelect: false
options:
  - label: "A. 以下の内容で確定する"
    description: "含むUnit: U2 cli-foundation。ウォーキングスケルトンではない。完了条件: Cli/Commands再構成、run_scan/run_clean/run_audit実装、旧フラグ廃止、既存スイートgreen、新規CLI層統合テスト。証明すること: 既存12コンポーネントの安全機構（Identity二重検証・dry-run既定・監査ログ必須）を破壊的変更後も維持できるか。担当: aidlc-developer-agent。--execute実装を含むため、team.md Walking Skeletonの方針によりBolt完了時に人間の明示承認ゲートを必須とする"
  - label: "C. 自由記述で伝える"
    description: "別の内容を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. この内容で確定（推奨どおり）

### Bolt 3: release-docs

```question
prompt: "Bolt 3（release-docs）の詳細を確認してください"
header: "Bolt 3詳細"
multiSelect: false
options:
  - label: "A. 以下の内容で確定する"
    description: "含むUnit: U3 release-docs。ウォーキングスケルトンではない。完了条件: README/CHANGELOG更新、移行ガイド（旧フラグ対応表）記載、Cargo.tomlバージョン0.2.0への更新。証明すること: 破壊的変更が利用者に正しく伝わる形で文書化されているか。担当: aidlc-developer-agent"
  - label: "C. 自由記述で伝える"
    description: "別の内容を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. この内容で確定（推奨どおり）

## Consolidated Summary Confirmation

- Bolt順序: audit-reader(Bolt1) → cli-foundation(Bolt2) → release-docs(Bolt3)、DAGトポロジカル順のまま
- スコアリング: WSJF等の正式モデルは不採用
- Bolt粒度: 1 Bolt = 1 Unit
- 並行実施: なし（単一開発者体制・1つずつ順番）
- 外部依存: なし
- 最大の懸念: audit-readerの構造的分離とcli-foundationの既存回帰リスク（両方とも早期Boltに配置）
- Bolt 2（--execute実装含む）はteam.md Walking Skeleton方針により完了時に人間承認ゲート必須

- Looks correct
- Request changes

[Answer]: Looks correct
