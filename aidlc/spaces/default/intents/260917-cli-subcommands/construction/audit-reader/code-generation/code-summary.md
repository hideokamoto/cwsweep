# code-summary.md — Unit: audit-reader

承認済み `code-generation-plan.md` の Step 1〜4（Step 5・6を除く実装部分）を
`src/audit.rs` の既存コードに対する拡張として実装した。新規ファイルは
作成していない。

## 実装したもの

`src/audit.rs` の末尾（既存の `AuditLogger`/`AuditWrite`定義の直後、
`#[cfg(test)] mod tests { ... }` の直前）に、`// === AuditRead (read-only)
region begin/end ===` マーカーで囲んだ読み取り専用の新領域を追加した。

- `AuditReadEntry`（`Debug, Clone, PartialEq, Serialize, Deserialize`）:
  `run_id` / `timestamp` / `account_id` / `region` / `log_group_name` /
  `action_kind: ActionKind`（`planner::ActionKind`を再利用） /
  `event: AuditEventKind`（既存の`audit::AuditEventKind`を再利用） /
  `success: bool` / `error_message: Option<String>` の9フィールド。
  既存の書き込み側 `AuditEntry` と同一スキーマだが、型としては意図的に
  分離した別の構造体である（entities.md準拠）。
- `SkippedLine { line_number: usize, reason: String }`。
- `AuditReadOutcome { entries: Vec<AuditReadEntry>, skipped_lines:
  Vec<SkippedLine> }`。
- `AuditReadError`（`thiserror`導出、`Clone, PartialEq, Eq`）: `Io(String)`
  の1バリアントのみ。`std::io::Error`を直接ラップせず、メッセージ文字列を
  保持する（`std::io::Error`は`Clone`/`Eq`を導出できないため。advisory
  reviewのR-02指摘への対応で、`src/error.rs`の既存エラー型群と同じ設計）。
- `AuditRead`トレイト: `fn entries(&self) -> Result<AuditReadOutcome,
  AuditReadError>`。
- `AuditReader { path: PathBuf }`（`impl AuditRead for AuditReader`）:
  - `File::open`で読み取り専用に開き、`io::ErrorKind::NotFound`の場合は
    エラーとせず空の`AuditReadOutcome`を返す（BR2.1）。
  - それ以外のオープン失敗、または`BufReader::lines()`の途中で発生する
    I/Oエラー（例: パスがディレクトリの場合のEISDIR相当）は
    `AuditReadError::Io`として呼び出し元へ伝播する（BR2.2）。
  - 行ごとに`serde_json::from_str::<AuditReadEntry>`を試み、失敗した行は
    `SkippedLine`へ記録し`eprintln!`で警告を出力した上で処理を継続する
    （BR1.1/BR1.2、team.md Q7の異常系挙動）。
  - `entries`/`skipped_lines`はファイル中の出現順をそのまま保持する
    （BR3.2）。
  - `OpenOptions::write`/`create`/`append`を一切使用せず、書き込み・
    ファイル作成を行わない（BR4.1/NFR3.3）。

## Step 4: 構造的分離の回帰テスト

`AuditReader`/`AuditRead`関連コードを `// === AuditRead (read-only) region
begin/end ===` マーカーで囲み、`audit_reader_module_has_no_execution_or_write_dependency`
テストがそのマーカー間のソーステキストに `ExecutionEngine` / `AuditWrite` /
`AuditLogger` の識別子が一切出現しないことを検証する。

実装上の注意点として、マーカー領域の内側にあるdocコメントで禁止識別子を
（「〜への依存を持たない」という説明のために）名指しすると、このテスト
自体が自己言及的に失敗することが実装中に判明した。そのため、禁止識別子を
名指しした説明文（project.md Forbiddenの引用等）はマーカーの**外側**
（`AuditLogger`の既存定義の直後、マーカー開始行の直前）に置き、マーカー
**内側**のdocコメントでは「本ファイル冒頭で定義される書き込み側の型」
のように一般化した表現に置き換えた。これはテストの検出漏れではなく、
むしろ「禁止識別子への型参照が実際に一切存在しない」ことをより厳密に
示す結果である。

このテストは advisory review（nfr-design R-01）が指摘した通り、多層防御の
一部である。エイリアスや`use ... as`による間接参照までは検出できず、
単独で「コンパイル時点で到達不能」を証明するものではない。主たる保証は
`AuditRead::entries(&self)`のシグネチャ自体が`ExecutionEngine`/
`AuditWrite`への参照を要求しない設計であること、および型を共有しない
設計そのものにある。この限界はテスト本体のdocコメントにも明記した。

## デビエーション（既知のGAP、Step 6）

1. **NFR3.2（100%パスカバレッジの機械的強制ギャップ）**: 本Unitの異常系
   分岐（パース失敗・フィールド欠落・ファイル不在・ファイル不在以外の
   I/Oエラー）はいずれもテストで個別にカバーしているが、CIの
   `coverage`ジョブは単一のグローバル80%行カバレッジ閾値のみを機械的に
   強制しており、「破壊的操作関連パスは100%パスカバレッジ＋境界値テスト」
   というteam.mdの要求を自動検出する専用のCIゲートは存在しない
   （既存のグローバルGAPであり、本Unitで新規に生じたものではない）。
2. **NFR2.1/NFR2.2（静的テストの限界）**: `audit_reader_module_has_no_execution_or_write_dependency`
   はテキストベースの静的検査であり、エイリアス（例:
   `use crate::execution::ExecutionEngine as X;`）や型の全部パス参照
   （`crate::execution::ExecutionEngine`のようにモジュールパスを介した
   参照で識別子文字列自体を分割する等の回避）までは検出できない。
   主たる保証はコンパイラの型システムそのもの（`AuditRead`実装が対象の
   型を`use`していない設計）であり、この静的テストはコードレビューのみに
   頼らない機械的検証としての多層防御の一部である。

いずれのGAPも承認済みプランに明記されていたものであり、本Unitのスコープ
内で解消可能なものではないため、デビエーションとして記録するにとどめた。

## テストの実行方法

```bash
# 本Unitスコープの単体テストのみ
cargo test --lib audit_reader_tests:: --locked

# lib全体の既存テストを含む回帰確認
cargo test --lib --locked

# フォーマット・リンタ
cargo fmt --check
cargo clippy --lib --tests --locked -- -D warnings
```

すべて green であることを確認済み（`cargo test --lib audit_reader_tests::
--locked` で8件全てpass、`cargo test --lib --locked` で既存を含む129件
全てpass、`cargo fmt --check`・`cargo clippy -- -D warnings`とも警告なし）。
`tests/scan_select_execute.rs`・`tests/audit_log_format.rs`を含む既存の
統合テストは一切変更していない。

## 変更しなかったもの

- `src/audit.rs`内の既存`mod tests`（`AuditLogger`書き込み側のテスト）は
  一切変更していない。
- `Cargo.toml`は変更していない（Step 5: 新規クレート追加なし）。
- `src/cli.rs`・`src/main.rs`（`run_audit`ハンドラの配線）は本Unitの
  スコープ外であり、別Unit（`cli-foundation`）の責務として変更していない。
