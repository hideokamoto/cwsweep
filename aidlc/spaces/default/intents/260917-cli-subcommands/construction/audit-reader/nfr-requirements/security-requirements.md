# security-requirements.md — Unit: audit-reader

Inception段階の `NFR2`（構造的安全性）と `NFR3`（監査ログ完全性）を、
`audit-reader` Unitの詳細要件として具体化する。`produces_kinds`上、この
Unit（`library`種別）が生成するNFR文書は本ファイルのみである
（performance/scalability/reliability/observability-requirements.mdは
このUnitには生成されない）。

## NFR2: 構造的安全性（削除・retention変更系への非依存）

### NFR2.1 型レベルの到達不能性

- **要件**: `AuditRead` トレイトの実装（`AuditReader`）、およびその依存する
  型グラフは、`delete-log-group`・`put-retention-policy`を実行しうる型
  （`ExecutionEngine`）、および監査ログ書き込み系の型（`AuditWrite`・
  `AuditLogger`）へコンパイル時点で到達不能でなければならない。
- **脅威モデル（STRIDE: 権限昇格）**: `audit`サブコマンド経由で、意図せず
  破壊的操作（削除・retention変更）や監査ログの改ざんが実行可能になる
  経路が生まれること。読み取り専用であるべきコマンドが書き込み・削除の
  権限昇格経路を持ってしまうリスクに対応する。
- **検証方法**: コードレビューのみによる担保は許容しない。Code
  Generation/Build and Test段階で、コンパイル成立を前提とした型／モジュール
  境界の構造的回帰テスト（またはアーキテクチャレベルの統合テスト）を
  設計・実装し、依存グラフに`ExecutionEngine`/`AuditWrite`が含まれない
  ことを機械的に検証する。
- **出典**: `NFR2`、`project.md` Forbidden（audit経路の構造的分離）、
  `rules.md` BR4.2。

### NFR2.2 依存性注入の限定

- **要件**: `run_audit`ハンドラ（`cli-foundation` Unit側の実装対象）へ注入
  される依存は、`AuditReader`が提供する読み取り専用型（`AuditRead`相当）
  のみに限定される。`AuditWrite`・`ExecutionEngine`等の書き込み・削除系の
  型は一切含めない。
- **検証方法**: NFR2.1と同一の構造的回帰テストの対象に含める。
- **出典**: `NFR2`、`project.md` Forbidden、`contract-summary.md` Contract 1
  （`forbidden_dependencies: [ExecutionEngine, AuditWrite]`）。

## NFR3: 監査ログ完全性（データ整合性・堅牢性）

### NFR3.1 不正行に対する堅牢性（OWASP: データ整合性の失敗への対策）

- **要件**: 監査ログ（信頼境界の外側にある、手動編集や部分的破損の
  可能性があるローカルファイル）の読み取り中に、不正フォーマットまたは
  必須フィールド欠落の行に遭遇しても、プロセスをクラッシュ・パニック
  させず、当該行をスキップして警告を出力し、残りの行の処理を継続する
  （`unwrap()`/`expect()`/`panic!`を用いない。`project.md` Forbidden）。
- **脅威モデル（STRIDE: サービス拒否）**: 1行の破損データによって
  `audit`コマンド全体が異常終了し、残りの正常なエントリが一切参照でき
  なくなること（可用性への影響）を防ぐ。
- **検証方法**: 少なくとも1行の不正フォーマット行を含む監査ログ
  フィクスチャを用いた統合テストで、(a) 不正行がスキップされ警告が
  出力されること、(b) 不正行の前後にある正常なエントリが引き続き
  表示されることの両方を検証する。
- **出典**: `NFR3`、`rules.md` BR1.1/BR1.2。

### NFR3.2 境界値・異常系の網羅（100%パスカバレッジ）

- **要件**: `AuditRead`実装の異常系分岐（不正フォーマット行の判定、
  必須フィールド欠落の判定、ファイル不在時の空扱い正常終了、ファイル
  不在以外のI/Oエラー）は、通常コードの80%行カバレッジ floorとは別に、
  **100%パスカバレッジ＋境界値テスト**（複数の不正行が連続する場合、
  ファイルの最終行が不正な場合等）を満たさなければならない。
- **検証方法**: `cargo llvm-cov`によるカバレッジ計測（team.md Testing
  Postureの計測範囲に準拠）。
- **出典**: `NFR3`、`team.md` Testing Posture カバレッジ基準。

### NFR3.3 読み取り専用性による監査ログ保護

- **要件**: `AuditReader`は監査ログファイルへの書き込み・新規作成・
  改変を一切行わない。ファイル不在時も新規ファイルを作成しない
  （`AuditLogger`の追記専用契約を、読み取り側から侵害しない）。
- **脅威モデル（STRIDE: 改ざん）**: 監査ログ閲覧という本来無害な操作が、
  意図せず監査証跡そのものを変更・破壊してしまうこと（監査証跡の
  信頼性喪失）を防ぐ。
- **検証方法**: `entries()`呼び出し前後でファイルの最終更新時刻・
  バイト列が変化しないことを検証するテスト。
- **出典**: `NFR3`、`rules.md` BR4.1。

## 適用外の確認（Anti-Requirements）

- 認証・認可（NFR-AUTH/NFR-AUTHZ）: `audit-reader`はプロセスローカルの
  ファイル読み取りのみであり、ネットワーク越しの認証・認可の対象では
  ない。適用外。
- 暗号化（NFR-DATA）: 監査ログファイル自体の暗号化は本intentのスコープ
  外（既存`AuditLogger`の暗号化方針を変更しない）。適用外。
- コンプライアンスフレームワーク（PCI-DSS/HIPAA等）: 本プロジェクトは
  組織内AWSリソース管理ツールであり、対象データ（アカウントID・
  リージョン・ロググループ名）に規制対象の機微データ区分は含まれない。
  適用外。
