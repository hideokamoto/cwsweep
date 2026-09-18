# Units Generation — 質問

本intentは単一Cargoパッケージ（bin `src/main.rs` + lib `src/lib.rs`、既存13コンポーネント）の
内部再構成であり、新規の独立デプロイ対象・新規サービス境界は生じない
（domain-design/components.mdの通り、変更が生じるのは`CliApp`と`AuditLogger`周辺のみ）。

## Q1. Unit境界戦略


```question
prompt: "Unitの境界はどの基準で切りますか?"
header: "Unit境界戦略"
multiSelect: false
options:
  - label: "A. 機能ストリーム別（CLI基盤/audit新設/破壊的変更ドキュメント化）"
    description: "①scan/cleanのサブコマンド化＋既存ハンドラ再利用（CliApp中心）、②AuditReader新設（構造的分離が要求される独立業務ロジック）、③README/CHANGELOG移行ガイド+バージョン更新、の3ストリームに分ける。AuditReaderは他と依存を持たないためBoltを並行化しやすい"
  - label: "B. 単一Unit（クレート全体を1つのUnitとして扱う）"
    description: "単一Cargoパッケージ・単一開発者体制であり、実質的に1つのBoltで完結させる。Unit分割の恩恵（並行開発・独立デプロイ）が乏しいと判断する場合"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 機能ストリーム別（推奨どおり確定）

## Q2. Unit粒度

```question
prompt: "Unitの粒度（粗粒度/細粒度）はどちらを優先しますか?"
header: "Unit粒度"
multiSelect: false
options:
  - label: "A. 粗粒度（3 Unit程度に留める）"
    description: "cli-foundation（scan/clean+ディスパッチ）、audit-reader（audit新設）、release-docs（README/CHANGELOG/バージョン）の3 Unitに留め、過剰な分割を避ける"
  - label: "B. 細粒度（サブコマンドごとに独立Unit、例: scan/clean/audit/docsの4 Unit）"
    description: "scanとcleanも別Unitとして分離する。ただし両者は既存のscan_all等の内部メソッドを共有するため、実装上の重複や依存の逆流が生じやすい"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 粗粒度（3 Unit程度）（推奨どおり確定）

## Q3. 依存順序の許容度

```question
prompt: "Unit間の依存順序は厳密なトポロジカル順のみ許容しますか、それとも独立Unit間の並行実施を許容しますか?"
header: "依存順序"
multiSelect: false
options:
  - label: "A. 並行実施を許容する"
    description: "audit-readerはCliAppのCommands enumへの追加が必要な点を除けば既存の削除・実行系コンポーネントに依存しないため、cli-foundationと並行してBolt実施が可能な設計とする"
  - label: "B. 厳密なトポロジカル順のみ（並行実施なし）"
    description: "単一開発者体制でありBoltを並行実施する体制がないため、DAGが許す並行性があっても常に1本の順序で実施する前提とする（DAGの記述自体は変わらないが、2.9 Delivery Planningでの解釈が変わる）"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 並行実施を許容（推奨どおり確定）

## Q4. Unit間の統合点

```question
prompt: "Unit間の統合点（API・共有データ・イベント）はどう扱いますか?"
header: "統合点"
multiSelect: false
options:
  - label: "A. 共有コード境界として扱う（clapのCommands enum・CliAppメソッドシグネチャ）"
    description: "Unit間にネットワークAPIや非同期イベントは存在しない。統合点は同一クレート内の型シグネチャ（Commands enum、run_scan/run_clean/run_auditのメソッドシグネチャ、OutputFormatterの入力型）であり、domain-design/components.mdのdepends_on関係がそのままUnit間統合点になる"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 共有コード境界として扱う（単一の妥当な選択肢のため人間確認なしで確定、承認ゲートで委ねる）

## Q5. デプロイモデル

```question
prompt: "Unitのデプロイモデル（モノリシック/独立/ハイブリッド）はどれですか?"
header: "デプロイモデル"
multiSelect: false
options:
  - label: "A. モノリシック（全Unitが単一バイナリにビルドされる）"
    description: "team.md Deploymentの通り、本ツールはタグ駆動リリースでクロスプラットフォームの単一バイナリを配布する。Unit分割はConstruction内での作業分割であり、デプロイ単位ではない"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. モノリシック（単一の妥当な選択肢のため人間確認なしで確定、承認ゲートで委ねる）

## Consolidated Summary Confirmation

- Unit境界戦略: 機能ストリーム別（Q1:A）
- Unit粒度: 粗粒度・3 Unit（Q2:A）— audit-reader（library）/ cli-foundation（service）/ release-docs（packaging）
- 依存構造: audit-reader → cli-foundation → release-docs の一方向チェーン（Q3:A、並行実施の余地はDAG上available）
- Unit間統合点: 同一クレート内の型シグネチャ（Q4:A）
- デプロイモデル: モノリシック単一バイナリ（Q5:A）
- 分解プラン（Step 4）は Approve Plan で承認済み

- Looks correct
- Request changes

[Answer]: Looks correct
