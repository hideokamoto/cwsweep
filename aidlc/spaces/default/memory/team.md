# Team-Level Rules

> This team's affirmed practices and corrections. Loaded after `org.md` as
> strict-additive guidance; contradictions with broader policy are rejected.
> Populated by the practices-discovery affirmation gate. Edit at the gate,
> not directly.

## Way of Working

- トランクベース開発を採用する。すべての変更は短命なフィーチャーブランチ経由で `main` にマージする。
- 本プロジェクトは単一開発者体制のため、形式的なプルリクエストの起票・複数人承認は求めない。
  「フィーチャーブランチで作業 → 自己レビュー → CI green → squash-merge」というシンプルな運用とする。
  ただし、マージ前に CI green（`cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo audit` /
  `cargo deny check` / テストスイート）がすべて通過していることを機械的なゲートとして必須とし、
  人的レビューの不在を機械的ゲートで補う。
- Construction ワークツリーのベースブランチは `main`、マージ先も `main` とする。
- Bolt ブランチは squash-merge で `main` に統合する。1 Bolt = 1 コミットとし、delivery-planning の
  Bolt 順序と `main` の履歴を 1:1 対応させる。中間コミットはソースブランチ側にのみ残り、監査ログが
  詳細な変更系列を保持する。

## Walking Skeleton

- 本プロジェクトのスコープファイルは `skeleton: off` を宣言しており、Walking Skeleton は作らない。
  最初の Bolt から通常の機能実装として進める。
- 単一開発者体制かつ破壊的操作（ログ削除・retention変更）を実装する性質上、**M6
  （`--execute` による実削除・実retention変更の実装）に到達する Bolt は毎回ゲート**とし、
  ユーザーの明示承認を経てから次の Bolt に進む。
- Bolt 1 完了後、残りの Bolt をどう進めるか（自律進行 or 毎 Bolt ゲート）は「ladder prompt」で
  決定し、`aidlc-state.md` の `Construction Autonomy Mode` に記録する（M6 到達 Bolt への
  ゲート要件は自律モードを選択した場合でも例外として維持する）。

## Testing Posture

- **Methodology**: custom
- **Ordering**: 基本方針は各層（Identity検証層、ページネーション集計層、対話式UI層、アクション実行層）
  を実装した直後にその層の単体テストを作成・実行する test-after 方式とするが、破壊的操作に直結する
  安全パス（Identity検証・削除直前の二重確認・監査ログ出力・dry-run既定の挙動）だけは、実装に先立って
  期待仕様をテストとして書いてから実装する TDD 的な進め方とする。
- カバレッジ基準:
  - 通常コードは行カバレッジ 80% 以上を floor とする。
  - 破壊的操作（`delete-log-group` / `put-retention-policy`）に関わるコードパス、および
    Identity検証失敗系・`--execute` 未指定時の抑止・対話式マルチセレクトの初期状態は、
    80% floor とは別に **100% パスカバレッジ＋境界値テスト**（不一致ID、リージョン跨ぎ、
    ページネーション途中エラー等）を要求する。
  - カバレッジ計測には `cargo llvm-cov` を使用する。
- テストの種類:
  - AWS SDK 境界（AssumeRole・各種 API 呼び出し）はモックを用いたユニットテストに加え、
    境界を跨ぐ統合的なシナリオテスト（スキャン → 選択 → 削除の一連の流れを複数モジュールを
    通して検証するテスト）を別途用意する。
  - 監査ログの出力内容（フォーマット・必須フィールド：対象アカウントID・リージョン・
    ログループ名・実行時刻・成功/失敗）自体を検証する専用テストを用意する。監査ログ書き込み
    失敗時に該当操作が中断されることを検証するテストも含める。
  - dry-run 既定・全選択禁止などのデフォルト挙動は、フラグを一切指定しない呼び出しを
    固定テストケースとして必須化し、将来の意図しないデフォルト値反転（退行）を検出できるようにする。
- CI では実 AWS アカウントに接触するテストとモックで完結するテストを明示的に分離し
  （例: `#[ignore]` + 専用ジョブ）、通常の CI 実行では前者を自動実行しない。
- 既存スイートは常にグリーンを維持する。

## Change Control

<!-- Affirmed by the team. Mode: strict or relaxed. Strict here holds for every intent and cannot be changed from chat. -->

## Deployment

- 本ツールはサーバーへのデプロイを行わない、スタンドアロンのバイナリ配布物（Rust CLI）である。
- バージョンタグ（例: `v0.1.0`）の push をトリガーに、CI がクロスプラットフォーム
  （Linux/macOS、x86_64/arm64）のリリースバイナリをビルドし、GitHub Releases に添付する
  「タグ駆動リリース」方式とする。`main` へのマージ自体はリリースをトリガーしない。
- crates.io への公開（`cargo install` 経由の配布）は v1 では対象外とする。
- M6（`--execute` による実削除・実retention変更）を含むリリースタグには、テスト用サンドボックス
  AWS アカウント（本番 Organization とは別）に対する動作確認済みチェックリストへのサイン off を
  要求する。チェックリストには、認証情報（AssumeRole で取得した一時クレデンシャル）がログ・
  標準出力・エラーメッセージのいずれにも出力されていないことの確認項目を含める。

## Code Style

- フォーマッタは `rustfmt` のデフォルト設定、リンタは `clippy` を採用する。
- CI で `cargo fmt --check` と `cargo clippy -- -D warnings` を実行し、失敗時は
  マージをブロックする。ただし開発初期の停滞を避けるため、最初の Bolt では重大な警告のみを
  エラー化し、以降の Bolt で段階的に厳格化していく。
- `unwrap()` / `expect()` の濫用と `panic!` を避け、エラーは `Result` で呼び出し元に伝播させる。
  `#[deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` は本番コードパスに適用し、
  テストコードは対象外とする。
- `unsafe` コードは全面禁止する（`#![forbid(unsafe_code)]` をクレートルートに宣言）。
- 依存関係の脆弱性スキャン（`cargo audit`）とライセンス／サプライチェーンチェック
  （`cargo deny check`）を CI 必須ゲートとする。`Cargo.lock` はリポジトリにコミットし、
  CI では `--locked` フラグでビルド・テストする。
- 一時クレデンシャル（AssumeRole で取得したアクセスキー・シークレット・セッショントークン）は
  ログ・標準出力・パニックメッセージのいずれにも一切出力しない。クレデンシャルを保持する構造体は
  `Debug` 実装をマスクするか `secrecy` クレート等でラップすることを検討する。
- 命名規則は Rust 慣習（snake_case、CamelCase 型名など）に従う。ドメイン語彙
  （`account`, `log_group`, `dry_run` 等）は型名・関数名で一貫させ、略語を避ける。
  スキャン時点の Identity 検証と削除直前の二重 Identity 検証は、役割が名前から読み取れるよう
  別関数として命名を分離する。
## Forbidden

<!-- Team-specific forbidden patterns -->

## Mandated

<!-- Team-specific mandates -->

## Corrections

<!-- Self-learning loop appends here. -->
