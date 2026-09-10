# NFR Requirements — Questions

requirements-analysisとfunctional-designはスコープ上SKIPされているため、PRD・Practices Discoveryでの確定事項・Domain Designを根拠に、各NFRカテゴリの数値目標を提案します。

## Q1. パフォーマンス目標について

Organization横断のスキャン（全アカウント×全リージョン）は、AWSの各APIレート制限に依存するため厳密な応答時間SLAは設定しにくいですが、以下の目安でよいですか？
- 10アカウント×2リージョン程度の標準的な規模で、スキャン完了まで実用上2分以内を目安とする（ハードな契約ではなく設計目標）
- ページネーションは並列化せず、AWS APIのスロットリングを尊重した逐次呼び出しとする（誤ってレート制限に抵触し他ツールに影響を与えないため）

A. はい、この目安でよい
B. 変更したい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この目安でよい

## Q2. セキュリティ要件について

以下の要件でよいですか？
- 一時クレデンシャル（AssumeRoleで取得したアクセスキー・シークレット・セッショントークン）はログ・標準出力・パニックメッセージに一切出力しない（Practices Discoveryで確定済み）
- 認証・認可はAWS IAMに完全に委譲し、本ツール独自のユーザー認証機構は持たない
- API呼び出し直前に毎回`sts:get-caller-identity`で実アカウントIDを検証し、不一致時は即座に失敗する（PRD構造的要件、IdentityVerifierコンポーネント）
- 削除・変更操作（`delete-log-group`/`put-retention-policy`）は`--execute`明示時のみ実行し、既定はdry-run
- 依存クレートの脆弱性スキャン（`cargo audit`）とライセンス/供給網チェック（`cargo deny check`）をCI必須ゲートとする（Practices Discoveryで確定済み）

A. はい、この要件でよい
B. 変更したい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この要件でよい

## Q3. スケーラビリティ要件について

以下の目安でよいですか？
- 対象アカウント数: Organization内の全アクティブアカウント（PRD検証時点で9アカウント程度、将来的に数十アカウント規模まで想定）
- 対象リージョン数: CLI引数で指定可能（デフォルトは既知の主要リージョンの明示指定を必須とし、全リージョン自動列挙は行わない — 誤って無関係なリージョンをスキャンする事故を防ぐため）
- ログループ数: 1アカウント×1リージョンあたり数百件規模を想定し、メモリ上に全件保持しても問題ない設計とする

A. はい、この目安でよい
B. 変更したい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この目安でよい

## Q4. 信頼性要件について

以下の要件でよいですか？
- 可用性SLA/SLOは設定しない（常駐サービスではなく都度実行のCLIバイナリのため）
- 単一アカウントでのAPI呼び出し失敗（AssumeRole失敗、API一時エラー等）は、そのアカウントの処理のみを失敗として扱い、他アカウントのスキャン・処理は継続する（1アカウントの障害で全体を止めない）
- 監査ログの書き込み失敗時は、実行中の削除/retention変更操作自体を中断する（Practices Discoveryで確定済み、フェイルセーフを優先しフェイルオープンにしない）
- バックアップ/リカバリの概念は対象外（削除されたCloudWatch Logsのログデータ自体の復元はAWS側の制約により不可能であり、本ツールの責務は誤削除の防止であって削除後の復旧ではない）

A. はい、この要件でよい
B. 変更したい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この要件でよい

## Q5. 可観測性要件について

以下の要件でよいですか？
- 実行結果の監査ログ（AuditLogger）をローカルファイルに必須出力（PRD必須要件）。フォーマットはJSON Lines（1操作1行）とし、後から`jq`等で集計しやすくする
- 標準エラー出力に進捗・エラーメッセージを人間可読な形で出力する
- 外部監視システム（CloudWatch/Datadog等）への統合は本スコープでは対象外（CLIツールでありサービス監視の概念がないため）
- ダッシュボードは対象外

A. はい、この要件でよい
B. 変更したい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この要件でよい

## Q6. 技術スタックの選定について

PRDで既に提案されている以下のスタックで確定してよいですか？
- 言語: Rust（2021 edition以降）
- AWS SDK: `aws-config`, `aws-sdk-organizations`, `aws-sdk-sts`, `aws-sdk-cloudwatchlogs`
- 非同期ランタイム: `tokio`
- CLI引数: `clap`（derive API）
- 対話式選択UI: `inquire`
- シリアライズ: `serde` / `serde_json`
- テーブル表示: `comfy-table`
- エラー型: `thiserror`（ライブラリ的エラー定義）+ `anyhow`は使わずResult型を明示的に伝播（Practices Discoveryで確定した`unwrap`/`expect`/`panic`禁止方針と整合させるため）
- クレデンシャルのマスキング: `secrecy`クレート（Practices Discoveryで検討事項に挙げられていたもの）を採用

A. はい、この技術スタックで確定する
B. 変更したい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. はい、この技術スタックで確定する

---

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
