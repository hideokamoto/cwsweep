# Evidence — practices-discovery（再確認: 260917-cli-subcommands）

## 参照した既存の affirm 済み記録

- `aidlc/spaces/default/memory/team.md` — `260910-cwsweep-cli` で affirm 済みの
  `## Way of Working` / `## Walking Skeleton` / `## Testing Posture` /
  `## Deployment` / `## Code Style`（5 セクションすべて非空であることを確認済み）。
- `aidlc/spaces/default/memory/project.md` — 同 intent で affirm 済みの
  `## Mandated`（9項目）/ `## Forbidden`（7項目）。いずれも AWS 破壊的操作の
  安全性（dry-run既定・二重Identity検証・監査ログ必須・クレデンシャル非出力・
  `unsafe`禁止・`unwrap`/`expect`/`panic!`禁止）に関するハード制約。

## 参照したブラウンフィールド逆設計エビデンス（`aidlc/spaces/default/codekb/cwsweep/`）

- `code-structure.md`: 単一 Cargo パッケージ（バイナリ `src/main.rs` + ライブラリ
  `src/lib.rs`、12 モジュール=12 コンポーネント）。`src/cli.rs`（`CliApp`）が
  `clap` の `Cli` 構造体とオーケストレーションを担い、`--regions`（必須）・
  `--role-name`・`--output`・`--execute`・`--scan-only`（`conflicts_with =
  "execute"`）・`--audit-log-path` というフラットなフラグ構成であることを確認。
  `src/main.rs` の `main()` にある `cli.scan_only` 判定 → 非TTYフォールバック →
  対話式選択・確認・実行、という条件分岐が本 intent の再配置対象であることを確認。
- `architecture.md`: 「Improvement Opportunities」節および「サブコマンド化に
  向けた設計上の要点（本 intent 固有）」節に、この intent の設計意図
  （`scan`＝スキャンのみ、`clean`＝選択・削除フロー＋`--execute`で実行/dry-run切替、
  `config`はスコープ外、`--audit-log-path`の無効化オプションなし制約は維持）が
  codekb 側に既に記述されていることを確認。また「監査ログの読み取り API 不在」が
  技術的負債シグナルとして挙げられており、新設の `audit` 閲覧サブコマンドには
  `AuditLogger`/`AuditWrite` への読み取り API 追加が必要になることを確認。
- `technology-stack.md` / `dependencies.md` / `code-quality-assessment.md` /
  `business-overview.md`: 参照したが、本 intent（CLI サブコマンド化）の
  team/project 慣行の再確認に直接影響する新規事実は見つからなかった
  （ports-and-adapters・`secrecy`によるクレデンシャルマスキング等は既存の
  team.md/project.md の記述と整合していることを確認しただけ）。

## リポジトリ状態

- 参照コミット: `git rev-parse HEAD` = `29e3a629ab808d3f5314ab7ed46ac1e756798e14`
  （評価時点、`/home/user/cwsweep`）。

## 推論・仮説（hypothesis）

- [hypothesis] `scan`/`clean`/`audit` の3サブコマンド構成は、`architecture.md`
  の記述と本 intent のスコープ説明から素直に導かれる分割だが、既存フラグ
  （`--role-name` 等の設定系）を各サブコマンドにどう配分するか（共通引数として
  `clap` の `#[command(flatten)]` を使うか、サブコマンドごとに重複定義するか）は
  実装方針であり、team.md の Code Style としてどこまで固定するかは未確定。
- [hypothesis] `audit` 閲覧サブコマンドは新規機能であり、既存の
  「破壊的操作は100%パスカバレッジ＋境界値テスト」という Testing Posture の
  対象には該当しない（閲覧のみで削除・retention変更を伴わない）と判断したが、
  この分類が team の意図と一致するか要確認。
- [hypothesis] CLI 引数の破壊的変更（`--scan-only`/`--execute` → サブコマンド）は
  既存利用者への互換性影響があり得るが、本プロジェクトは単一開発者体制かつ
  v1 未リリース段階と推測されるため、後方互換シム（旧フラグのエイリアス提供等）は
  不要と仮置きした。人間インタビューで確認が必要。

## 人間インタビューで解決すべき未解決事項（Assumptions & Open Questions で再掲予定）

1. サブコマンド化に伴う CLI インターフェースの破壊的変更について、後方互換
   （旧フラグのエイリアス）を提供する必要があるか、それとも v1 未リリース前提で
   不要と割り切ってよいか。
2. `audit` 閲覧サブコマンドの読み取り専用制約（監査ログファイルへの書き込み・
   変更・削除を一切行わない）を、project.md の Mandated/Forbidden として
   ハード制約に昇格すべきか、それとも team.md の設計方針レベルに留めるべきか。
3. サブコマンドハンドラの命名規則（`run_scan`/`run_clean`/`run_audit` 接頭辞方式）
   を Code Style の追加方針として team.md に affirm してよいか。
4. `--role-name` 等の設定系フラグを各サブコマンド共通の引数として
   `#[command(flatten)]` で共有する設計を、Code Style/Way of Working として
   明示的に方針化する必要があるか（単なる実装詳細として Code Generation 段階に
   委ねてよいか）。
5. 破壊的操作を伴う `clean` サブコマンドに到達する Bolt への毎回ゲート要件
   （Walking Skeleton セクション既存の M6 ゲート方針の踏襲）に異論がないか。
