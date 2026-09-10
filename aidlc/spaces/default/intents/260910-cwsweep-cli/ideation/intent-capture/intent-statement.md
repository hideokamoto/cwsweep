# Intent Statement — cwsweep

## Problem Statement

AWS Organization横断で複数アカウント・複数リージョンにCloudWatch Logsのログループが無期限保持のまま蓄積し、Free Tierの上限に近づく、あるいは超過するインシデントが発生している。この状況を調査・是正するために場当たり的なbashスクリプト（`check.sh`）を用いたが、(1) `AWS_PROFILE`が環境変数に残っているとAssumeRoleで取得した一時クレデンシャルより優先され全アカウントで同じアカウントを見てしまう、(2) `describe-log-groups`のページネーション未対応により1ページ最大50件にしか集計が及ばず実態と乖離する、という2つの構造的な事故で何度も往復した。[Q1][Q4]

## Target Customer

開発者本人（作成者兼利用者）。複数のAWS Organizationアカウントを横断してCloudWatch Logsを調査・削除する際の手間と事故リスクを削減したい単独開発者。将来的に他のチームメンバーが利用者に加わる可能性は現時点では想定していない。[Q2][Q5]

## Success Metrics

- Organization横断でのログ調査から削除実行までを1つのツール内で完結できること
- 削除を伴う操作で、意図しないアカウント・意図しないログループを誤って削除した事故がゼロであること（監査ログで追跡可能）
- Cursor/Claude CodeのbashツールからJSON出力（`--output json`）でスキャン結果を取得できること
[Q3]

## Initiative Trigger

技術的負債・運用事故（`AWS_PROFILE`の取り違え、ページネーション未対応による集計誤り）を構造的に解消したいという内発的動機と、CloudWatch Logs Free Tier超過の請求警告という直接のインシデント対応の両方がトリガーとなっている。[Q4]

## Initial Scope Signal

- **Workflow-selected scope（[scope], workflow-selected）**: `cwsweep-orgwide-logs-cleanup`（カスタム合成scope、13/33ステージ実行、Standard深度、Change Control: strict）
- **User-confirmed product boundary（[Q8]）**: 上記スコープはユーザーの意図するプロダクト境界と一致することが確認済み。v1はCloudWatch Logsのみを対象とし、他リソース種別（EC2/S3/EBS等）への拡張は対象外（PRDのOut of Scope節に明記）。

## Assumptions & Open Questions

None.
