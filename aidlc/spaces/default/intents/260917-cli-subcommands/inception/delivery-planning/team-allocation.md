# team-allocation.md — Bolt担当割当（260917-cli-subcommands）

本intentのTeam Formation（1.5）はスコープ`classic`のため実施されていない（スキップ）。
team.mdのWay of Working（単一開発者体制）を踏まえ、全Boltを単一の**mob**（Boltを担当する
実施単位。ここでは人間の複数人チームではなく、AIエージェント1体が担当するという意味）
——`aidlc-developer-agent`——が担当する。

## Bolt-to-Mob 割当表

| Bolt | Unit | 担当Mob | 備考 |
|---|---|---|---|
| Bolt 1: audit-reader | U1 | aidlc-developer-agent | — |
| Bolt 2: cli-foundation | U2 | aidlc-developer-agent | `--execute`実装含む。Bolt完了時に人間承認ゲート必須 |
| Bolt 3: release-docs | U3 | aidlc-developer-agent | — |

## 備考

- Team Formation（1.5）がSKIPのスコープ（`classic`）であるため、Program Board
  （複数チームの引き渡しタイミングを可視化する図）は本intentでは不要（担当チームが
  常に1つのため）。
- 単一開発者体制であるため、Bolt間の引き継ぎ・調整コストは発生しない。人間の関与は
  各Boltの承認ゲート（Bolt 2は必須、他は`Construction Autonomy Mode`のladder promptに従う）
  のみである。
