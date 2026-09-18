# tech-stack-decisions.md — Unit: audit-reader

`audit-reader`（`AuditReader`/`AuditRead`）は、既存の技術スタック
（`aidlc/spaces/default/codekb/cwsweep/technology-stack.md`）に新規
クレートを追加しない。既存の依存関係を再利用する。

## 選定と根拠

| 用途 | 選定 | 根拠 |
|---|---|---|
| JSON Lines のパース | `serde_json`（既存依存、`^1.x`） | 書き込み側`AuditLogger`（`src/audit.rs`）が既に`serde_json::to_string`で監査ログを書き込んでおり、読み取り側も同一クレートで対称的にパースするのが最も自然。新規依存を増やさずシリアライズ/デシリアライズの意味論を統一できる |
| ファイルの行単位ストリーミング読み取り | 標準ライブラリ `std::fs::File` + `std::io::{BufReader, BufRead}` | 監査ログはサイズが線形に増加しうるため、全体を一括ロードせず行単位でストリーミング読み取りする（`entities.md`/`functional-spec.md`のFR4.1〜FR4.4要件）。標準ライブラリのみで実現可能であり、新規クレートは不要 |
| エラー型 | `thiserror`（既存依存） | 既存の`src/error.rs`と同じエラー定義パターン（`AuditReadError::Io(std::io::Error)`）を踏襲し、プロジェクト全体のエラーハンドリング規約（project.md Code Style: `Result`伝播、`unwrap`/`expect`/`panic!`禁止）と一貫させる |
| 構造体定義 | `serde::{Serialize, Deserialize}`（既存依存） | `AuditEntry`は書き込み側と同じフィールド構成で`Deserialize`を導出する。新規クレートは不要 |

## 明示的に採用しないもの

- **新規のログパース専用クレート**（例: 汎用JSON Lines読み取りライブラリ）:
  `serde_json`とストリーミング読み取りの組み合わせで要件を満たせるため、
  依存関係を増やし`cargo audit`/`cargo deny check`のCI対象を広げる理由が
  ない（team.md Code Style: 依存関係の脆弱性スキャン・サプライチェーン
  チェックのCI必須ゲート）。
- **`chrono`によるタイムスタンプの厳密パース**: `timestamp`フィールドは
  文字列としてそのまま保持・表示するのみで、日時としての演算・検証は
  行わない（rules.md BR1.2は「フィールドが存在すること」のみを要求し、
  フォーマット妥当性の検証は要件外）。既存の`chrono`依存はAuditLogger側の
  書き込み用途のままとし、読み取り側での新たな利用は導入しない。

## CI/CDへの影響

新規クレートを追加しないため、既存のCircleCI設定（`fmt`/`clippy`/`test`/
`coverage`/`audit`/`deny`ジョブ）に変更は不要。`cargo llvm-cov --lib`の
カバレッジ計測対象に`audit-reader`Unitの新規モジュールがそのまま含まれる
（`lib`クレート側に実装するため、team.md確定事項Q3のスコープと整合する）。
