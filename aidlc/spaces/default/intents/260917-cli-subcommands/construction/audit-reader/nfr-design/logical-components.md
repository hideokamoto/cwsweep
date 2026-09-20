# logical-components.md — Unit: audit-reader

NFR設計の決定を、Infrastructure Design（次のConstructionステージ）へ
橋渡しするための論理コンポーネントビュー。

## コンポーネント一覧

| 論理コンポーネント | 種別 | デプロイモデル | 障害ドメイン | Blast Radius |
|---|---|---|---|---|
| `AuditReader`（`AuditRead`実装） | インプロセスライブラリモジュール | `cwsweep`単一バイナリに静的リンク。独立した実行時ランタイムを持たない | `cwsweep audit`コマンド呼び出し1回分のプロセス内に閉じる | 読み取り専用のため、失敗しても他コンポーネント（`scan`/`clean`の状態、監査ログの内容）に影響しない |

## インフラストラクチャへの含意

- **新規AWSリソースなし**: `AuditReader`はネットワークサービスでも
  クラウドリソースでもない。VPC・IAMロール・Lambda・コンテナ等の
  プロビジョニング対象は一切生じない。
- **Infrastructure Designステージへの申し送り**: 本Unitに関して
  Infrastructure Design（3.4）が追加すべき設計判断はない。`cwsweep`は
  既存どおりスタンドアロンバイナリとしてGitHub Releasesで配布される
  （team.md Deployment、タグ駆動リリース方式に変更なし）。
- **共有リソース**: 監査ログファイル（`--audit-log-path`で指定される
  ローカルファイル）を、既存の書き込み専用コンポーネント`AuditLogger`
  と共有する。ただし共有の方向は一方向（`AuditLogger`が書き、
  `AuditReader`が読む）であり、双方が同時に同一プロセス内で稼働する
  ことはない（`audit`サブコマンドの実行中は`clean`の削除・retention
  変更フローは動作せず、その逆も同様。単一プロセス・単一コマンドの
  CLIであるため）。したがって、読み取り側のファイルロック機構は
  設計上不要と判断する（同時アクセスによる競合はUnit間の設計課題では
  なく、そもそも`cwsweep`の実行モデル上発生しない）。

## 分離の妥当性（Well-Architectedの観点からの参考整理）

本Unitは実運用中のクラウドワークロードではなく、開発者が手元または
CIから実行するCLIツールの一部であるため、Well-Architected Frameworkの
可用性・スケーラビリティ・コスト最適化の各観点は形式的には非適用である。
唯一関連するのはSecurityピラー（最小権限・データ保護）であり、これは
`security-design.md`で扱った構造的分離（NFR2）とデータ整合性（NFR3）が
その実質的な対応である。
