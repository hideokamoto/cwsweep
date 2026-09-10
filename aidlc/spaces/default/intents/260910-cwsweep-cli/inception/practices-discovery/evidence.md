# エビデンス（ドラフト）

## 前提

本プロジェクト（cwsweep）は greenfield であり、参照可能な既存コード・既存 git 履歴・過去の
Construction 成果物は存在しない。そのため、本ドラフトの根拠は以下の2つのソースのみに基づく。

## ソース

1. **`aidlc/spaces/default/memory/org.md`** — フレームワークのデフォルトプラクティス
   （Way of Working: トランクベース開発・squash-merge、Testing Posture: test-after +
   80%カバレッジfloor、Deployment: マージ時ステージングデプロイ + 本番手動承認、
   Code Style: 言語標準ツールへの委任）。本プロジェクトには既存の `team.md` 内容が
   存在しない（greenfield）ため、これらは「確立済みのチーム事実」ではなく
   **提案されたデフォルト（suggested defaults）**として扱った。

2. **承認済み PRD/スコープ文書** —
   `../../ideation/scope-definition/scope-document.md`
   （scope-definition ステージの成果物）。以下の観点で `team-practices.md` と
   `discovered-rules.md` の草案に反映した:
   - Organization横断のAssumeRole構成とIdentity検証要件（M1）
   - 全ページ取得によるページネーション集計の構造的強制（M2）
   - dry-runデフォルト・`--execute`明示・二重Identity検証・監査ログ必須出力という
     破壊的操作に対する安全設計（Testing Posture の追加提案、discovered-rules の
     Mandated/Forbidden の直接の根拠）
   - スタンドアロンバイナリとしての配布形態（サーバーへのデプロイが存在しないため、
     Deployment セクションで org.md の「マージ時ステージングデプロイ」をそのまま
     適用せず、タグ駆動リリースへの読み替えを提案した根拠）
   - シーケンシング方針（リスク優先＝構造的事故の解消を最優先）を Walking Skeleton の
     候補スコープ提案に反映

## 適用しなかった/保留したもの

- 既存コードベースのスキャン、既存のCI設定ファイル、既存の `Cargo.toml` 等の実成果物は
  greenfield のため存在せず、参照していない。
- テスト手法（TDD/test-afterの最終選択）、リリース配布チャネルの詳細、`clippy`厳格度
  などチームの合意が必要な事項は、PRDに明記がないため推測で確定させず、
  `team-practices.md` 内に「[要インタビュー]」として明示した。
