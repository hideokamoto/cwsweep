# Infrastructure Design — Unit: audit-reader — 質問

`audit-reader`は`library`種別のUnitであり、`produces_kinds`上、この
ステージで生成されるのは`cicd-pipeline.md`・`traceability.json`のみ
（`infrastructure-specification.md`・`monitoring-design.md`は
`[service, ui, packaging]`のみが対象であり本Unitには生成されない）。
本Unitは既存の`cwsweep`単一Cargoパッケージ内のライブラリモジュールで
あり、独立したデプロイ対象・新規インフラを持たないため、CI/CDパイプライン
（CircleCI）に新規ジョブ・新規ステージは不要と判断した。個別の質問は
生成せず、以下でこの理解を確認する。

## Consolidated Summary Confirmation

- `cicd-pipeline.md`は、新規CircleCIジョブを追加せず、既存の
  `fmt`/`clippy`/`test`/`coverage`/`audit`/`deny`ジョブが`audit-reader`
  Unitのコード（同一`lib`クレート内）をそのままカバーする方針を記録する
- ただし、team.mdが要求する「破壊的操作関連コードパス等での100%パス
  カバレッジ＋境界値テスト」という*本Unit固有の追加基準*は、既存の
  `cargo llvm-cov --lib --fail-under-lines 80`という単一グローバル閾値
  では機械的に強制されないため、この検証手段をどう担保するか（PRレビュー
  時の手動確認か、モジュール限定のカバレッジレポート確認か等）を
  Build and Testステージで具体化する必要がある旨をGAPとして
  `traceability.json`に明記する

- Looks correct
- Request changes

[Answer]: Looks correct
