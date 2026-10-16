---
title: scan — 棚卸し
description: ロググループを読み取り専用で棚卸しする。
sidebar:
  order: 1
---

ロググループを表形式で表示して終了します。対話式の選択・確認へは進まず、監査ログも書きません。

```bash
cwsweep scan --regions us-east-1,us-west-2

# 出力フォーマットをJSONに切り替える（Cursor/Claude Code等のCLIエージェント向け）
cwsweep scan --regions us-east-1 --output json

# メンバーアカウントへAssumeRoleする際のロール名を上書きする（既定: OrganizationAccountAccessRole）
cwsweep scan --regions us-east-1 --role-name CustomOrgRole

# 商用パーティションの全リージョンを対象にする（大文字小文字は区別しない）。
# 対象リージョン数を標準エラーへ表示し、TTYでは実行前に確認を求める
cwsweep scan --regions all

# --regions を省略すると、TTYでは account:ListRegions の結果からマルチセレクトで選択する。
# 非TTY（CI/パイプ）ではAWSへ接続せずエラー終了する
cwsweep scan
```

## リージョンの指定方法

| 指定 | 挙動 |
|---|---|
| `--regions us-east-1,ap-northeast-1` | 指定したリージョンのみを対象にする |
| `--regions all`（`ALL` / `All` も可） | 管理アカウントの資格情報で `account:ListRegions` を呼び、商用パーティション（`aws`）で有効化済みの全リージョンを対象にする。GovCloud・中国リージョン、未有効化のオプトインリージョンは除外。`all` を他の値と併用した場合は `all` が優先される。件数と一覧を標準エラーへ表示し、TTYでは確認（既定: No）を取る |
| 未指定・TTY | 同じ列挙結果から対話式マルチセレクトで選択する。0件選択はエラー |
| 未指定・非TTY | 「`--regions was not specified. In non-interactive environments such as CI or pipes, specify --regions <region>[,<region>...] or --regions all.`」というエラーで終了する（AWSへは接続しない） |

リージョンの指定方法は `clean` / `retention set` でも共通です。
