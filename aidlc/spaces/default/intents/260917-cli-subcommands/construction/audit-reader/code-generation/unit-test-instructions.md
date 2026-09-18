# unit-test-instructions.md — Unit: audit-reader

## テストフレームワーク

既存プロジェクトと同一（`cargo test`、`#[cfg(test)]`インラインモジュール、
`tempfile`クレートで一時ディレクトリ/ファイルを作成 — 追加クレート不要）。

## 本Unitのテスト実行コマンド（厳密スコープ）

```
cargo test --lib audit_reader_tests:: --locked
```

新設する`#[cfg(test)] mod audit_reader_tests`配下のテストのみを対象と
する。既存の`mod tests`（`AuditLogger`向け）には触れない。プロジェクト
全体の`cargo test`は実行しない。

## 期待されるテストケース（Standard戦略: 5〜8件）

1. `audit_read_entry_roundtrips_through_json` — `AuditReadEntry`のJSON
   シリアライズ→デシリアライズが元の値と一致すること
2. `audit_read_entry_deserialization_fails_when_required_field_missing` —
   必須フィールド（例: `event`）欠落時に`serde_json::from_str`が`Err`を
   返すこと
3. `entries_returns_empty_outcome_when_file_does_not_exist` — ファイル
   不在時に`Ok(AuditReadOutcome { entries: vec![], skipped_lines: vec![] })`
   相当が返ること（BR2.1）
4. `entries_preserves_order_across_multiple_valid_lines` — 複数の正常な
   行が、ファイル中の出現順のまま`entries`に格納されること（BR3.2）
5. `entries_skips_malformed_line_and_continues_with_warning` —
   少なくとも1行の不正フォーマット行を含むフィクスチャで、(a)
   不正行が`skipped_lines`に記録され警告が出力されること、(b) 不正行の
   前後にある正常なエントリが引き続き`entries`に含まれること
   （team-practices.md Q7・NFR3.1）
6. `entries_returns_io_error_when_path_is_not_a_regular_file` — ファイル
   不在以外のI/Oエラー（例: パスがディレクトリ）で`AuditReadError::Io`
   が返ること（BR2.2）
7. `entries_never_creates_or_writes_the_target_file` — `entries()`
   呼び出し前後でファイルが新規作成されない・内容が変化しないこと
   （BR4.1/NFR3.3）
8. `audit_reader_module_has_no_execution_or_write_dependency` — 構造的
   分離の静的回帰テスト（NFR2.1/NFR2.2、多層防御の一部であることを
   テストのdocコメントに明記する）

## カバレッジ目標

- 通常コード: 80%行カバレッジ floor（`cargo llvm-cov --lib`）
- 異常系分岐（パース失敗・フィールド欠落・ファイル不在・ファイル不在
  以外のI/Oエラー）: 100%パスカバレッジ + 境界値テスト（team.md
  Testing Posture）。上記テストケース3・5・6がこの異常系分岐をカバー
  する。ただしこの100%floorを既存の`coverage`ジョブ（単一グローバル
  80%閾値）が機械的に強制しない点はinfrastructure-design段階で
  GAPとして記録済み — Build and Testステージでの検証方法確定を待つ。

## モック/スタブ方針

外部依存なし（ファイルシステムのみ）。`tempfile::tempdir()`で
一時ディレクトリを作成し、実ファイルへの読み取りをそのまま検証する
（既存`src/audit.rs`のテストパターンと同一）。

## テストデータ管理

各テストが専用の一時ディレクトリ・フィクスチャファイルを作成する
（既存パターンに準拠、テスト間の共有可変状態なし）。
