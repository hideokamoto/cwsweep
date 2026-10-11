---
title: スコープ制約
description: cwsweep v1 が対象とする AWS パーティションの範囲。
sidebar:
  order: 2
---

v1 は AWS 標準パーティション（`aws`、商用リージョン）のみを対象とします。

Identity検証（`sts:get-caller-identity`）用のSTSクライアントは、グローバルサービスの性質上リージョンを `us-east-1` に固定しています。商用パーティション内であれば、どの `--regions` を指定してもそのまま機能します。

AWS GovCloud (US) や中国リージョン（`aws-us-gov` / `aws-cn` パーティション）など、非商用パーティションのメンバーアカウントに対する動作は v1 のスコープ外であり、動作確認していません。
