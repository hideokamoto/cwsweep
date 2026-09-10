**Collaborator:** aidlc-devsecops-agent

## Contribution

セキュリティ観点から、リード案（`team-practices.md` / `discovered-rules.md`）に対して以下を提案する。本プロジェクトは単一開発者の Rust CLI でありながら、Organization 全体にまたがる `AssumeRole` とログの破壊的削除（`delete-log-group` / `put-retention-policy`）を実行するため、「レビュー人数が足りない分をツールで補う」という原則で設計する。

### 1. Lint / Format — SAST 系静的解析

- `rustfmt --check` と `cargo clippy -- -D warnings` は Code Style 案どおり CI 必須とする。**単一開発者体制では人間の二次レビューが存在しないため、`clippy` の警告レベルは全 lint グループを対象に厳格化することを推奨する**：`clippy::all` に加え `clippy::pedantic` と `clippy::nursery` の少なくとも一部（誤検知の多いものは個別 allow）、さらに **`clippy::unwrap_used` / `clippy::expect_used` / `clippy::panic` を deny** することを提案する。理由: 本ツールは Organization 全アカウント×全リージョンを横断してループ処理するため、1 アカウントでの `unwrap()` パニックが実行途中で全体を中断させたり、想定外の状態（部分実行）を残すリスクがある。エラーは常に `Result` で呼び出し元に伝播させる方針（construction.md のエラーハンドリング原則とも整合）。
- `unsafe` コードは原則禁止（`#![forbid(unsafe_code)]` をクレートルートに明示）。AWS SDK for Rust (aws-sdk-*, aws-config) 呼び出しのみで完結する設計であれば `unsafe` を必要とする理由がない。もし将来的にネイティブ依存が必要になった場合は、その箇所だけ例外化し理由をコメントで明記する。
- ログ出力コードにおける機密情報混入を防ぐため、`clippy` だけでは検出できない「認証情報のログ出力」を防ぐレビュー観点を Code Style に明記する（後述）。

### 2. 依存関係の脆弱性スキャン / サプライチェーン

- `cargo audit`（RustSec Advisory DB 照合）を CI に必須で組み込み、既知脆弱性のある依存クレートがあればマージをブロックする。
- `cargo deny check`（advisories / licenses / bans / sources）も併用することを提案する。特に `sources`（crates.io 以外の git 依存の禁止または明示許可）と `bans`（重複依存・意図しない multiple-versions の検出）は、本ツールが AWS 認証情報を扱う以上、依存クレートの出自を厳格に管理する意味で重要。
- `Cargo.lock` は必ずリポジトリにコミットし、CI では `--locked` フラグ（`cargo build --locked` / `cargo test --locked`）でビルドして、ローカルとCIで解決される依存バージョンの差異（＝意図しない依存更新の混入）を防ぐ。
- 依存クレートの更新は Dependabot（または `cargo outdated` + 手動レビュー）で定期的に検知するが、**単一開発者かつ破壊的操作を行うツールという性質上、依存更新PRも `cargo audit` + 既存テストスイートのグリーンを機械的ゲートとして必須にする**（人間の目視レビューに頼らない）。
- AWS SDK for Rust のメジャーバージョン更新（`aws-sdk-cloudwatchlogs`, `aws-sdk-organizations`, `aws-sdk-sts`, `aws-config` 等）は破壊的操作に直結するため、更新時に M6 相当のセーフティ機構（Identity検証、ページネーション全件取得等）の既存テストが全てグリーンであることを昇格条件とする。

### 3. シークレット漏洩防止（本ツール固有の最重要事項）

スコープ文書が要求する「監査ログの必須出力」と「実行前確認画面」は、実装を誤ると **AWS認証情報やSTSトークンをログ・標準出力に書き出してしまうリスク**と表裏一体である。以下を Mandated 相当として追加提案する。

- **NEVER**: 監査ログ・標準出力・エラーメッセージのいずれにも、`AssumeRole` で取得した一時クレデンシャル（`AccessKeyId` / `SecretAccessKey` / `SessionToken`）を出力しない。ログに含めてよいのは「対象アカウントID・リージョン・ログループ名・実行時刻・成功/失敗」（スコープ文書が明記する項目）のみであり、クレデンシャル構造体全体を `{:?}` でダンプするような実装を禁止する。
- 認証情報を保持する構造体（`Credentials` 等)は `Debug` の手動実装でマスクするか、`secrecy` クレート等の「意図せずログに出せない型」でラップすることを推奨する。CI に `clippy::dbg_macro` を deny として追加し、デバッグ用の `dbg!()` マクロの残留を機械的に防ぐ。
- リポジトリ内のシークレット混入防止として、`gitleaks` または `detect-secrets` を pre-commit / CI に組み込むことを提案する（AWSアカウントID・アクセスキー形式のパターン検出）。AWSアカウントIDは機密情報ではないが監査ログ上は正規の出力対象であるため、誤検知が出た場合はallowlist運用とする。
- CI/CD がAWS認証情報を使う場合（後述のIaCスキャンや将来のE2Eテストで実アカウントに接続する場合）、長期的なIAMアクセスキーをCIシークレットとして保存するのではなく、GitHub Actions OIDC + AssumeRole方式を採用し、CI自体の認証情報露出面を減らすことを推奨する。

### 4. DAST / 動的検証

本ツールはWebアプリケーションではないため古典的なDASTは適用外。代わりに以下を提案する。

- **統合テスト（実AWS環境に対する動作確認）**は、破壊専用の使い捨てサンドボックスOrganizationアカウント（本番Organizationとは別）に対してのみ実行し、CI変数で本番アカウントIDへの誤接続を機械的に防止する（例: テストコード側で許可済みアカウントIDのallowlistを持ち、一致しなければテスト自体を失敗させる — スコープ文書のIdentity検証原則をテストコードにも適用）。
- ローカルモック（`aws-smithy-mocks` やLocalStackなど）によるユニット/結合テストを基本とし、実Organizationへの接続を伴うテストは明示的にタグ分け（例: `#[ignore]` + 専用CIジョブ）して、通常のCI実行では走らないようにする。誤って実AWS環境に対して`--execute`相当のテストが自動実行されることを防ぐ。

### 5. IaC / パイプラインセキュリティ

- 本v1スコープはCLI配布のみでインフラのプロビジョニングを含まないため、cfn-lint/cfn-nag/Checkov相当のIaCスキャンは現時点では対象外と判断する。ただし、将来的にCI/CD自体のインフラ（GitHub Actions runner用のIAMロール、OIDCプロバイダ設定など）をIaC化する場合は、その時点でCheckov等の適用を再検討する。
- CI/CDパイプライン自体のセキュリティとして、GitHub Actionsを使う前提であれば `permissions:` を最小権限（デフォルト`read-all`を明示的に絞る）に設定し、リリースジョブ（GitHub Releasesへのアップロード）のみ `contents: write` を付与するなど、ジョブ単位でスコープを分離することを提案する。
- サードパーティGitHub Action は主要なもの（`actions/checkout` 等公式以外）はコミットSHA固定（タグではなくSHA pin）を推奨する。サプライチェーン攻撃（tag再利用による悪意コード混入）対策。

### 6. Mandated/Forbidden への追加提案（`discovered-rules.md` への追記提案）

- ALWAYS: 一時認証情報（AssumeRoleで取得したCredentials）は、ログ・標準出力・エラーメッセージ・パニックメッセージのいずれにも出力しない。
- ALWAYS: 依存クレートの脆弱性スキャン（`cargo audit`）と `Cargo.lock` の固定ビルド（`--locked`）をCIで実行し、失敗時はマージをブロックする。
- NEVER: `unsafe` コードブロックを使用しない（クレートルートで `#![forbid(unsafe_code)]` を宣言）。
- NEVER: `unwrap()` / `expect()` / `panic!` をライブラリ層（AWS API呼び出し・Identity検証・削除実行のパス）で使用しない。呼び出し元に `Result` で伝播させる。

## Positions

AGREE: Way of Working — トランクベース開発、単一開発者のため自己レビュー＋CI green、Bolt squash-merge。単一開発者体制での妥当な簡素化であり、監査ログが詳細な変更系列を保持する点でトレーサビリティも確保されている。

AGREE: Walking Skeleton — M1相当（Organizations一覧取得→AssumeRole→Identity検証）を骨格とする方針、およびM6到達Boltへの毎回ゲート推奨。破壊的操作を含むBoltへの追加ゲートはセキュリティ上も強く支持する。

AGREE: Testing Posture — test-after、層ごとの実装直後テスト、80%カバレッジfloor。

AGREE: Testing Posture の安全性追加提案（Identity検証失敗系・`--execute`未指定時抑止・マルチセレクト初期状態への100%パスカバレッジ＋境界値テスト） — 構造的事故防止のコアパスであり、通常floorとは別扱いにする判断は妥当。TDDの部分適用（安全クリティカルパス先行）についても、少なくともIdentity検証と`--execute`ガードの2箇所は先にテストを書く価値があると考える。

AGREE: Deployment — タグ駆動リリース方式、`main`マージ＝リリーストリガーとしない点、M6リリースタグへの実Org動作確認チェックリスト要求。実Org確認チェックリストには、認証情報がログに出力されていないことの確認項目を追加することを提案する。

AGREE: Code Style — `rustfmt`/`clippy`採用、CI必須化、Rust慣習の命名規則。

OBJECT: Code Style の「要インタビュー」扱いの `clippy` 警告レベルと `unsafe` 使用可否 — インタビュー確認事項とすること自体には反対しないが、セキュリティ上のデフォルト方針としては「厳格側」（`unsafe` 原則禁止、`unwrap`/`expect`/`panic` をdeny、`-D warnings`全面適用）を強く推奨したい。単一開発者でレビュー多重化が効かない以上、機械的ゲートを緩める理由がない。インタビューでは「どこまで厳格化するか」ではなく「厳格化した上で個別に allow する例外があるか」という向きで確認することを提案する。

OBJECT: 現在の `discovered-rules.md` の Mandated/Forbidden に、認証情報の非ログ出力ルールと依存脆弱性スキャンの必須化が含まれていない点 — スコープ文書自身が「破壊的操作の構造的防御」を最重要事項としているにもかかわらず、監査ログ要件（有効化必須）と認証情報の非出力要件がセットで明記されていない。これは実装時に「監査ログを詳細にすればするほど認証情報も一緒に出力してしまう」という事故を誘発しかねないため、上記「Mandated/Forbidden への追加提案」の2項目（認証情報非出力、依存脆弱性スキャン必須）を正式にMandatedへ追加することを求める。

AGREE: Way of Working の「要インタビュー」扱い（PR起票の要否） — セキュリティ上はPRの有無より「マージ前にCI green（fmt/clippy/audit/testが全て通過）」が機械的に強制されていることの方が重要であり、この点は既にリード案でカバーされている。
