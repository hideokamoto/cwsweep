# エビデンス（確定版）

## 前提

本プロジェクト（cwsweep）は greenfield であり、参照可能な既存コード・既存 git 履歴・過去の
Construction 成果物は存在しない。そのため、根拠は以下の情報源に基づく：
リードの初期ドラフト（`org.md` デフォルト ＋ 承認済み PRD/スコープ文書）、3名の支援エージェントに
よるブラインドレビュー（各エージェントは互いの内容を見ずに独立して検討）、そして人間インタビュー
（全7問）である。

## リード（pipeline-deploy-agent）ドラフトの根拠ソース

1. **`aidlc/spaces/default/memory/org.md`** — フレームワークのデフォルトプラクティス
   （トランクベース開発・squash-merge、test-after + 80%カバレッジfloor、マージ時ステージング
   デプロイ + 本番手動承認、言語標準ツールへの委任）。既存の `team.md` 内容が存在しないため、
   これらは提案されたデフォルトとして扱った。
2. **承認済み PRD/スコープ文書**（`../../ideation/scope-definition/scope-document.md`）—
   Organization横断のAssumeRole構成、Identity検証要件（M1）、全ページ取得のページネーション
   集計（M2）、dry-runデフォルト・`--execute`明示・二重Identity検証・監査ログ必須出力という
   破壊的操作への安全設計、スタンドアロンバイナリとしての配布形態、リスク優先のシーケンシング方針。

## 3名の支援エージェントが検証・指摘した内容

- **QAエージェント**（`contributions/aidlc-quality-agent.md`）: ドラフトのテスト戦略を
  「安全制御が実装として存在し、壊れたら必ず検出されるか」という基準で評価。
  (1) AWS SDK境界を跨ぐ統合/シナリオテストの位置づけが明記されていない点、
  (2) デフォルト値（dry-run既定等）の退行を検出する固定テストケース規約がない点、
  (3) 監査ログの出力内容自体を検証するテストへの言及がない点、
  (4) CI上でモックテストと実AWS接触テストを分離する規約がない点、
  (5) カバレッジ計測ツールが未指定である点を指摘。いずれも本ドラフトの Testing Posture に反映した。
- **開発者エージェント**（`contributions/aidlc-developer-agent.md`）: モジュール分割案
  （`identity` / `discovery` / `ui` / `actions` / `audit`）と、型レベルで dry-run 抜けを防ぐ
  設計（検証済みであることを表す newtype 経由でしか `actions` を呼べない設計）を提案。
  監査ログ書き込み失敗時の挙動（fail-fast か警告継続か）がスコープ文書に未定義であることを
  指摘し、インタビュー Q7 として確認事項に追加した。また `clippy -D warnings` の初回全面適用に
  条件付きで反対し、段階的厳格化を提案（インタビュー Q6 に反映）。
- **DevSecOpsエージェント**（`contributions/aidlc-devsecops-agent.md`）: 単一開発者でレビュー
  多重化が効かない分を機械的ゲートで補うという原則のもと、
  (1) 一時クレデンシャルの非ログ出力ルール（`discovered-rules.md` の Mandated/Forbidden に
  欠落していた）、
  (2) `cargo audit` / `cargo deny check` による依存脆弱性スキャンの必須化（同じく欠落していた）
  を Mandated へ追加するよう要求。両方とも本ドラフトの discovered-rules.md に統合した。
  また `clippy` 警告レベル・`unsafe` 方針について「厳格側をデフォルトにすべき」という方向性を
  示し、インタビュー Q6 の設問設計（厳格な方針を初期案として提示し確認を取る形）に反映した。

## 人間インタビュー（全7問、回答はすべて A）で確定した事項

- Q1: トランクベース開発、単一開発者のため自己レビュー＋CI green＋squash-merge運用を確定。
- Q2: `skeleton: off` のまま、Walking Skeleton を作らず最初のBoltから通常実装を確定。
- Q3: テスト手法は custom（基本 test-after、破壊的操作の安全パスのみ先行TDD）を確定。
  QAエージェントが指摘した「安全制御の壊れやすさ」への対応として、開発者・DevSecOpsエージェントの
  賛同も得た方向性。
- Q4: 通常コード80%カバレッジ、破壊的操作パス100%カバレッジ、AWS SDK境界の統合/シナリオテスト、
  監査ログ出力内容検証テスト、`cargo llvm-cov` の採用を確定。QAエージェントが指摘した(1)〜(3)の
  ギャップをすべて埋める内容。
- Q5: タグ駆動リリース（GitHub Releasesへのクロスプラットフォームバイナリ添付）、crates.io公開は
  v1対象外を確定。
- Q6: `rustfmt`デフォルト、`clippy -D warnings`（初回Boltは重大警告のみ、段階的に厳格化）、
  `unwrap`/`expect`/`panic`のdeny（本番コードのみ）、`unsafe`全面禁止、`cargo audit`/
  `cargo deny check`必須、一時クレデンシャルの非ログ出力（`secrecy`クレート等の検討）を確定。
  開発者エージェントの「段階的厳格化」提案とDevSecOpsエージェントの「厳格な安全項目」提案の
  両方を統合した形で確定。
- Q7: 監査ログ書き込み失敗時は、実行中の削除/retention変更操作自体を中断することを確定。
  開発者エージェントが指摘した未定義事項への回答であり、`discovered-rules.md` の Mandated に
  新規追加した。

## 適用しなかった/保留したもの

- 既存コードベースのスキャン、既存のCI設定ファイル、既存の `Cargo.toml` 等の実成果物は
  greenfield のため存在せず、参照していない。
- QAエージェントが提案した「テスト用サンドボックスAWSアカウントに対する最小限の実環境スモーク
  テスト（CI組み込み）」および DevSecOps エージェントが提案した「実Organizationへの接続を伴う
  テストの明示的タグ分け（`#[ignore]` + 専用CIジョブ）」は、team-practices.md の Testing Posture
  内に「CIでは実AWSアカウントに接触するテストとモックで完結するテストを明示的に分離し、通常のCI
  実行では前者を自動実行しない」という方針として反映した。ただし、そのサンドボックスアカウントの
  具体的な調達方法・専用CIジョブの構成詳細は、v1のインタビュー範囲を超えるため未確定のまま残す
  （Construction フェーズの CI Pipeline / Infrastructure Design ステージで具体化する）。
- DevSecOpsエージェントが提案した `gitleaks` / `detect-secrets` によるリポジトリ内シークレット
  混入防止、GitHub Actions OIDC + AssumeRole によるCI認証情報の露出面削減、サードパーティ
  GitHub ActionのSHA固定は、インタビューの設問には含めなかった。実装レベルの詳細としてCI
  Pipeline ステージで検討することとし、未決事項として残す。

## 結論

本文書の内容は practices-discovery ステージの最終確定版（affirmed baseline）である。
上記「適用しなかった/保留したもの」の3点（サンドボックスアカウント調達方法、専用CIジョブ構成、
リポジトリシークレット対策の詳細）を除き、テスト手法・カバレッジ基準・リリース方式・コード
スタイル・セキュリティゲート・監査ログ失敗時挙動について未解決の不確実性は残っていない。
