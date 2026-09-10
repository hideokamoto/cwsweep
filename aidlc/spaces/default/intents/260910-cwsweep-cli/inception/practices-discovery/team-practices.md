# チームプラクティス（ドラフト）

> **ステータス: ドラフト（未確定）** — この文書は practices-discovery のインタビュー実施前に、
> `org.md` のデフォルトと承認済み PRD/スコープ文書（`../../ideation/scope-definition/scope-document.md`）
> から機械的に導出した提案です。`team.md` への反映（affirmation）はインタビュー結果を踏まえて行います。

## Way of Working

- トランクベース開発を採用する。すべての変更は短命なフィーチャーブランチ経由で `main` にマージする（org.md のデフォルトを踏襲）。
- 本プロジェクトは単一開発者（single-developer）体制のため、PR レビューは自己レビュー＋CI green を必須条件とする（複数人レビューは前提としない）。
- Construction ワークツリーのベースブランチは `main`、マージ先も `main` とする。
- Bolt ブランチは squash-merge で `main` に統合する。1 Bolt = 1 コミットとし、delivery-planning の Bolt 順序と `main` の履歴を 1:1 対応させる。中間コミットはソースブランチ側にのみ残り、監査ログ（audit log）が詳細な変更系列を保持する。
- **[要インタビュー]** 単一開発者プロジェクトのため、PR という儀式自体を省略し「フィーチャーブランチ→自己レビュー→CI green→squash-merge」で運用するか、それとも形式的にでも PR を起票してレビュー記録を残すかは未確定。

## Walking Skeleton

- 本プロジェクトは greenfield の Rust CLI であり、スコープファイルが `skeleton: on` を宣言する場合のみ Walking Skeleton Bolt（Bolt 1）を実施する。実施する場合、Bolt 1 は単独・ゲート付きで進め、ユーザーが明示承認してから残りの Bolt を進行する。
- Walking Skeleton の候補内容（ドラフト）: M1 相当（Organizations アカウント一覧取得 → AssumeRole → `sts:get-caller-identity` による Identity 検証）を最小疎通させる骨格。スコープ文書のシーケンシング方針（構造的事故の解消を最優先）と整合する。
- Bolt 1 完了後、残りの Bolt をどう進めるか（自律進行 or 毎 Bolt ゲート）は「ladder prompt」で決定し、`aidlc-state.md` の `Construction Autonomy Mode` に記録する。単一開発者かつ破壊的操作（ログ削除）を含むツールのため、**M6（`--execute` 実装）に到達する Bolt だけは毎回ゲートを推奨**する。
- **[要インタビュー]** `skeleton: on` / `off` の選択、および M6 到達時の追加ゲート方針の最終確認。

## Testing Posture

- **Methodology**: test-after
- **Ordering**: 各層（Identity検証層、ページネーション集計層、対話式UI層、アクション実行層）を実装した直後に、その層の単体テストを作成し実行する。層をまたいで実装を先行させない。
- 適用スコープは `mvp` / `enterprise` / `feature` / `infra` / `classic` 相当として扱い、80% 行カバレッジの floor と、マージ前の CI 実行を要求する（org.md デフォルト）。
- **[安全性に関する追加提案 — 要インタビュー]** 本ツールは Organization 全体を横断して `delete-log-group` / `put-retention-policy` という破壊的操作を実行する。スコープ文書は以下の構造的防御をすでに要件化している:
  - API 呼び出し直前の `sts:get-caller-identity` による実アカウントID検証（不一致時は即失敗）
  - 削除直前の二重 Identity 検証（スキャン時点／実行時点）
  - dry-run をデフォルトとし `--execute` 明示時のみ破壊的APIを許可
  - 対話式マルチセレクトの初期状態は全チェックOFF
  これらの構造的防御に対応するコードパス（Identity 検証の失敗系、`--execute` 未指定時の抑止、マルチセレクト初期状態）は、80% floor とは別に **100% パスカバレッジ＋境界値テスト（不一致ID、リージョン跨ぎ、ページネーション途中エラー等）を追加要件として提案**する。TDD の部分適用（この安全クリティカルパスのみ先にテストを書く）を検討する余地があるかはインタビューで確認する。
- 既存スイートは常にグリーンを維持する。

## Deployment

- 本ツールはサーバーへのデプロイを行わない、スタンドアロンのバイナリ配布物（Rust CLI）である。org.md の「マージ時にステージングへデプロイ」は適用不可のため、以下を代替提案する:
  - タグ付け（例: `vX.Y.Z`）またはバージョン番号のバンプをトリガーに、CI（GitHub Actions 等）でクロスプラットフォームのリリースバイナリ（Linux/macOS、x86_64/arm64 を想定）をビルドし、GitHub Releases にアップロードする「タグ駆動リリース」方式とする。
  - `main` へのマージ自体はリリースをトリガーしない（マージ＝リリース候補が確定するだけで、実配布はタグ起点）。
  - 本番相当の「実行環境」という概念がないため、環境別デプロイ承認ゲート（tech lead + product owner sign-off）は適用外とする。代わりに、**M6（`--execute` による実削除実装）を含むリリースタグには、実 Organization 環境での動作確認済みチェックリストへのサイン off を要求**することを提案する。
- **[要インタビュー]** リリース物の具体的な配布チャネル（GitHub Releases のみか、`cargo install` 経由の公開も視野に入れるか）、バージョニング規則（SemVer 準拠を明示するか）、クロスコンパイル対象OS/アーキテクチャの確定。

## Code Style

- Rust の言語標準ツールチェーンに従う: フォーマッタは `rustfmt`、リンタは `clippy` を採用する（org.md の「言語デフォルトに委ねる」方針に合致）。
- CI で `cargo fmt --check` と `cargo clippy -- -D warnings` を実行し、失敗時は PR/マージをブロックする。
- 命名規則は Rust 慣習（snake_case、CamelCase 型名など）に従い、プロジェクト独自のリネームルールは設けない。
- **[要インタビュー]** `clippy` の警告レベル（`-D warnings` で全警告をエラー化するか、一部 allow するか）、`unsafe` コードの使用可否方針（AWS SDK 呼び出しのみで完結する想定のため原則禁止を提案）。
