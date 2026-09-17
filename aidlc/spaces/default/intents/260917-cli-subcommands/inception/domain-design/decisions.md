# decisions.md — Architecture Decision Records (260917-cli-subcommands)

## ADR-001: CliAppを維持したままサブコマンド別ハンドラメソッドを追加する

### Status
Accepted

### Date
2026-09-17

### Context
現行の`CliApp`は`scan_all`→`render_output`→`select`→`plan`→`confirm`→`execute`の
オーケストレーションを一手に担う唯一のハブである。CLIをサブコマンド(`scan`/`clean`/
`audit`)へ再構成するにあたり、このオーケストレーション責務の持たせ方を決める必要が
あった。単一開発者体制（team.md Way of Working）で、破壊的変更として割り切る方針
（practices-discovery Q6）が既に確定しており、変更範囲の最小化と可逆性が優先される。

### Decision
`CliApp`構造体自体は維持し、`run_scan`/`run_clean`/`run_audit`という3つのサブコマンド
別ハンドラメソッドを追加する。既存の`scan_all`・`select`・`plan`・`confirm`・`execute`
等の内部メソッドはそのまま再利用し、`run_clean`がそれらを呼び出す。`run_scan`は
スキャン系コンポーネントのみ、`run_audit`は`AuditReader`のみを呼ぶ。

### Consequences

#### Positive
- 既存のhexagonal(ports-and-adapters)構造・依存関係グラフをほぼそのまま継承でき、
  変更差分が最小になる。
- 既存の単体テスト（`CliApp`メソッド単位のテスト）の多くをそのまま維持できる。
- 後から専用構造体への分割が必要になった場合も、`run_*`メソッドの中身を移すだけで
  済む（可逆性が高い）。

#### Negative
- `CliApp`は引き続き比較的多くのコンポーネントに依存する「ハブ」であり続ける
  （依存数自体は減らない）。将来的にサブコマンドが増えた場合、この構造では
  `CliApp`が肥大化するリスクがある。

#### Neutral
- サブコマンド分岐ロジック自体は`CliApp`(lib側)に実装し、`main.rs`は薄い配線層の
  ままとする（practices-discovery Q3の構造要件をそのまま適用）。

### Alternatives Rejected

#### Alternative: サブコマンドごとに専用の薄いオーケストレータ構造体を新設(ScanApp/CleanApp/AuditApp)
- Description: `CliApp`を廃止し、サブコマンドごとに専用の構造体を新設する。
- Pros: サブコマンドごとの責務がより明確に分離される。将来的にサブコマンドが
  増えても各構造体は肥大化しにくい。
- Cons: 変更差分が大きくなる（既存の`CliApp`メソッド呼び出しをすべて作り直す
  必要がある）。単一開発者体制・破壊的変更として割り切る方針の下では、今回の
  スコープに対して過剰な設計変更と判断した。

人間が「まかせる」と回答した上で、上記の可逆性・変更コストの観点からOption Aを
推奨し、その推奨どおり確定した（domain-design Q1）。

---

## ADR-002: `AuditReader`を新設コンポーネントとして分離する

### Status
Accepted

### Date
2026-09-17

### Context
`audit`サブコマンド(監査ログ閲覧)の新設にあたり、既存の`AuditLogger`/`AuditWrite`
（追記専用）とは別に、読み取り専用のコンポーネントが必要になった。practices-discovery
のQ4で、「`audit`ハンドラは削除・retention変更APIを実行しうる型（`ExecutionEngine`・
`AuditWrite`等）への依存を一切持たない」という制約が`project.md`の`## Forbidden`
（ハード制約）として既に確定している。

### Decision
`AuditLogger`とは独立した新規コンポーネント`AuditReader`を導入する。`AuditReader`は
`error`以外のコンポーネントに一切依存しない（`depends_on: []`）。ファイルの物理配置は
既存の`src/audit.rs`内とし（domain-design Q2）、型レベルでの分離（`AuditRead`相当の
トレイト・実装が`AuditWrite`とは別の型階層に属する）によって構造的強制を担保する。
監査ログの1行分は`AuditReader`が新規に所有する`AuditEntry`エンティティとして表現し、
`AuditLogger`側の既存の書き込み用型とは共有しない（domain-design Q3）。

### Consequences

#### Positive
- `AuditReader`の依存注入グラフに削除・retention変更・監査ログ書き込み系の型が
  一切含まれないことが、コンパイル時点で構造的に保証される
  （team-practices.md確定のNFR2「構造的安全性」を満たす）。
- `AuditEntry`を独立エンティティとして持つことで、将来`AuditLogger`側の書き込み
  フォーマットが変わっても、読み取り側の表示ロジックへの影響を局所化できる。

#### Negative
- 監査ログの「1エントリ」を表す型が2つ（書き込み用・読み取り用）に分かれるため、
  両者のJSON Lines表現の整合性（同じファイルを書いて読める）を別途テストで
  担保する必要がある。

#### Neutral
- ファイルは`src/audit.rs`に同居させるため、依存関係の非対称性はコードレビュー時に
  型定義を見て確認する必要がある（ファイル分割ほど視覚的には明確でない）。

### Alternatives Rejected

#### Alternative: 新規モジュール(`src/audit_read.rs`)として完全分離
- Description: ファイルレベルでも書き込み系(`audit.rs`)と読み取り系(`audit_read.rs`)
  を分離する。
- Pros: ファイルレベルでも非対称性が可視化され、レビュー時に見落としにくい。
- Cons: ファイル数が増え、監査ログフォーマットの知識が2ファイルに分散する。
  型レベルの分離で構造的強制の要件（project.md Forbidden）は既に満たせるため、
  ファイル分割は必須ではないと判断し、コスト対効果でOption Aを採用した。

人間がOption A（既存`src/audit.rs`に追加）を明示的に選択した（domain-design Q2）。

#### Alternative: 既存`AuditLogger`側の型（書き込み用）を読み取り時にも再利用する
- Description: 監査ログの1エントリを表す型を1つに統一する。
- Pros: 型が1つで済み、シリアライズ/デシリアライズの往復整合性が自明になる。
- Cons: 読み取り専用の`AuditReader`が、書き込み用の型定義（`AuditLogger`が所有）に
  依存することになり、依存注入グラフの構造的分離という目的そのものに反する。

人間が「AuditRead側で新規にAuditEntryエンティティを所有する」を明示的に選択した
（domain-design Q3）。

---

## ADR-003: `OutputFormatter`の責務を拡張し、監査エントリの出力にも対応させる

### Status
Accepted

### Date
2026-09-17

### Context
`audit`サブコマンドは`--output table|json`を持つ（requirements.md FR4.3）。
既存の`OutputFormatter`は`ScanAggregator`の出力専用に設計されている。

### Decision
新規コンポーネントを起こすのではなく、既存`OutputFormatter`の対象データを拡張し、
`AuditReader`が返す`AuditEntry`一覧のtable/JSON整形にも対応させる。

### Consequences

#### Positive
- table/JSONの書式ルール（列幅、JSON構造の一貫性等）を1箇所に集約でき、
  出力フォーマットの一貫性が保たれる。
- 新規コンポーネントを増やさずに済み、変更差分が小さい。

#### Negative
- `OutputFormatter`が2種類のデータ形状（`LogGroupRecord`一覧と`AuditEntry`一覧）を
  扱うことになり、内部実装が多少複雑になる（データ形状ごとに整形関数を分ける想定）。

#### Neutral
- `OutputFormatter`の依存元(dependents)は変わらず`CliApp`のみだが、依存先
  (depends_on)に`AuditReader`が加わる。

### Alternatives Rejected

#### Alternative: `AuditReader`専用の出力整形ロジックを`AuditReader`自身に持たせる
- Description: `AuditReader`が自身のtable/JSON整形メソッドを持つ。
- Pros: `OutputFormatter`を変更せずに済む。
- Cons: table/JSON整形という同じ関心事のロジックが2箇所に分散し、将来の書式変更時に
  修正漏れのリスクが生じる。`AuditReader`の「depends_on: []」という単純な依存
  グラフを維持する観点からも、整形はCliApp経由でOutputFormatterに委ねる方が
  一貫する。

この決定は明示的な人間確認を経ていないが、単一の妥当な選択肢と判断し（Domain Design
Step4冒頭のオプション提示ルールにおける「単一の分解しか妥当でない場合」に該当）、
承認ゲートで人間のレビューに委ねる。
