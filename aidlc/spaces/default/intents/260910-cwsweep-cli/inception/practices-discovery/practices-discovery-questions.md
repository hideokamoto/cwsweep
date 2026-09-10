# Practices Discovery — Interview

リード（release engineer役）と3名のレビュアー（QA・開発者・DevSecOps）の独立した検討結果を踏まえた質問です。

## Q1. ブランチ運用とマージ方法について

トランクベース開発（短命フィーチャーブランチ→`main`へsquash-merge）で進めますか？単独開発のため、形式的なプルリクエストのレビュー承認は求めず、「ブランチで作業→自己レビュー→CIグリーン→squash-merge」というシンプルな運用でよいですか？

A. はい、その運用で進める
B. 別の運用にしたい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、その運用で進める

## Q2. 最初から動く最小限の縦串（Walking Skeleton）を作りますか？

Walking Skeletonとは、実際の機能を作り込む前に、パーツがつながることを証明するために最初に作る「一気通貫で動く最小版」のことです。本スコープファイルでは`skeleton: off`（作らない）と既に設定されています。この方針のままでよいですか？

A. はい、Walking Skeletonは作らず、最初のBoltから通常の機能実装として進める
B. 作りたい（自由記述で理由を補足する）
X. Other (please specify)

[Answer]: A. はい、Walking Skeletonは作らず、最初のBoltから通常の機能実装として進める

## Q3. テストの進め方（テスト手法）について

実装してからテストを書く「テスト後」方式（test-after）と、先にテストを書く「テスト駆動」方式（TDD）のどちらで進めますか？レビュアーからは、破壊的操作（削除・retention変更）を伴う経路については通常より高いカバレッジと、監査ログ出力自体の検証テスト、dry-run既定などのデフォルト挙動が壊れないための固定テストケース（退行防止テスト）が必要という指摘がありました。

A. 基本はtest-after方式で進めるが、破壊的操作の経路（Identity検証・二重確認・監査ログ出力・dry-run既定）だけは仕様を先にテストとして書いてから実装するTDD的な進め方にする（custom: test-after + 安全パスのみTDD）
B. 全体を通してtest-after方式のみでよい
C. 全体を通してTDD方式で進めたい
X. Other (please specify)

[Answer]: A. 基本はtest-after方式で進めるが、破壊的操作の経路（Identity検証・二重確認・監査ログ出力・dry-run既定）だけは仕様を先にテストとして書いてから実装するTDD的な進め方にする（custom: test-after + 安全パスのみTDD）

## Q4. テストのカバレッジ基準と種類について

以下の基準でよいですか？
- 通常コードは行カバレッジ80%以上
- 破壊的操作（削除・retention変更）に関わるコードパスは100%カバレッジ
- AWS SDKの境界（AssumeRole・API呼び出し）はモックを用いたユニットテストに加えて、境界を跨ぐ統合的なシナリオテスト（複数モジュールを通した一連の流れの検証）も別途用意する
- 監査ログの出力内容（フォーマット・必須フィールド）自体を検証するテストを用意する
- カバレッジ計測には`cargo llvm-cov`を使う

A. はい、この基準でよい
B. 変更したい項目がある（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この基準でよい

## Q5. リリース・配布の方法について

サーバーへのデプロイという概念がない単一バイナリCLIであるため、「マージ時にステージング環境へデプロイ」というorg.mdの既定は当てはまりません。代わりに、バージョンタグ（例: `v0.1.0`）をpushしたときにCIがクロスプラットフォーム（Linux/macOS、x86_64/arm64を想定）のリリースバイナリをビルドし、GitHub Releasesに添付する、という方式でよいですか？`cargo install`向けのcrates.io公開は現時点では対象外とします。

A. はい、その方式でよい
B. 別の方式にしたい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、その方式でよい

## Q6. コードスタイル・Lintの厳格さについて

以下の方針でよいですか？
- フォーマットは`rustfmt`のデフォルト設定に従う
- Lintは`clippy`を使用し、CIでは`-D warnings`（警告をエラー扱い）とする。ただし初期実装時に大量の警告で開発が滞らないよう、最初のBoltでは重大な警告のみをエラー化し、以降のBoltで段階的に厳格化する
- `unwrap()`/`expect()`の濫用と`panic!`を避け、エラーは`Result`で伝播させる（`#[deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`は本番コードパスに適用し、テストコードは対象外とする）
- `unsafe`コードは全面禁止する（`#![forbid(unsafe_code)]`）
- 依存関係の脆弱性スキャン（`cargo audit`）とライセンス/供給網チェック（`cargo deny check`）をCI必須ゲートとする
- 一時クレデンシャル（AssumeRoleで取得したアクセスキー・シークレット・セッショントークン）はログ・標準出力・パニックメッセージに一切出力しない（Debug実装のマスキング、`secrecy`クレート等の利用を検討する）

A. はい、この方針でよい
B. 変更したい項目がある（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この方針でよい

## Q7. 監査ログ書き込みが失敗した場合の挙動について

開発者エージェントから、監査ログの書き込み自体が失敗した場合の挙動が未定義との指摘がありました。監査ログはPRDで「必須出力（無効化オプションなし）」とされていますが、書き込み失敗時にどうしますか？

A. 監査ログの書き込みに失敗した場合は、実行中の削除/retention変更操作自体を中断し、エラーとして扱う（ログが残せない操作は実行しない）
B. 監査ログの書き込みが失敗しても、操作自体は続行し、失敗を標準エラー出力に警告として表示するのみ
X. Other (please specify)

[Answer]: A. 監査ログの書き込みに失敗した場合は、実行中の削除/retention変更操作自体を中断し、エラーとして扱う（ログが残せない操作は実行しない）

---

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
