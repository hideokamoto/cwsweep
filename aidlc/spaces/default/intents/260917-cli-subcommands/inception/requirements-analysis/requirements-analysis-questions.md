# Requirements Analysis — 質問

このintentは brownfield (cwsweep既存リポジトリ)。practices-discoveryの人間インタビューで
既に7件の実装方針(lib側ディスパッチ、auditのForbidden昇格、run_接頭辞、後方互換なし、
不正ログ行の扱い、M6ゲート適用、開発フロー変更なし)が確定済みなので、ここでは
「サブコマンドのCLI引数面の具体的な仕様」に絞って確認する。

## Q1. `scan` サブコマンドのフラグ構成

現行の `Cli` 構造体は `--regions`(必須)・`--role-name`・`--output`(table|json)・
`--audit-log-path` を持つ。`scan` は読み取り専用サブコマンドとして、これらのうち
どれを引き継ぎますか?

```question
prompt: "scan サブコマンドは --regions（必須）・--role-name・--output（table|json）を引き継ぎ、--audit-log-path は不要（scanは監査ログに書き込まない）という理解でよいですか?"
header: "scanフラグ"
multiSelect: false
options:
  - label: "A. その理解でよい"
    description: "--regions/--role-name/--outputのみ。--audit-log-pathはscanには持たせない"
  - label: "B. --audit-log-pathも持たせる"
    description: "将来的な監査対象拡張に備え、scanにも受け付けさせておく（未使用でも）"
  - label: "C. 自由記述で伝える"
    description: "別の組み合わせを指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. その理解でよい

## Q2. `clean` サブコマンドのフラグ構成

`clean` は現行のデフォルトフロー(スキャン→対話式選択→確認→実行)を担う。

```question
prompt: "clean サブコマンドは --regions（必須）・--role-name・--execute・--audit-log-path を引き継ぎ、--output は不要（対話式選択のみでtable/json出力を切り替える理由がない）という理解でよいですか?"
header: "cleanフラグ"
multiSelect: false
options:
  - label: "A. その理解でよい"
    description: "--regions/--role-name/--execute/--audit-log-pathのみ"
  - label: "B. --outputも持たせる"
    description: "選択前のスキャン結果表示フォーマットを切り替えられるようにする"
  - label: "C. 自由記述で伝える"
    description: "別の組み合わせを指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. その理解でよい

## Q3. `audit` サブコマンドの絞り込みオプション

`audit` は新設の監査ログ閲覧サブコマンドで、`--audit-log-path` で指定した(既定
`cwsweep-audit.jsonl`)JSON Linesファイルを読み取り専用で表示する。

```question
prompt: "audit サブコマンドに、対象アカウントID・リージョン・成功/失敗などでの絞り込みオプションを持たせますか、それとも初回実装ではファイル全体をそのまま表示するだけ（絞り込みは--outputのjsonにパイプ+jq等の外部ツールに委ねる）にしますか?"
header: "audit絞り込み"
multiSelect: false
options:
  - label: "A. 初回は絞り込みなし（全件表示のみ）"
    description: "--output table|json のみ。絞り込みは外部ツール(jq等)に委ねる"
  - label: "B. 基本的な絞り込み（アカウントID・リージョン・成功/失敗）を持たせる"
    description: "--account/--region/--status等のフィルタフラグを追加する"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 初回は絞り込みなし（全件表示のみ）

## Q4. `audit` サブコマンドの出力フォーマット

```question
prompt: "audit サブコマンドの出力フォーマットは、scan と同様に --output table|json を持たせますか、それとも常にJSON Lines形式（元ファイルにほぼ近い形）で出力しますか?"
header: "audit出力形式"
multiSelect: false
options:
  - label: "A. --output table|json を持たせる（scanと統一）"
    description: "table既定、jsonでCLIエージェント向け出力も可能にする"
  - label: "B. 常にJSON Lines形式で出力"
    description: "元の監査ログ形式に近い形でそのまま表示（スキップ行は警告として標準エラーへ）"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. --output table|json を持たせる（scanと統一）

## Q5. バージョニング・リリースノート方針

practices-discoveryのQ6で「破壊的変更として割り切る」ことは確定済み。バージョン番号の
扱いを確認する。

```question
prompt: "このサブコマンド化を含むリリースは、Cargo.tomlのバージョンを 0.1.0 から上げますか（例: 0.2.0）、それとも本intentの作業自体はバージョン変更を含まず、別途リリース作業時に決めますか?"
header: "バージョニング"
multiSelect: false
options:
  - label: "A. 本intentでは0.2.0へ上げる"
    description: "破壊的変更を明示するため、このintentの成果物にバージョン変更を含める"
  - label: "B. バージョン変更はこのintentのスコープ外"
    description: "コード変更のみ。リリースタイミングは別途判断する"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 本intentで0.2.0へ上げる

## Consolidated Summary Confirmation

- scan サブコマンド: `--regions`（必須）・`--role-name`・`--output`(table|json) を引き継ぎ、`--audit-log-path` は持たない
- clean サブコマンド: `--regions`（必須）・`--role-name`・`--execute`・`--audit-log-path` を引き継ぎ、`--output` は持たない
- audit サブコマンド: 初回実装では絞り込みオプションを持たず、ファイル全体を表示する
- audit サブコマンドの出力形式: `--output` table|json を持たせ、scan と統一する
- バージョニング: 本intentの成果物で Cargo.toml のバージョンを 0.1.0 から 0.2.0 へ上げる

Does this all look correct before I generate the requirements artifact?

[Answer]: Looks correct
