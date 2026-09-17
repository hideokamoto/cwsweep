# Practices Discovery — インタビュー

このintent(`260917-cli-subcommands`)は、cwsweepの既存プロジェクトに対する再実行(re-run)です。
`team.md`/`project.md`にすでに承認済みの方針があるため、そのまま引き継げる部分は質問せず、
今回のCLIサブコマンド化(`scan`/`clean`/`audit`、`config`は対象外)によって新たに判断が必要になった点だけを確認します。

## Q1. 開発フロー・デプロイ方針は変更なしでよいか

トランクベース開発・squash-merge・タグ駆動リリースなど、既存の開発フロー/デプロイ方針は
今回のCLI再構成でも変わらないと判断していますが、これでよいですか?

```question
prompt: "既存の開発フロー(トランクベース開発・squash-merge)とデプロイ方針(タグ駆動リリース)は、今回のCLIサブコマンド化でも変更なしという理解でよいですか?"
header: "開発フロー"
multiSelect: false
options:
  - label: "変更なしでよい"
    description: "既存のteam.mdの記述をそのまま引き継ぐ"
  - label: "見直したい点がある"
    description: "自由記述で伝える"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]:

## Q2. `clean`サブコマンドの`--execute`実装Boltは、引き続き毎回ゲート対象か

「破壊的操作(ログ削除・retention変更)を実装するBoltは毎回、人間の明示承認を経てから次に進む」という
既存の合意(Walking Skeleton節)は、`--execute`によるログ削除・retention変更が`clean`サブコマンドに
移った後も、そのまま`clean`の実装Boltに適用される、という理解でよいですか?

```question
prompt: "「破壊的操作を実装するBoltは毎回ゲートする」という既存ルールは、今後 `clean` サブコマンド(旧 `--execute` 相当)の実装Boltにそのまま適用される、という理解でよいですか?"
header: "ゲート要件"
multiSelect: false
options:
  - label: "その理解でよい"
    description: "cleanサブコマンドの実装Boltも毎回ゲートする"
  - label: "違う運用にしたい"
    description: "自由記述で伝える"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]:

## Q3. サブコマンド分岐ロジックの置き場所(カバレッジ計測の抜け穴対策)

品質担当エージェントの指摘: 現状のCIカバレッジ計測(`cargo llvm-cov --lib`)は `src/main.rs`(binクレート)を
計測対象外にしています。もし新しい`Commands`(scan/clean/audit)の分岐ロジックを`main.rs`に書くと、
80%/100%のカバレッジ基準の対象から漏れてしまいます。

```question
prompt: "新しいサブコマンド分岐ロジックをカバレッジ計測対象にするため、分岐処理を lib 側(cli.rs 等、cargo llvm-cov --lib の対象)に置き、main.rs は薄い呼び出しだけにする、という方針でよいですか?"
header: "カバレッジ対象"
multiSelect: false
options:
  - label: "lib側に置く方針でよい"
    description: "既存のCliApp設計(トレイト経由の依存注入)を踏襲し、分岐ロジックをlib側でテスト可能にする"
  - label: "coverageジョブの対象範囲を広げる"
    description: "main.rsも含めて計測できるようCI設定側を変更する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]:

## Q4. `audit`サブコマンドを構造的に読み取り専用にするか、その強制レベル

開発・DevSecOps両エージェントが、新しい`audit`(監査ログ閲覧)サブコマンドについて指摘しています:
現状の`AuditLogger`/`AuditWrite`は追記専用で、閲覧用の読み取りAPIが存在しません。
DevSecOps担当は、「`audit`ハンドラは削除・retention変更APIへの依存を一切持たない(型レベルで
削除系操作を実行できない)」という制約を、`project.md`の`## Forbidden`(ハード制約・機械的強制対象)
に昇格させることを推奨しています。

```question
prompt: "「audit サブコマンドは削除・retention変更のAPIに一切依存できない(構造的に読み取り専用)」という制約を、破ったら即アウトの project.md Forbidden ルール(監査ログ書き込み失敗時の扱いなどと同格)にしますか、それとも team.md の設計方針レベルの緩いガイダンスに留めますか?"
header: "audit読み取り専用"
multiSelect: false
options:
  - label: "project.md Forbiddenにする"
    description: "他の破壊的操作系ルールと同格のハード制約として明文化する"
  - label: "team.mdの設計方針に留める"
    description: "コードレビューで担保する程度のガイダンスとする"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]:

## Q5. ハンドラ関数の命名規則

開発担当エージェントの指摘: リード案は`run_scan`/`run_clean`/`run_audit`という`run_`接頭辞を
提案していますが、既存コードの`CliApp`メソッド(`scan_all`・`execute`・`plan`など)はこの接頭辞を
使っていません。

```question
prompt: "新しいサブコマンドのハンドラ関数の命名は、提案されている run_scan / run_clean / run_audit のような run_ 接頭辞にしますか、それとも既存コードに合わせて scan_all / execute のような接頭辞なしの動詞形にしますか?"
header: "命名規則"
multiSelect: false
options:
  - label: "run_ 接頭辞にする"
    description: "サブコマンドのエントリポイントであることが名前から分かる"
  - label: "既存に合わせ接頭辞なしにする"
    description: "コードベース全体の一貫性を優先する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]:

## Q6. 旧フラグ(`--scan-only`/`--execute`)の後方互換

サブコマンド化は破壊的変更(`cwsweep --regions ... --execute` のような既存の呼び出し方が
動かなくなる)です。

```question
prompt: "cwsweepはまだv1未満(0.1.0)の段階ですが、サブコマンド化にあたって旧フラグ(--scan-only・--execute)のエイリアスや後方互換は用意しますか、それとも破壊的変更として割り切りますか?"
header: "後方互換"
multiSelect: false
options:
  - label: "割り切って破壊的変更にする"
    description: "v1未満なので後方互換は用意しない。README/CHANGELOGで明示する"
  - label: "後方互換のエイリアスを用意する"
    description: "旧フラグ形式も一定期間サポートする"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]:

## Q7. `audit`サブコマンドが不正な監査ログ行を読んだ場合の挙動

開発担当エージェントの指摘: 監査ログの一部が破損/不正な形式だった場合、`audit`サブコマンドは
処理全体を中断すべきか、その行をスキップして警告を出し継続すべきか、未確定です。

```question
prompt: "audit サブコマンドが監査ログの中に壊れた/不正な形式の行を見つけた場合、そこで処理を中断してエラーにしますか、それともその行をスキップして警告を出しつつ残りを表示しますか?"
header: "不正ログ行の扱い"
multiSelect: false
options:
  - label: "スキップして警告し継続する"
    description: "閲覧用途なので可能な限り読める範囲を見せる"
  - label: "中断してエラーにする"
    description: "監査ログの整合性を疑わせる異常として扱う"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]:

## Consolidated Summary Confirmation

以下の7件の決定事項を`team-practices.md`/`discovered-rules.md`/`evidence.md`/`practices-discovery-timestamp.md`に統合しました:

1. 開発フロー・デプロイ方針: 変更なし(既存を継承)
2. Walking Skeleton: `clean`サブコマンドの`--execute`実装Boltにも毎回ゲート要件を適用
3. カバレッジ対象: サブコマンド分岐ロジックをlib側(cli.rs等)に置き、main.rsは薄い呼び出しのみにする
4. `audit`読み取り専用: project.md Forbiddenへ昇格(削除・retention変更APIへの依存を型レベルで禁止)
5. 命名規則: `run_scan`/`run_clean`/`run_audit`(`run_`接頭辞)
6. 後方互換: 用意しない(破壊的変更として割り切る、pre-v1)
7. `audit`の不正ログ行: スキップして警告を出し継続する

Does this all look correct before I generate the artifact?

[Answer]: Looks correct
