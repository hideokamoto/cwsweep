# NFR Requirements — Unit: cli-foundation — 質問

Inceptionの NFR1（テスト容易性）・NFR4（後方互換の意図的放棄）・NFR5（既存NFR継続）は
本Unitに直接割り当てられ、NFR2（構造的安全性）は `run_audit` の依存構成として本Unitにも
及ぶ。定量目標が未確定なのは以下の2点。

## Q1. ディスパッチ層のカバレッジ目標

```question
prompt: "NFR1 は lib 全体の 80% ライン floor を要求します。新設ディスパッチ層（Cli/Commands 解析・run_* 分岐）自体の目標はどうしますか？"
header: "ディスパッチ層カバレッジ"
options:
  - label: "A. 全体 floor(80%) のみ"
  - label: "B. 引数解析・分岐は lib 単体テストで全分岐を網羅し、全体 floor も維持する（推奨）"
```

[Answer]: B。clap の `Commands` 3バリアント＋未指定エラー＋所属外オプション拒否の各分岐を
lib 単体テストで網羅する。AWS I/O を含む `run_scan`/`run_clean` 本体は既存のモックアダプタ
（`scan_all` テスト）を再利用し、全体 80% floor を維持する。

## Q2. 旧フラグ廃止の固定化テスト範囲

```question
prompt: "NFR4 の回帰テストとして、どの旧形式呼び出しをパースエラーとして固定化しますか？"
header: "旧フラグ固定化"
options:
  - label: "A. `--scan-only` のみ"
  - label: "B. `cwsweep --regions ...`（サブコマンド無し）、`--scan-only`、トップレベル `--execute`、`scan --execute`、`scan --audit-log-path`、`clean --output` の6形（推奨）"
```

[Answer]: B。上記6形を lib テストで `Err` として固定化する。
