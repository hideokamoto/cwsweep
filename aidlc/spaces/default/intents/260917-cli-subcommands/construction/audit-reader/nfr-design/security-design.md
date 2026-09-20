# security-design.md — Unit: audit-reader

`security-requirements.md`（`NFR2.1`/`NFR2.2`/`NFR3.1`〜`NFR3.3`）を、
Rust言語における具体的な設計パターンへ落とし込む。本Unitはネットワーク
境界を持たないローカルファイル読み取り専用コンポーネントであるため、
認証・認可・暗号化・シークレット管理・CSRF/XSS対策は適用外である
（詳細は末尾の「適用外」節を参照）。

## NFR2.1/NFR2.2: 構造的分離の実装パターン

### モジュール構成

domain-design（`components.md` Q2）で確定した「同一ファイル
（`src/audit.rs`）内での型レベル分離」方針を踏襲する。

```
src/audit.rs
├── pub enum AuditEventKind { Intent, Result }      // 既存（AuditWrite側）
├── pub struct AuditEntry { ... }                    // 既存（AuditWrite側、書き込み用）
├── pub trait AuditWrite { ... }                      // 既存（書き込み専用）
├── pub struct AuditLogger { ... }                    // 既存（書き込み専用実装）
│
├── pub struct AuditReaderEntry { ... }               // 新設（読み取り専用、entities.md AuditEntry相当）
├── pub struct SkippedLine { ... }                    // 新設
├── pub struct AuditReadOutcome { ... }               // 新設
├── pub enum AuditReadError { Io(std::io::Error) }    // 新設
├── pub trait AuditRead { fn entries(&self) -> Result<AuditReadOutcome, AuditReadError>; } // 新設
└── pub struct AuditReader { path: PathBuf }           // 新設、AuditReadを実装
```

**設計上の注意**: 書き込み側の`AuditEntry`（`AuditWrite`が使う型）と、
読み取り側が返すエントリ型は、名前を分ける（例:
`AuditReaderEntry`、実装時に最終名はCode Generationで確定してよい）。
これは単なる命名規約ではなく、`AuditReader`実装が`AuditWrite`モジュール
の型を`use`する経路自体を作らないための設計上の境界である
（型を共有すると、その型を経由した間接的な依存関係が生まれうる）。

### 依存グラフの制約（設計時点でのルール）

- `AuditReader`・`AuditRead`・`AuditReadOutcome`・`SkippedLine`・
  `AuditReadError`のいずれの定義・実装ブロックも、`ExecutionEngine`
  （`src/execution.rs`）・`AuditWrite`トレイト・`AuditLogger`構造体への
  `use`文または型参照を一切含まない。
- `AuditReader::entries()`のシグネチャは`&self`のみを受け取り、
  `ExecutionEngine`や`AuditWrite`を実装する値への参照を一切要求しない。

### 検証手段（NFR2.1/NFR2.2の実現方法）

`contract-summary.md`が指摘する通り、`forbidden_dependencies`という
YAML宣言自体はコンパイル時点での到達不能性を強制しない。Rustには
モジュール依存グラフをコンパイラレベルで制限する標準機構がないため、
以下の**テキストベースの静的回帰テスト**を構造的検証手段として設計する
（Code Generation/Build and Test段階で実装）:

```rust
// 疑似コード（Code Generation段階で具体化する）
#[test]
fn audit_reader_module_has_no_execution_or_write_dependency() {
    let source = std::fs::read_to_string("src/audit.rs").unwrap();
    let reader_section = extract_span(&source, "AuditReader region markers");
    assert!(!reader_section.contains("ExecutionEngine"));
    assert!(!reader_section.contains("AuditWrite"));
    assert!(!reader_section.contains("AuditLogger"));
}
```

この静的検査は、`AuditReader`関連コードのソーステキスト中に禁止識別子が
一切出現しないことを機械的に確認する。コンパイラの型システムほど
強力ではないが、コードレビューのみに頼らない機械的検証として機能する
（project.md Forbidden: 「コードレビューのみによる担保は許容しない」を
満たす）。加えて、`run_audit`ハンドラ（`cli-foundation`側、次のUnitの
責務）の依存注入シグネチャに`AuditRead`実装以外の型が現れないことを
検証する統合テストを、`cli-foundation`のCode Generationで追加する
（本Unitの範囲外だが、境界の両側で検証することを推奨事項として記録する）。

## NFR3.1〜NFR3.3: データ整合性・読み取り専用性の実装パターン

### 行単位のResult型ベース処理

```rust
// 疑似コード（entities.md/rules.mdの仕様をRustの型に対応させたイメージ）
for (line_number, line) in reader.lines().enumerate() {
    let line = line?; // I/Oエラー（BR2.2）はここで AuditReadError::Io として即座に伝播
    match serde_json::from_str::<AuditReaderEntry>(&line) {
        Ok(entry) => outcome.entries.push(entry),
        Err(e) => {
            outcome.skipped_lines.push(SkippedLine {
                line_number: line_number + 1,
                reason: e.to_string(),
            });
            eprintln!("warning: skipped malformed audit log line {}: {}", line_number + 1, e);
        }
    }
}
```

- `serde_json::from_str`の失敗（構文エラー・型不一致・必須フィールド欠落
  いずれも`serde`の`Deserialize`導出が検出する）を`SkippedLine`へ変換し、
  ループを継続する（NFR3.1）。`unwrap()`/`expect()`/`panic!`は使用しない。
- ファイル自体のI/Oエラー（`reader.lines()`が返す`Err`、権限不足等）は
  即座に`AuditReadError::Io`として呼び出し元へ伝播する（BR2.2）。

### 読み取り専用性の保証（NFR3.3）

- `AuditReader::entries(&self)`は`&self`（不変参照）のみを取り、内部で
  `File::open`（読み取り専用モード、書き込みフラグなし）以外のファイル
  操作を行わない。`OpenOptions::write`/`create`/`append`を一切使用しない。
- ファイル不在判定（`path.exists()`相当）はメタデータ確認のみで行い、
  ファイルを新規作成する副作用を持つAPI（`OpenOptions::create(true)`等）
  を使用しない。

### 100%パスカバレッジの設計上の含意（NFR3.2）

異常系分岐（パース失敗、フィールド欠落、ファイル不在、ファイル不在
以外のI/Oエラー）を、`AuditReader`内で明確に分岐したコードパスとして
実装し、それぞれに対応するテストケースをBuild and Test段階で用意
できるようにする。分岐を暗黙的に畳み込まない（例: パース失敗と
フィールド欠落を同一の`match`アームで区別なく処理してもよいが、
テストはそれぞれの入力パターンを個別に用意する）。

## 適用外（Anti-Requirements、ネットワーク境界・機微データ非該当）

| トピック | 適用状況 | 理由 |
|---|---|---|
| 認証・認可 | 適用外 | プロセスローカルのファイル読み取りのみ。ネットワーク越しのリクエストを一切受けない |
| 暗号化（保管時/転送時） | 適用外 | 監査ログファイル自体の暗号化方針は既存`AuditLogger`の管轄であり、本Unitは変更しない。転送も発生しない |
| 入力検証（Webセキュリティ的な意味） | 部分適用 | SQLインジェクション・XSS等は非該当。ただし「信頼できない可能性のあるローカルファイル入力の検証」という意味でのNFR3.1が対応する |
| セキュリティヘッダー・CORS | 適用外 | HTTPサーバーではない |
| シークレット管理 | 適用外 | 本Unitはクレデンシャルを一切扱わない（`AssumeRole`等は`CredentialProvider`の管轄） |
