# Initiative Brief — cwsweep

## Intent and Problem Statement

AWS Organization横断で複数アカウント・複数リージョンにCloudWatch Logsのログループが無期限保持のまま蓄積し、Free Tierの上限に近づく、あるいは超過するインシデントが発生している。既存のbashスクリプト（`check.sh`）による調査・対応は、`AWS_PROFILE`によるクレデンシャル取り違えと`describe-log-groups`のページネーション未対応という2つの構造的な事故を繰り返した。cwsweepはこれらを構造的に防止するRust製の単一バイナリCLIとして、Organization横断のスキャンから対話式削除・retention変更までを1ツール内で完結させる。

## Market Validation Summary

Not applicable（market-researchはSKIP。社内/自己利用ツールであり市場調査の対象となる市場が存在しない）。

## Feasibility and Risk Highlights

feasibilityステージはSKIP（技術スタックはPRDで確定済みの標準的なAWS SDK統合パターンであり実現性検証は不要と判断）。主要リスクとその構造的緩和策は以下の通り、PRDの非機能要件に明記済み:

- 認証情報取り違え → アカウントごとの明示的`CredentialsProvider` + `sts:get-caller-identity`による直前検証、不一致時は即時失敗
- ページネーション未対応による集計誤り → 全ページ取得後にメモリ上で集計する独立層
- 誤削除事故 → dry-runデフォルト、削除直前の二重Identity検証、監査ログ必須出力、対話式選択の初期全チェックOFF

## Scope Boundary

`cwsweep-orgwide-logs-cleanup`スコープ（13/33ステージ、Standard深度、Change Control: strict）。v1はCloudWatch Logsのみを対象とし、EC2/S3/EBS等の他リソース、自動削除判定、OAMセットアップ、常駐化はOut of Scope。MVPはPRDのマイルストーンM1〜M5（実削除M6は別段階）。

## Concept Visuals

Not applicable（rough-mockupsはSKIP。CLIツールでありUIモックアップの価値が薄いためPRDのM3/M4記述で代替）。

## Team Plan

Not applicable（team-formationはSKIP。単独開発者による実装であり複数チームの調整は発生しない）。

## Go/No-Go Recommendation

**Go。** 意図・スコープは開発者本人（唯一のステークホルダー）によって既に合意されており、主要リスクには構造的な緩和策がPRDに明記済み、予算・リソースのコミットメントも本人の時間投資のみで完結する。Inceptionフェーズ（Practices Discovery → Requirements Analysis相当をPRDで代替 → Domain Design → Units Generation → Delivery Planning相当をPRDのM1-M6で代替）へ進むことを推奨する。

## Assumptions & Open Questions

None.
