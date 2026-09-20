# User Stories — 実施判断

## 判断: Skip

## 根拠

このintent（`260917-cli-subcommands`）は、既存のCLIツール(cwsweep)のフラグ構成を
サブコマンド構成(`scan`/`clean`/`audit`)へ再構成する作業であり、以下の理由で
User Storiesの生成価値が低いと判断した:

- **単一ペルソナ**: 利用者は現行と変わらず「AWS Organizationを運用するSRE/プラット
  フォームチーム」のみ（`business-overview.md`「想定利用者・利用シーン」参照）。新規
  ペルソナの追加はない。
- **業務ロジックの変更なし**: スキャン・削除・retention変更の業務ロジック自体
  （Identity二重検証、dry-run既定、監査ログ必須等）は一切変更しない。変更対象は
  CLI引数の分岐構造（`Cli`構造体とディスパッチロジック）のみ。
- **クロスチーム調整なし**: 単一開発者体制（team.md Way of Working）であり、複数
  チームの合意形成を要する変更ではない。
- **新設の`audit`サブコマンドも単純**: 唯一の新規機能である`audit`（監査ログ閲覧）
  は、既存の監査ログファイルを読み取り表示するだけの単純な read-only 機能であり、
  requirements.md の FR4 群に既に具体的な受け入れ基準相当の記述（絞り込みなし・
  出力形式・不正行スキップ挙動）がある。これをUser Story + Acceptance Criteria
  形式に変換しても、requirements.mdの記述以上の情報は追加されない。

## 検討した要因

- プロジェクト種別: brownfield、既存CLIの内部再構成
- ユーザー向けスコープ: CLIインターフェースの再構成のみ。エンドユーザーへの価値提案
  自体（AWS CloudWatch Logsの棚卸し・削除）は変わらない
- 複雑性シグナル: 低い。既存モジュール構造(12コンポーネント)への変更は限定的
  （`cli.rs`/`main.rs`のディスパッチ層のみ）

## 代替カバレッジ

`requirements.md`のFR1〜FR6・NFR1〜NFR5が、各サブコマンドの入出力仕様・振る舞い・
受け入れ可能な挙動を十分に具体的に記述しており、Domain Design以降の設計作業は
requirements.mdを直接の入力として進められる。
