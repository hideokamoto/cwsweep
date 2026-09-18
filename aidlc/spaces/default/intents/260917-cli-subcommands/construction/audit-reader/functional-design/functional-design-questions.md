# Functional Design — Unit: audit-reader — 質問

Inception段階（`contract-summary.md` Contract 1）で仮置きされた `AuditEntry`
モデル（`timestamp` / `account_id` / `region` / `log_group_name` / `action` /
`result`）は簡略化された想定であり、実際の `src/audit.rs` の `AuditLogger` が
書き込むフィールド構成とは異なることが判明した（[desc] brownfield既存実装
`src/audit.rs`・`src/execution.rs` を確認）。実際のJSON Lines 1行は以下の
フィールドを持つ:

- `run_id`（実行単位のID）
- `timestamp`
- `account_id`
- `region`
- `log_group_name`
- `action_kind`（構造化された `Delete` / `SetRetention { days }`）
- `event`（`intent` または `result` — 1操作につき、API呼び出し「前」に
  `intent` 行、呼び出し「後」に `result` 行の**2行**が記録される。`intent`
  行の `success` は結果未確定のプレースホルダ `false`）
- `success`
- `error_message`（任意）

この2段階（intent/result）記録という実際の構造は、Inceptionの想定に
なかった新しい情報であり、`audit` コマンドの表示方針に関わる真の設計判断が
必要となる。

## Q1. intent/result 2種類のレコードをどう表示するか

```question
prompt: "実際の監査ログは1つの削除・retention変更操作につき「実行意図（intent、API呼び出し前）」と「結果（result、成功/失敗）」の2行を記録します。auditコマンドはこれをどう表示すべきですか？"
header: "intent/result表示方針"
multiSelect: false
options:
  - label: "A. 両方をそのまま時系列に全件表示する"
    description: "intent行・result行をログの記録順どおりすべて表示する。result行が存在しないintent行（監査ログ書き込み自体の失敗等で結果が記録されないまま中断したケース）が可視化できる利点があるが、1操作が2行に見えるため一覧性は下がる"
  - label: "B. result行のみ表示する（intent行は除外）"
    description: "各操作の最終結果のみを表示し、一覧をシンプルに保つ。ただしresult行が存在しない中断済み操作（安全上重要な情報になりうる）は見えなくなる"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 両方をそのまま時系列に全件表示する

## Q2. result行が存在しないintent行（未完了操作）の扱い

```question
prompt: "対応するresult行が存在しないintent行（＝操作が実行意図まで記録されたが結果が確定していない、または監査ログ書き込み失敗で処理自体が中断したケース）を、auditコマンドは特別に区別して表示すべきですか？"
header: "未完了操作の扱い"
multiSelect: false
options:
  - label: "A. 特別扱いせず、他の行と同じ形式でそのまま表示する（intent行はevent=intentとして識別可能）"
    description: "追加のロジックを持たず、AuditEntryをそのまま列挙する。event列を見ればintent/resultは区別できるため、利用者が目視で判断できる"
  - label: "B. 「未完了」として明示的に強調表示する"
    description: "result行のないintent行を検出し、table/json双方で追加のステータス（例: pending/incomplete）を付与して目立たせる。安全上の見落としを防ぐが、AuditReader/OutputFormatterに追加ロジックが必要になる"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 特別扱いせず、他の行と同じ形式でそのまま表示する（intent行はevent=intentとして識別可能）

## Consolidated Summary Confirmation

- 実際のAuditEntryフィールド構成（`run_id` / `timestamp` / `account_id` / `region` / `log_group_name` / `action_kind`(Delete/SetRetention{days}) / `event`(intent/result) / `success` / `error_message`）に基づいてentities.md・rules.mdを設計する（Inceptionの簡略化プレースホルダではなく実装済み`src/audit.rs`のスキーマに合わせる）
- audit表示方針: intent行・result行を両方とも時系列に全件表示する
- result行が存在しないintent行（未完了操作）も特別扱いせず、event列で識別可能な形でそのまま表示する

- Looks correct
- Request changes

[Answer]: Looks correct
