---
title: ログ出力とトラブルシューティング
description: エラー表示・詳細ログの出し方と、よくあるエラーへの対処。
sidebar:
  order: 5
---

## ログ出力・エラー表示

- 致命的エラーが発生した場合、標準エラーへ `Error: <メッセージ>` を赤文字（TTY接続時のみ。パイプ/リダイレクト時は色なし）の1行だけで表示します。AWS SDK内部のエラー型構造のダンプ等は表示しません。
- `--verbose`（`--debug` でも同義）を指定すると、AWS SDK / 内部クレートが出す詳細ログ（認証プロバイダの解決過程など）を標準エラーへそのまま出力します。指定しない場合、これらの生ログは抑制され、`cwsweep` 自身の警告のみが表示されます。`--verbose` / `--debug` はサブコマンドの前後どちらに置いても有効です（例: `cwsweep scan --verbose` / `cwsweep --debug clean`）。
- `RUST_LOG` 環境変数が設定されている場合は、`--verbose` の有無に関わらずそちらを優先します。

```bash
# 認証エラーの詳細を確認したい場合
cwsweep scan --regions us-west-2 --verbose
```

## よくあるエラー

### `WARN profile [xxx] ignored; sections ... must have a prefix i.e. [profile my-profile]`

`~/.aws/config` のセクション名が `[xxx]` になっています。AWS SDK は `default` 以外のプロファイルに `profile` プレフィックスを要求するため、`[profile xxx]` に書き換えてください（`~/.aws/credentials` 側は `[xxx]` のままで構いません）。

### `ProfileFile provider could not be built: ... the credentials-login feature must be enabled`

`aws login` で作成した `login_session_arn` 付きプロファイルは `aws-config` の`credentials-login` フィーチャが必要です。現行のソースでは有効化済みなので、このエラーが出る場合は古いバイナリを使っています。最新版を再インストールしてください。
