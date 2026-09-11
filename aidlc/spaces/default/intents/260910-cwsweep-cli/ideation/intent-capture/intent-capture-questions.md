# Intent Capture — Questions

## Sources

- [desc] Initial description: "PRDに基づきcwsweep(仮称)を実装する。AWS Organization横断・対話式CloudWatch Logsクリーンアップの単一バイナリCLI(Rust)。PRDは既に確定済み(背景・目的・スコープ・非機能要件・技術スタック・成功指標・マイルストーンM1-M6を含む)。ゼロからのグリーンフィールド実装。"
- [scope] Workflow-selected scope: `cwsweep-orgwide-logs-cleanup`.

## Q1. どのようなビジネス上の問題を解決しますか？

PRDの背景に基づく前提: AWS Organization横断で溜まったCloudWatch Logsの容量がFree Tierの上限に近づき、また不要ログの手動確認・削除作業が `check.sh` のような場当たり的なbashスクリプトの試行錯誤に頼っており、認証情報の取り違え・ページネーション未対応などの構造的事故を繰り返している。

A. 上記の前提の通りで正しい
B. 少し違う（自由記述で補足する）
C. Not yet defined
X. Other (please specify)

[Answer]: A. 上記の前提の通りで正しい

## Q2. 顧客は誰ですか（社内/社外）？どんな痛みを抱えていますか？

A. 社内の自分自身（単独開発者）が、複数のAWSアカウントを横断してログ管理を行う際の手間・事故を減らしたい
B. 自分以外の社内チームメンバーも将来使う可能性がある
C. Not identified
X. Other (please specify)

[Answer]: A. 社内の自分自身（単独開発者）が、複数のAWSアカウントを横断してログ管理を行う際の手間・事故を減らしたい

## Q3. 成功とは何を意味しますか？どの指標が重要ですか？

PRD記載の成功指標: (1) Org横断でのログ調査から削除実行までを1つのツール内で完結できること、(2) 削除操作で意図しないアカウント・ログループを誤って削除した事故がゼロであること（監査ログで追跡可能）、(3) `--output json` でスキャン結果をCLIエージェント（Cursor/Claude Code）から取得できること。

A. 上記3点の通りで正しい
B. 追加/変更したい指標がある（自由記述で補足する）
C. Not yet defined
X. Other (please specify)

[Answer]: A. 上記3点の通りで正しい

## Q4. この取り組みのトリガーは何ですか（市場圧力・技術的負債・規制・機会）？

A. 技術的負債・運用事故（AWS_PROFILEの取り違え、ページネーション未対応による集計誤り）を構造的に解消したいという内発的動機
B. Free Tier超過の請求警告という直接のインシデント対応がきっかけ
C. 両方（AとBの組み合わせ）
D. Not identified
X. Other (please specify)

[Answer]: C. 両方（AとBの組み合わせ）

## Q5. 主要なステークホルダーは誰で、それぞれ何を重視しますか？

A. 開発者本人（作成者兼利用者）のみ — 実装の正確性・安全性・保守コストの低さを重視
B. 開発者本人に加えて、将来的に他のチームメンバーも利用者になり得る
C. Not identified
X. Other (please specify)

[Answer]: A. 開発者本人（作成者兼利用者）のみ — 実装の正確性・安全性・保守コストの低さを重視

## Q6. スコープや優先順位を決定するのは誰ですか？また誰が影響力を持ちますか？

A. 開発者本人が単独で決定する
B. Not identified
X. Other (please specify)

[Answer]: A. 開発者本人が単独で決定する

## Q7. コミュニケーション要件や報告のケイデンスはありますか？

A. なし（単独開発、外部への定期報告は不要）
B. None
X. Other (please specify)

[Answer]: A. なし（単独開発、外部への定期報告は不要）

## Q8. このワークフローは `cwsweep-orgwide-logs-cleanup` スコープ（13/33ステージ実行、Standard深度）で開始されました。これはあなたの意図するプロダクト境界と一致しますか？

A. 一致する。このスコープのまま進める
B. 一致しない。境界を変更したい（自由記述で補足する）
X. Other (please specify)

[Answer]: A. 一致する。このスコープのまま進める

---

Guided / Self-guided / Chat: PRDの内容に基づき、各質問に最も自然な選択肢をデフォルトで記入済みです（Guide Meモード）。このまま確定して良ければ「Looks correct」、修正したい項目があれば該当箇所を書き換えるか、自由記述で教えてください（Self-guided/Chatどちらでも構いません）。

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
