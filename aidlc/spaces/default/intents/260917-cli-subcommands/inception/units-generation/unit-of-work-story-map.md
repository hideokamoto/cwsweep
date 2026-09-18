# unit-of-work-story-map.md — 要件マッピング（260917-cli-subcommands）

本intentはUser Storiesステージをスキップした（`inception/user-stories/user-stories-assessment.md`
参照：単一ペルソナ・業務ロジック変更なし・単一開発者体制のため）。そのため本ステージの
`stage-protocol.md`指示に従い、`stories.md`の代わりに`requirements.md`の機能要件（FR）を
Unitへマッピングする。

## FR → Unit マッピング

| FR ID | 内容概要 | 実装Unit | Directory |
|---|---|---|---|
| FR1.1 | 3サブコマンド体系への再構成、未指定時エラー | U2 cli-foundation | u2-cli-foundation |
| FR1.2 | ディスパッチロジックのlib側実装 | U2 cli-foundation | u2-cli-foundation |
| FR2.1 | `scan`の棚卸し・表示 | U2 cli-foundation | u2-cli-foundation |
| FR2.2 | `scan`は`--audit-log-path`非対応 | U2 cli-foundation | u2-cli-foundation |
| FR2.3 | `run_scan`命名 | U2 cli-foundation | u2-cli-foundation |
| FR2.4 | `scan`の終了コード方針(R-03)維持 | U2 cli-foundation | u2-cli-foundation |
| FR3.1 | `clean`のデフォルトフロー | U2 cli-foundation | u2-cli-foundation |
| FR3.2 | `clean`は`--output`非対応 | U2 cli-foundation | u2-cli-foundation |
| FR3.3 | `--execute`未指定時の抑止(dry-run既定) | U2 cli-foundation | u2-cli-foundation |
| FR3.4 | `run_clean`命名 | U2 cli-foundation | u2-cli-foundation |
| FR3.5 | `--audit-log-path`既定値維持 | U2 cli-foundation | u2-cli-foundation |
| FR4.1 | `audit`の基本呼び出し・表示 | U1 audit-reader | u1-audit-reader |
| FR4.2 | 絞り込みなし(初回実装) | U1 audit-reader | u1-audit-reader |
| FR4.3 | `--output table|json` | U1 audit-reader + U2 cli-foundation(OutputFormatter) | u1-audit-reader, u2-cli-foundation |
| FR4.4 | 不正行スキップ+警告継続 | U1 audit-reader | u1-audit-reader |
| FR4.5 | `run_audit`の構造的分離 | U1 audit-reader | u1-audit-reader |
| FR4.6 | ファイル不在時は空扱いで正常終了 | U1 audit-reader | u1-audit-reader |
| FR5.1 | メンバーアカウントAPI呼び出し前のIdentity検証 | U2 cli-foundation | u2-cli-foundation |
| FR5.2 | 実行直前の二重目Identity検証 | U2 cli-foundation | u2-cli-foundation |
| FR5.3 | ページネーション完走集計 | U2 cli-foundation | u2-cli-foundation |
| FR5.4 | 管理アカウント/メンバーアカウントのAssumeRole非対称性 | U2 cli-foundation | u2-cli-foundation |
| FR6.1 | 旧フラグ廃止・後方互換なし | U2 cli-foundation | u2-cli-foundation |
| FR6.2 | README/CHANGELOG移行ガイド | U3 release-docs | u3-release-docs |
| FR6.3 | Cargo.tomlバージョン更新 | U3 release-docs | u3-release-docs |

## 横断的関心事(Cross-cutting)

- **FR4.3**（`--output table|json`）はU1（AuditReaderが提供するAuditEntryデータ）とU2
  （OutputFormatterによる実際の整形処理）の両方にまたがる。domain-design/ADR-003の通り、
  出力整形自体はOutputFormatter（U2側）が担うが、対象データの供給元はU1である。

## Unit内実装順序

### U1 audit-reader
1. `AuditEntry`エンティティ定義
2. JSON Linesストリーミング読み取り・パース（不正行スキップ+警告、FR4.4）
3. ファイル不在時の空扱い正常終了（FR4.6）
4. 依存注入グラフの構造的分離を検証する回帰テスト（NFR2）

### U2 cli-foundation
1. `Cli`/`Commands`enum再構成（FR1.1）
2. `run_scan`実装（既存`scan_all`等の再利用、FR2.1-FR2.4）
3. `run_clean`実装（既存フローの再利用、FR3.1-FR3.5）
4. `run_audit`実装（U1呼び出し、FR4.1・FR4.3）
5. `OutputFormatter`拡張（AuditEntry一覧の整形、FR4.3）
6. 旧フラグ廃止・エラー表示（FR6.1）

### U3 release-docs
1. README.md更新
2. CHANGELOG更新（移行ガイド）
3. `Cargo.toml`バージョン更新

## Coverage Verification

全24件のFR（FR1.1〜FR6.3）が上記のいずれかのUnitに割り当てられている。Unit未割当のFRは
存在しない（GAPなし）。各Unitは最低1件のFRを担う（U1: 6件、U2: 17件、U3: 2件、FR4.3は
両方にカウント）。
