# risk-and-sequencing-rationale.md — Bolt順序の根拠（260917-cli-subcommands）

## 採用した順序

audit-reader (Bolt 1) → cli-foundation (Bolt 2) → release-docs (Bolt 3)

## 根拠

本intentのUnit依存DAG（`unit-of-work-dependency.md`）は単一の一方向チェーンであり、
トポロジカルに有効な順序は実質的にこの1通りしかない
（audit-readerはcli-foundationに依存され、cli-foundationとaudit-readerの両方が
release-docsに依存される）。したがって、正式なスコアリングモデル（WSJF等）を導入する
までもなく、DAGの制約自体が順序を強く規定する。

この制約の範囲内で、**リスク優先**（Boehm, Spiral Model：不確実性の高い部分を先に着手し、
後続の意思決定を較正する）の考え方を補助的に適用した:

- **audit-reader（Bolt 1）**は新設コンポーネントであり、既存の破壊的操作系コンポーネント
  （`ExecutionEngine`/`AuditWrite`）に一切依存しない構造的分離という、project.mdの
  Forbidden制約に由来する特異な設計要件を持つ。この要件を満たせるかどうかは実装して
  みるまで確証が持てない不確実性であり、最も早い段階で検証する価値がある。
- **cli-foundation（Bolt 2）**は既存12コンポーネントの呼び出し順序を再編し、かつ
  `--execute`による実削除・実retention変更を含む。既存の安全機構（Identity二重検証・
  dry-run既定・監査ログ必須記録）を壊さずに再編できるかという回帰リスクが最大の
  Boltであり、DAG制約上も2番目に着手せざるを得ない。
- **release-docs（Bolt 3）**はドキュメント記述のみで新規ロジックがなく、複雑度も最小
  （S）である。Bolt 1・2の両方が確定して初めて正確な移行ガイドを書けるため、必然的に
  最後になる。

**ウォーキングスケルトン（Cockburn, Crystal Clear）は採用しなかった**: 本プロジェクトの
スコープファイルは`skeleton: off`を宣言しており、team.md Walking Skeletonセクションも
「最初のBoltから通常の機能実装として進める」と明記している。単一Cargoパッケージの
内部再構成であり、証明すべき新規アーキテクチャ層（新しいデプロイ先・新しい外部統合等）が
存在しないため、薄い一枚岩を独立して作る価値が乏しいと判断した既存方針をそのまま踏襲する。

**WSJF（Reinertsen / SAFe）は採用しなかった**: Bolt数がわずか3件、かつDAG制約により
実質的に順序の自由度がないため、価値・緊急度・規模を定量化してスコアリングする
コストが、得られる判断上の利益を上回らないと判断した。

## DAGからの逸脱の有無

**逸脱なし**。採用した順序（Bolt 1→2→3）は`unit-of-work-dependency.md`のトポロジカル順
そのものである。DAG上、audit-reader（Bolt 1）はcli-foundation（Bolt 2）の着手を待たずに
並行実施できる余地があるが（同ファイル「並行開発の余地」参照）、本intentは単一開発者
体制・AI単独セッションでの逐次実施を採用しているため、並行実施の余地を行使しない
という判断であり、DAGが許さない順序への逸脱ではない。

## 参照した知見

- Boehm, *Spiral Model*（リスク優先の考え方）
- Reinertsen, *Principles of Product Development Flow*（WSJF、今回は不採用の判断根拠として参照）
- Cockburn, *Crystal Clear*（ウォーキングスケルトン、今回は不採用の判断根拠として参照）
