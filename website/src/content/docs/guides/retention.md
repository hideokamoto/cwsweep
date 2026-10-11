---
title: retention set — retention の一括設定
description: スキャンで見つかった全ロググループへ retention 日数を一律設定する（削除なし）。
sidebar:
  order: 3
---

```bash
# dry-run（既定）。スキャン → 確認までは進むが、put-retention-policy は一切呼ばれない
cwsweep retention set --regions us-east-1 --days 30

# 実際に retention を設定する（このフラグを明示的に渡さない限り絶対に実行されない）
cwsweep retention set --regions us-east-1 --days 30 --execute

# 監査ログの出力先を上書きする（clean と同じく無効化するオプションは存在しない）
cwsweep retention set --regions all --days 90 --execute --audit-log-path /var/log/cwsweep/audit.jsonl
```

- `--days` は必須です。CloudWatch Logs が受け付ける離散値（1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180,365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, 3653）以外は AWS へ接続する前にエラーになります。
- スキャンで見つかった全ロググループが対象です（対話式選択は行いません）。TTY では実行前に対象一覧を表示して確認を取ります。非TTY（CI 等）では dry-run のみ実行でき、`--execute` を渡すとエラー終了します。
- `delete-log-group` へ到達する経路を持ちません。dry-run 既定・監査ログ必須記録・二重 Identity 検証は`clean` と同じ安全機構を流用します。
