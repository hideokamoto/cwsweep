# Logical Components — NFR Design View

Domain Designの`components.md`で定義した12コンポーネントを論理インフラコンポーネントの単位としてそのまま用いる。ここではNFR観点（障害ドメイン・分離戦略）を追加する。

## 障害ドメインマッピング

| 障害ドメイン | 対象コンポーネント | 分離戦略 |
|---|---|---|
| ドメイン1: アカウント単位 | CredentialProvider, IdentityVerifier, LogGroupScanner | 1アカウントの失敗（AssumeRole失敗、Identity不一致、API一時エラー）は当該アカウントのみを失敗として扱い、他アカウントの処理を継続する（バルクヘッド、NFR4.2） |
| ドメイン2: 実行単位 | ExecutionEngine, AuditLogger | 1回の`--execute`実行全体で1つの障害ドメインとする。AuditLogger書き込み失敗は実行全体（現在処理中の操作）を中断する（フェイルセーフ、NFR4.3） |
| （分離対象外） | CliApp, OrgDiscovery, ScanAggregator, OutputFormatter, InteractiveSelector, ActionPlanner, ConfirmationPresenter | 読み取り専用または表示専用の処理であり、独立した障害ドメインを持たない |

## 共有リソース

ローカル監査ログファイル（AuditLoggerが書き込む単一ファイル）のみ。外部共有リソース（データベース、キャッシュ、キュー）は存在しない。

## Blast Radius

最大のBlast Radiusは「ExecutionEngineが誤ったPlannedActionを実行すること」であり、これはDomain DesignのADR-003（計画・確認・実行の3層分離）とNFR2.9（初期全選択OFF）によって構造的に抑制されている。

## Assumptions & Open Questions

None.
