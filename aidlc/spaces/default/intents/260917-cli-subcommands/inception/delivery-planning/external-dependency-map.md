# external-dependency-map.md — 外部依存マップ（260917-cli-subcommands）

本intentに外部依存（このチーム外の要因でブロックされうるもの: 外部API・データ提供・
承認プロセス・他チームからのハンドオフ等）は存在しない。

## 理由

- cwsweepは単一Cargoパッケージのスタンドアロンバイナリであり、本intentはその内部の
  CLIインターフェース再構成（フラグ方式→サブコマンド方式）に限定される。
- 既存のAWS Organizations／CloudWatch Logs／STS APIへの依存は変更されない（既存実装を
  そのまま再利用する）。新規の外部サービス統合・新規の外部API公開は行わない
  （contract-summary.md「外部APIは公開しない」と整合）。
- 単一開発者体制（team.md Way of Working）であり、他チームからのハンドオフ待ちは
  発生しない。
- 唯一の承認プロセスはBolt 2（cli-foundation、`--execute`実装含む）完了時の人間承認
  ゲートであるが、これはチーム内部のプロセス（team.md Walking Skeleton方針）であり、
  外部依存には該当しない。

## Bolt別チェック

| Bolt | 外部依存 | ブロック要因 | 対応 |
|---|---|---|---|
| Bolt 1: audit-reader | なし | — | — |
| Bolt 2: cli-foundation | なし（内部承認ゲートのみ） | 人間の`--execute`実装承認 | team.mdの既存プロセスに従う。外部要因ではない |
| Bolt 3: release-docs | なし | — | — |
