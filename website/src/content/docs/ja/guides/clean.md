---
title: clean — 選択して削除・retention変更
description: 対話式に選択したロググループを削除・retention変更する。
sidebar:
  order: 2
---

スキャン → 対話式選択 → 確認の順に進みます。**既定は dry-run** で、`--execute` を明示的に渡さない限り削除・retention変更は一切実行されません。

```bash
# dry-run（既定）。スキャン → 対話式選択 → 確認までは進むが、削除・retention変更は一切実行されない
cwsweep clean --regions us-east-1

# 実際に削除・retention変更を実行する（このフラグを明示的に渡さない限り絶対に実行されない）
cwsweep clean --regions us-east-1 --execute

# 監査ログの出力先を上書きする（既定: カレントディレクトリ直下 cwsweep-audit.jsonl。
# 無効化するオプションは存在しない）
cwsweep clean --regions us-east-1 --audit-log-path /var/log/cwsweep/audit.jsonl
```

- 対話式マルチセレクトの初期状態は常に全チェックOFFです。
- 実行前に、対象アカウントID・リージョン・ログループ名一覧・合計バイト数を再掲した確認画面を表示します。
- 標準入力がTTYでない場合（パイプ・CI等）、`clean` はスキャン結果を表示したうえで「対話式選択に進めない」警告を出して正常終了します。非対話環境での棚卸しには[`scan`](/ja/guides/scan/) を使ってください。

安全機構の全体は [安全機構](/ja/reference/safety/) を参照してください。
