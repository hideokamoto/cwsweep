# Functional Design — Unit: cli-foundation — 質問

Inception（`requirements.md` FR1〜FR3・FR5・FR6、`components.md` CliApp、
`contract-summary.md` Contract 1/3）でサブコマンド体系・ハンドラ命名・
フラグ所属はほぼ確定している。既存実装（`src/cli.rs` / `src/main.rs`）を
確認した結果、設計判断が残るのは以下の3点である。

## Q1. `clean` で非TTY環境をどう扱うか

```question
prompt: "現行実装は標準入力がTTYでない場合、`--scan-only`相当（スキャン結果表示で終了）へ自動フォールバックしています。`clean`サブコマンド化後、非TTYでの`clean`呼び出しはどう扱うべきですか？"
header: "clean 非TTY挙動"
options:
  - label: "A. 現行どおりスキャン結果表示のみで正常終了（0）する"
    description: "後方互換を優先。ただし`scan`と`clean`の役割分離が曖昧になる"
  - label: "B. スキャン結果を表示した上で「対話式選択に進めない」旨を警告し正常終了（0）する（推奨）"
    description: "役割分離を明示しつつCI等での誤用でも破壊的操作へ進まない。`scan`の利用を案内する"
  - label: "C. エラー終了（非ゼロ）とし`scan`の利用を促す"
    description: "最も厳格。既存のCI利用があれば破壊的変更になる"
```

[Answer]: B. スキャン結果表示後に警告を出して正常終了する。`clean`は対話式選択を前提とするサブコマンドだが、非TTYで呼ばれた場合も破壊的操作には一切進まない（dry-run既定・確認プロンプト必須は不変）。CI/エージェント用途には`scan`を案内する。（Devin代理判断: 安全側かつ後方互換を大きく崩さない選択）

## Q2. `scan` の対象ゼロ件時のメッセージ

```question
prompt: "`scan`でロググループが0件だった場合の表示はどうしますか？"
header: "scan 0件表示"
options:
  - label: "A. 現行と同じ「削除対象のロググループはありません。」を表示する"
    description: "文言が`clean`寄り"
  - label: "B. 「ロググループは見つかりませんでした。」のように棚卸し文脈の文言にする（推奨）"
    description: "`scan`は削除前提ではないため中立的な文言にする。`--output json`では空配列を返し追加文言は標準出力に混ぜない"
```

[Answer]: B。`--output table`のときのみ0件メッセージを標準出力に出し、`--output json`では構造化出力のみ（`records: []`）とする。既存の`OutputFormatter`のJSON形は変更しない。

## Q3. `audit` のtable列構成

```question
prompt: "`audit --output table`の列構成はどうしますか？（Contract 2: 既存table出力と同じ書式ルールを踏襲）"
header: "audit table列"
options:
  - label: "A. 9フィールド全列（run_id含む）"
    description: "情報量最大。run_idは36桁で横幅を圧迫する"
  - label: "B. timestamp / event / account_id / region / log_group_name / action_kind / success / error_message の8列（run_idは省略）（推奨）"
    description: "run_idはjson出力で参照可能。tableは人間可読性優先"
```

[Answer]: B。tableは8列（`run_id`省略）、`json`は`AuditReadEntry`をそのまま配列でシリアライズし全9フィールドを含む（Contract 2）。`action_kind`はtableでは`Delete` / `SetRetention(30d)`の短縮表記、`success`は`intent`行では`-`表記とする（intent行の`success=false`は結果未確定プレースホルダであり成功/失敗判定に使ってはならない、audit-reader entities.md参照）。
