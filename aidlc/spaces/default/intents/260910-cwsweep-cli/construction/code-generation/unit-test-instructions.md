# Unit Test Instructions — cwsweep (zero-Unit / stage-level)

## テストフレームワーク・設定

- 標準の`cargo test`をユニットテストランナーとして使用する。各モジュール内に`#[cfg(test)] mod tests { ... }`でユニットテストを配置する。
- カバレッジ計測は`cargo llvm-cov`を使用する。ローカル実行:
  ```bash
  cargo llvm-cov --lib --summary-only
  ```
- 統合テストは`tests/`直下に配置し、実AWSアカウントへ接触するテストには`#[ignore]`属性を付与してCI通常ジョブから分離する。

## 本ユニット（stage-level, 単一実装）を実行する厳密スコープコマンド

- ライブラリ内ユニットテストのみ（本ステージが生成する全モジュール）:
  ```bash
  cargo test --lib
  ```
- 統合テスト（モックのみ、`#[ignore]`を除く）:
  ```bash
  cargo test --test scan_select_execute --test audit_log_format
  ```
- 破壊的操作パス・安全パスの100%パスカバレッジ確認（該当モジュールに限定）:
  `cargo llvm-cov ... -- <filter>`の`--`以降は`cargo test`と同様にテストバイナリへの
  実行フィルタであり、カバレッジ計測対象を特定モジュールに絞るものではない
  （`-- identity::tests execution::tests audit::tests selector::tests`のように書いても、
  そのフィルタに一致するテストだけが実行された上で、結果は結局クレート全体のsummaryとして
  出力されるだけであり、モジュール単位でカバレッジを分離計測できているわけではない）。
  `cargo llvm-cov`にはモジュール単位でレポートを絞り込むオプションは存在しないため、
  正しい手順は以下のとおり:
  1. クレート全体のカバレッジを計測する（テストフィルタは付けない。各安全制御のテストは
     他モジュールのテストとも整合性を取って実行される必要があるため）:
     ```bash
     cargo llvm-cov --lib --summary-only
     ```
  2. `--summary-only`はソースファイルごとの内訳を1行ずつ出力する（`Filename`列にファイル名、
     `Lines`/`Missed Lines`/`Cover`列に当該ファイルの行カバレッジ）。出力の中から対象4ファイルの
     行を確認する: `identity.rs`（Identity検証）・`execution.rs`（破壊的操作パス・
     `--execute`未指定時の抑止）・`audit.rs`（監査ログ）・`selector.rs`（マルチセレクト初期状態）。
  3. 特定モジュールのテストだけを実行して素早く確認したい場合は、`cargo llvm-cov`ではなく
     `cargo test`側のテストフィルタを使う（カバレッジ計測は行われない、実行確認のみ）:
     ```bash
     cargo test --lib identity::
     cargo test --lib execution::
     cargo test --lib audit::
     cargo test --lib selector::
     ```
- 実AWSアカウントに接触するテスト（`#[ignore]`）は本ステージのCI必須ゲートに含めない:
  ```bash
  cargo test --test '*' -- --ignored
  ```

上記はいずれも本stageが生成するコード（cwsweepクレート全体）にスコープされたコマンドであり、プロジェクト全体を跨ぐ他クレートは存在しない（本リポジトリは単一クレート構成）。

## 期待カバレッジ目標

- 通常コード: 行カバレッジ80%以上（`cargo llvm-cov --lib --summary-only`で計測）。
- 破壊的操作（`delete-log-group` / `put-retention-policy`）に関わるコードパス、Identity検証失敗系、`--execute`未指定時の抑止、対話式マルチセレクトの初期状態: 100%パスカバレッジ＋境界値テスト（不一致ID、リージョン跨ぎ、ページネーション途中エラー等）。

## モック/スタブ方針

- AWS SDK境界（`aws-sdk-organizations`, `aws-sdk-sts`, `aws-sdk-cloudwatchlogs`）は、各コンポーネントのトレイト境界（例: `OrganizationsClientTrait`, `StsClientTrait`, `CloudWatchLogsClientTrait`）を切り、テストでは手書きのモック実装またはクロージャベースのスタブを注入する。
- `secrecy::SecretString`でラップした認証情報はテスト内でも`.expose_secret()`を最小限の箇所でのみ使用し、アサーション対象はマスクされた`Debug`出力の文字列一致（`[REDACTED]`が含まれること、実値が含まれないこと）とする。
- ファイルI/O（`AuditLogger`）はテスト用一時ディレクトリ（`tempfile`クレート、または`std::env::temp_dir()`配下の一意パス）に対して行い、書き込み失敗ケースは読み取り専用パスや不正なパスを与えることでシミュレートする。

## テストデータ管理

- アカウントID・リージョン名・ログループ名はテスト内定数として明示し、本物のAWSアカウントIDを使用しない（例: `111111111111`, `222222222222`のようなダミー値）。
- ページネーション複数ページのフィクスチャは各テスト関数内でインラインに構築する（共有ミュータブル状態を避ける）。

## テスト量（Standard戦略）

- 各コンポーネント（12モジュール）につき5〜8件のユニットテスト。
- 主要境界（スキャン→選択→削除の統合フロー、監査ログ出力）に統合テストスタブを用意する。
- 本スコープ（`poc`/`refactor`/`workshop`ではない一般スコープ）は選択済み戦略（Standard）に加えて追加の新規テストフロアを課さない。既存スイートは常にグリーンを維持する。

## Assumptions & Open Questions

None.
