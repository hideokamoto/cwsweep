# unit-of-work.md — Unit定義（260917-cli-subcommands）

本intentは単一Cargoパッケージの内部再構成であり、新規デプロイ対象は生じない。Unit分割は
Construction（Bolt実施）上の作業分割であり、デプロイ単位ではない（domain-design/components.md
参照）。

## Unit一覧

| Unit ID | Directory | Unit名 | Kind | 複雑度 |
|---|---|---|---|---|
| U1 | u1-audit-reader | audit-reader | library | M |
| U2 | u2-cli-foundation | cli-foundation | service | M |
| U3 | u3-release-docs | release-docs | packaging | S |

## U1: audit-reader

**Directory**: `u1-audit-reader`

### 説明
`audit`サブコマンド専用の新設コンポーネント`AuditReader`（読み取り専用の監査ログパーサ）と、
その所有エンティティ`AuditEntry`を実装する。既存の`src/audit.rs`に同居させつつ、削除・
retention変更API（`ExecutionEngine`）および監査ログ書き込み系（`AuditLogger`/`AuditWrite`）
への依存を型レベルで一切持たない構造とする（project.md Forbidden・FR4.5・NFR2）。

### 境界・責務
- 監査ログ(JSON Lines)ファイルの行単位ストリーミング読み取り・パース
- 不正フォーマット行のスキップ+警告出力、正常行の継続表示（FR4.4、practices-discovery Q7）
- 対象ファイル不在時は空扱いで正常終了（FR4.6）
- `AuditEntry`エンティティの列挙

### デプロイモデル
standalone（同一バイナリ内のライブラリモジュール。独立した実行可能物ではない）

### 複雑度
M — 新規コンポーネント＋構造的分離の型設計＋NFR2/NFR3の回帰テスト要件

### Unit Kind
`library` — 再利用可能なコードで、独立した実行時ランタイムを持たない（domain-design/components.md
の`AuditReader`、`depends_on: []`）

### 実装ノート・制約
- `run_audit`ハンドラへ注入する依存はこのUnitが提供する読み取り専用型（`AuditRead`相当）のみ
- U2（cli-foundation）が本Unitの公開APIを呼び出す（Commands enumのAuditバリアント経由）
- 依存注入グラフに削除・retention変更・監査ログ書き込み系の型を含めないことを検証する構造的
  回帰テストを含める（team-practices.md Q4対応）

### カバーする要件
FR4.1, FR4.2, FR4.3（OutputFormatter側の実装だがAuditReaderが出力対象データを提供）, FR4.4,
FR4.5, FR4.6, NFR2, NFR3

---

## U2: cli-foundation

**Directory**: `u2-cli-foundation`

### 説明
`clap`ベースの`Cli`/`Commands`（`scan`/`clean`/`audit`の3バリアント）の再構成と、サブコマンド
別ディスパッチロジック（`run_scan`/`run_clean`/`run_audit`）を実装する。既存の`scan_all`→
`select`→`plan`→`confirm`→`execute`という`clean`のデフォルトフローはそのまま再利用する
（ADR-001）。`OutputFormatter`を拡張し、`AuditEntry`一覧のtable/JSON出力にも対応させる
（ADR-003）。

### 境界・責務
- `Cli`/`Commands`enum定義とサブコマンド判定・ディスパッチ（`lib`側、FR1.1・FR1.2）
- `run_scan`: スキャンのみ実行、対話式選択・確認・実行には進まない（FR2.1-FR2.4）
- `run_clean`: 既存のスキャン→選択→確認→実行フローを担う（FR3.1-FR3.5）
- `run_audit`: U1の`AuditReader`を呼び出し、結果を`OutputFormatter`経由で表示（FR4.1、FR4.3）
- 旧フラグ形式（`--scan-only`/`--execute`単体）の廃止と、サブコマンド未指定時のclap標準
  エラー表示（FR6.1）
- `src/main.rs`は薄いアダプタ配線層として維持（FR1.2、practices-discovery Q3）

### デプロイモデル
standalone（`main.rs`がリンクする単一バイナリの中核。実行可能物そのもの）

### 複雑度
M — 既存12コンポーネントの呼び出し順序再編＋新設U1との結線＋後方互換シムなしの回帰確認

### Unit Kind
`service` — ビルドされる実行可能物（`cwsweep`バイナリ）そのものを構成する

### 実装ノート・制約
- U1（audit-reader）の公開APIに依存する（`run_audit`がAuditReaderを呼ぶ）
- 既存の`IdentityVerifier`二重検証・`AuditLogger`必須記録・dry-run既定はいずれも変更しない
  （FR5.1-FR5.4、project.md Mandated）
- サブコマンド判定・ディスパッチロジックは`cargo llvm-cov --lib`の計測対象に含まれる`src/cli.rs`
  等に実装する（NFR1）
- `--execute`実装を含む本Unitの実行系パスは、team.md Walking Skeletonの方針によりBolt完了時に
  毎回ゲート対象となる

### カバーする要件
FR1.1, FR1.2, FR2.1, FR2.2, FR2.3, FR2.4, FR3.1, FR3.2, FR3.3, FR3.4, FR3.5, FR4.3（出力整形側）,
FR5.1, FR5.2, FR5.3, FR5.4, FR6.1, NFR1, NFR5

---

## U3: release-docs

**Directory**: `u3-release-docs`

### 説明
CLIサブコマンド化という破壊的変更をREADME/CHANGELOGに明示し、移行ガイド（旧フラグ→新
サブコマンドの対応表）を記載する。`Cargo.toml`のバージョンを`0.1.0`から`0.2.0`へ更新する。

### 境界・責務
- README.mdへの新コマンド体系の説明追加
- CHANGELOGへの破壊的変更エントリ追加（移行ガイド含む）
- `Cargo.toml`バージョン更新

### デプロイモデル
standalone（ドキュメント・メタデータ変更のみ。実行可能物には寄与しない）

### 複雑度
S — ドキュメント記述とバージョン番号変更のみ、新規ロジックなし

### Unit Kind
`packaging` — ビルド・配布物（バージョンメタデータ・リリースノート）を構成し、独自の
業務ロジックモデルを持たない

### 実装ノート・制約
- U2（cli-foundation）とU1（audit-reader）双方の最終的な振る舞いが確定してから記述する
  （移行ガイドの正確性を担保するため）
- タグ駆動リリース方式・M6チェックリスト要件自体は変更しない（team.md Deployment）

### カバーする要件
FR6.2, FR6.3

---

## Coverage Verification

全6件のFR群（FR1-FR6）・5件のNFR群（NFR1-NFR5）が上記3 Unitのいずれかに割り当てられている
（詳細は`traceability.json`参照）。
