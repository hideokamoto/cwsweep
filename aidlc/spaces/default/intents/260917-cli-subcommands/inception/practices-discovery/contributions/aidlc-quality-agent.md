**Collaborator:** aidlc-quality-agent

## Contribution

### 1. カバレッジ計測スコープと `main.rs` への漏れリスク（重要な見落とし）

`code-quality-assessment.md` によれば、CI の `coverage` ジョブは
`cargo llvm-cov --lib --locked --fail-under-lines 80 --summary-only` であり、
**`--lib` のみを対象とし、バイナリクレート（`src/main.rs`）はカバレッジ計測の
対象外**である。さらに同ファイルの技術的負債シグナル2は、`main.rs` の
`action_kind_from_prompt` が `inquire` への直接依存ゆえに未テストであることを
既に指摘している。

`code-structure.md` によると、現状のサブコマンド化対象の分岐（`cli.scan_only`
判定 → 非TTYフォールバック → 対話式選択・確認・実行）は `src/main.rs`
`main()` にある。lead の draft（team-practices.md）は「破壊的操作パスは
100%パスカバレッジ＋境界値テスト」「通常コードは80%floor」という基準の
サブコマンドへの読み替えを扱っているが、**この新しいサブコマンド分岐ロジックを
`main.rs`（バイナリクレート）に置いたまま実装した場合、`--lib` スコープの
coverage ジョブはそのロジックを一切計測しない**という、既存 CI 設定と
Testing Posture 基準の間の整合性リスクに触れていない。これは今回のリファクタで
悪化しうる問題であり（分岐点が増える）、次のいずれかを人間インタビューで
明確化する必要がある：
(a) `Commands`（`scan`/`clean`/`audit`）へのディスパッチロジックを `cli.rs`
（`lib` クレート側）に集約し、`main.rs` は薄いアダプタ配線のみに留める設計方針を
Code Style/Way of Working として明示する、または
(b) CI の `coverage` ジョブ自体を `--lib` から `--workspace`（バイナリ含む）に
拡張する。
(a) の方が既存の ports-and-adapters パターン（本番アダプタは `main.rs`、
ロジックは `lib` 側）とも整合的であり、質より軽微な変更で済むと考えられるが、
これは設計判断であるため断定はしない。

### 2. `tests/scan_select_execute.rs` の実際の役割についての訂正提案

lead の draft は「既存の統合テスト `tests/scan_select_execute.rs` はサブコマンド
経由の呼び出し（`scan` → 選択 → `clean --execute`）に更新が必要になる可能性が
高い」としているが、実ファイルを確認したところ、このテストは `cwsweep::cli::Cli`
や `clap` のパース処理には一切触れておらず、`OrgDiscovery`・
`CredentialProvider`・`LogGroupScanner`・`InteractiveSelector`・
`ActionPlanner`・`ConfirmationPresenter`・`ExecutionEngine` といった
ドメイン層コンポーネントを直接手組みで配線して検証する、**CLI 非依存の
パイプラインテスト**である。したがって、サブコマンド化そのものによって
このテストの中身を書き換える必要は本来ない（`Cli`/`Commands` の型を経由しない
ため、`--scan-only`/`--execute` フラグとサブコマンドの入れ替えの影響を
受けない）。

真のギャップは「既存テストの更新」ではなく、**`Cli`/`Commands` のパースと
`main()` のサブコマンドディスパッチそのものを検証する統合テストが現状
存在しない**ことである。現在の統合テストはいずれもドメイン層のみを対象とし、
CLI 引数解析からエンドツーエンドで検証するテストが `tests/` 配下に無い
（`code-structure.md` のテスト配置パターン参照）。draft の
「サブコマンド一切指定なしの場合の振る舞い」「`clap` の `Cli`/`Commands`
構造そのものに対するCLI引数パーステスト」という追加観点自体は正しい方向だが、
それは「既存テストの更新」ではなく「新規の CLI 層統合テストの新設」として
明確に位置づけるべきである。加えて、パース検証は `assert_cmd` 等によるプロセス
起動よりも、`Command::try_parse_from` / `Cli::try_parse_from` を使った
インプロセスの単体テスト（`cli.rs` 内 `#[cfg(test)]`）の方が高速かつ
`--lib` カバレッジにも算入される点を、実装方針として人間インタビューで
確認することを推奨する。

### 3. 構造的な安全境界（`scan`/`audit` から破壊的操作を呼べない）へのテスト要求が未定義

discovered-rules.md の「本 intent での追加候補（Forbidden）」は、破壊的操作を
`clean` サブコマンドの経路のみに構造的に限定する案を挙げているが、
team-practices.md の Testing Posture 側には、この構造的保証を検証する
テスト要求（例：`scan`/`audit` のハンドラ関数のシグネチャや型が
`delete_log_group`/`put_retention_policy` を実行しうる `ExecutionEngine`
到達不能であることを保証する設計、あるいは最低限のコンパイル時/実行時
回帰テスト）が明記されていない。フラグベースの `conflicts_with` は
既存の `--scan-only`/`--execute` 相互排他をランタイムテストで直接検証
できたが、型ベースの構造的保証は「テストで壊れないことを示す」以上に
「型で不可能にする」設計であるため、対応するテストは「`scan`/`audit`
ハンドラのモジュールが `ExecutionEngine`/`ActionApiOperations` の削除系
メソッドを一切参照しないこと」を確認する回帰テスト（コンパイル成功を
前提とした構造テスト、または `cargo modules`/静的解析的アプローチ）に
なりうる。この具体的なテスト方法は人間インタビューで確認すべき。

### 4. `audit` サブコマンドの異常系テストの具体化不足

draft は「監査ログの読み取り（パース）ロジックに対する不正フォーマット・
欠落フィールド等の異常系テスト」を要求しているが、`code-quality-assessment.md`
の技術的負債シグナル3（監査ログの読み取り API が現状存在しない）を踏まえると、
新設される読み取り API（`AuditLogger`/`AuditWrite` への読み取り機能追加）
自体の設計がまだ固まっていない段階でテスト要求だけを先に書いている状態である。
既存の `tests/audit_log_format.rs` は書き込み側のフォーマット検証のみであり、
読み取り側のテストパターン（1行ずつストリーミングパースするのか、全件
メモリロードするのか、破損した1行があった場合に全体を失敗させるのか
該当行のみスキップするのか）は Testing Posture 側で方針化されていない。
これは domain-design/contract-design 段階で解決される設計事項かもしれないが、
practices-discovery としては「壊れた1行がある場合の挙動（fail-fast か
skip-and-warn か）」を Forbidden/Mandated 相当の判断が必要になりうる点を
インタビューの未解決事項に追加することを推奨する。

## Positions

OBJECT: `tests/scan_select_execute.rs` は `Cli`/`clap` パースを経由しない
ドメイン層専用の統合テストであるため、「サブコマンド経由の呼び出しへの更新が
必要になる可能性が高い」という lead の記述は不正確。真のギャップは
「既存テストの更新」ではなく「CLI引数パース〜ディスパッチを検証する
統合テストの新設」であり、team-practices.md 該当箇所の表現を訂正すべき。

OBJECT: lead の draft には、CI の `coverage` ジョブが `--lib` スコープのみで
`main.rs`（バイナリクレート）を計測対象外としている既存の事実、およびそれが
サブコマンドディスパッチロジックの配置場所次第で新しいロジックを
80%/100%floor の外側に置いてしまうリスクへの言及が欠けている。人間
インタビューで「ディスパッチロジックを `lib` 側に集約する」か「coverage
ジョブのスコープを拡張する」かを確認すべき事項として追加する必要がある。

AGREE: 100%パスカバレッジ＋境界値テストの floor を `clean` サブコマンドの
実行系経路に限定し、`scan`/`audit`（読み取り専用）を対象外として80%floorのみ
適用するという lead のスコープ再定義は、既存 project.md の Mandated/Forbidden
（破壊的操作に関する二重Identity検証・監査ログ必須等）の意図と整合しており
妥当。

AGREE: `audit` サブコマンドの読み取りロジックに異常系テスト（不正フォーマット・
欠落フィールド）を要求する方向性自体は正しいが、読み取り API の設計
（ストリーミング/全件ロード、破損時の fail-fast/skip 方針）が固まっていない
段階での具体化には限界があり、この点は人間インタビューの未解決事項として
追加すべき（上記 Contribution 4 参照）。
