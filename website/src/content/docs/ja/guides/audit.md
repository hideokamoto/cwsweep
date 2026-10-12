---
title: audit — 監査ログの閲覧
description: 削除・retention変更の監査ログを読み取り専用で表示する。
sidebar:
  order: 4
---

```bash
# 既定パス（cwsweep-audit.jsonl）の監査ログを表形式で全件表示する
cwsweep audit

# パスと出力フォーマットを指定する
cwsweep audit --audit-log-path /var/log/cwsweep/audit.jsonl --output json
```

`audit` はAWSに接続せず、監査ログを一切書き換えません。ファイルが存在しない場合は空の結果として正常終了し、フォーマット不正な行は標準エラーへ警告を出してスキップします（前後の正常なエントリは表示されます）。
