# Architecture Decision Records — Domain Design

## ADR-001: LogGroupScannerとScanAggregatorを分離する

- **Context**: PRD背景にある2つの構造的事故のうち1つは、`describe-log-groups`のページネーション未対応による集計誤り（1ページ最大50件にしか集計が及ばなかった）。この事故を構造的に再発防止する必要がある。
- **Decision**: ログループの取得（全ページ完了を保証する`LogGroupScanner`）と、取得済みデータの集計（`ScanAggregator`）を別コンポーネントに分離する。
- **Consequences**: 「1ページだけ見て集計する」実装がコード構造上書けなくなる（LogGroupScannerは全ページ取得完了後にのみScanAggregatorへデータを渡す）。コンポーネント数は1つ増えるが、可逆性は高い。
- **Alternatives Rejected**: 取得と集計を1コンポーネントに統合する案。実装がシンプルになる一方、途中経過での集計呼び出しを防ぐ構造的な強制力がなく、PRDの非機能要件（ページネーション完全対応の構造的強制）を満たせないため却下。

## ADR-002: CredentialProviderとIdentityVerifierを独立コンポーネントとする

- **Context**: PRD背景にあるもう1つの構造的事故は、`AWS_PROFILE`環境変数がAssumeRoleで取得した一時クレデンシャルより優先され、全アカウントで同じアカウントを見てしまったこと。
- **Decision**: クレデンシャル構築（`CredentialProvider`）と、API呼び出し直前の実アカウントID検証（`IdentityVerifier`）を独立コンポーネントとし、環境変数由来の暗黙のクレデンシャルチェーンに他コンポーネントが依存できないようにする。IdentityVerifierはLogGroupScanner呼び出し前とExecutionEngine（削除実行）直前の両方で呼び出される（二重検証）。
- **Consequences**: すべてのAWS API呼び出しがCredentialProvider経由の明示的クレデンシュルを使う設計になり、環境変数の影響を受けない。呼び出し箇所が増える分、IdentityVerifier呼び出しの徹底が実装規律として必要（NFR Requirementsで具体的な受入基準に落とし込む）。
- **Alternatives Rejected**: AWS SDKのデフォルト認証チェーンに委ねる案。実装は簡単だが、まさにPRD背景の事故を再発させる設計であるため却下。

## ADR-003: 計画・確認・実行を3コンポーネントに分離する

- **Context**: 誤削除事故ゼロという成功指標を満たすには、「何をするか決める」「本当にするか確認する」「実際にする」の間に明確な境界が必要。
- **Decision**: `ActionPlanner`（計画）、`ConfirmationPresenter`（確認）、`ExecutionEngine`（実行）を独立コンポーネントとし、ConfirmationPresenterの確認を経ずにExecutionEngineへ到達できない依存構造にする。
- **Consequences**: 誤って確認をスキップして実行するコードパスが構造的に作りにくくなる。コンポーネント数は増えるが、それぞれの責務が単純になりテストしやすい。
- **Alternatives Rejected**: ActionPlannerとConfirmationPresenterを1コンポーネントに統合する案。実装は簡略化されるが、「計画」と「人間の最終確認」という異なる関心事が混在し、確認ロジックの単体テストが難しくなるため却下。

## ADR-004: AuditLoggerを独立コンポーネントとし、書き込み失敗時は操作を中断する

- **Context**: PRDおよびPractices Discoveryのインタビュー（Q7）で、監査ログは「無効化オプションなし」の必須要件であり、書き込み失敗時は操作自体を中断する方針が確定している。
- **Decision**: `AuditLogger`を独立コンポーネントとし、`ExecutionEngine`から呼び出す形にする。書き込み失敗時はExecutionEngine側でエラーとして扱い、削除/retention変更操作を中断する。
- **Consequences**: 監査ログの記録漏れを構造的に防げる。ExecutionEngineとAuditLoggerの間でエラー伝播（Result型）の設計が重要になる。
- **Alternatives Rejected**: 監査ログ出力をExecutionEngine内部の一処理として実装する案。責務分離が弱く、監査ログのテストがExecutionEngineの他の処理と混在するため却下。

## Assumptions & Open Questions

None.
