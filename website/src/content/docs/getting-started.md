---
title: はじめに
description: cwsweep のインストールと最初のスキャン。
---

## インストール

[GitHub Releases](https://github.com/hideokamoto/cwsweep/releases) から、使っている環境（Linux / macOS、x86_64 / arm64）向けのバイナリをダウンロードしてください。

ソースからビルドする場合は、Rust ツールチェーンを用意したうえで次を実行します。

```bash
cargo install --git https://github.com/hideokamoto/cwsweep --locked
```

## 前提

- AWS Organizations の**管理アカウント**の認証情報で実行します。管理アカウント自身へはAssumeRole せず、現在の認証情報をそのまま使います。
- メンバーアカウントへは AssumeRole します。既定のロール名は `OrganizationAccountAccessRole` で、`--role-name` で上書きできます。
- 対象は AWS 標準パーティション（`aws`、商用リージョン）のみです。詳しくは[スコープ制約](/reference/scope/) を参照してください。

## 最初のスキャン

まずは読み取り専用の `scan` で棚卸しします。`scan` はロググループを表形式で表示して終了し、削除やretention変更に到達する経路を持ちません。

```bash
cwsweep scan --regions us-east-1,us-west-2
```

## サブコマンド

| サブコマンド | 役割 | AWSへの書き込み |
|---|---|---|
| [`scan`](/guides/scan/) | 棚卸し（表示のみ） | なし |
| [`clean`](/guides/clean/) | 対話式に選択して削除・retention変更 | `--execute` 指定時のみ |
| [`retention set`](/guides/retention/) | 全ロググループへ retention 日数を一律設定 | `--execute` 指定時のみ（削除なし） |
| [`audit`](/guides/audit/) | 監査ログの閲覧 | なし（AWSに接続しない） |

サブコマンドを省略した呼び出しはエラーになります。
