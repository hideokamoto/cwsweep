---
name: cwsweep
description: AWS Organization 全体の CloudWatch Logs ロググループを cwsweep CLI で棚卸しし、不要なロググループの削除や保持期間(retention)の一括変更を安全に進める手順。「Organization 内の全アカウントのロググループを一覧したい」「使っていないロググループを消したい」「ログの保持期間を揃えたい」「CloudWatch Logs のコストを減らしたい」といった依頼や、cwsweep コマンドの実行・監査ログの確認を頼まれたときに使う。
compatibility: cwsweep コマンドと、AWS Organizations の管理アカウントの認証情報が必要。clean と retention set の本実行は対話可能な端末(TTY)でのみ動く。
---

# cwsweep: Organization 横断の CloudWatch Logs 棚卸しと整理

cwsweep は、AWS Organization の管理アカウントの認証情報で各メンバーアカウントへ AssumeRole し、CloudWatch Logs のロググループを一覧・削除・retention 変更する CLI。ドキュメント: https://cwsweep.hidetaka.dev/

## 安全ルール(必ず守る)

1. **最初に必ず `scan` を実行する。** `scan` は読み取り専用で、削除も retention 変更もせず、監査ログも作らない。
2. **`--execute` は自分では付けない。** `--execute` を付けない実行(dry-run)で対象と影響を確認し、利用者に「何を・どのアカウント/リージョンで・何件」実行するかを見せて、明示的な承認を得てから本実行する。
3. **削除は取り消せない。** AWS の DeleteLogGroup は、ロググループとそのログイベントを完全に削除する(出典: https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_DeleteLogGroup.html)。迷うなら削除ではなく `retention set` を提案する。
4. **対象リージョンを推測しない。** `--regions` は利用者が決める。`--regions all` を使う前に、対象が全リージョンであることを利用者に確認する。
5. **本実行は利用者の端末で行ってもらう。** `clean` は対話式の選択・確認を必須とし、`retention set --execute` も標準入力が TTY でないとエラーになる。エージェントの実行環境に TTY がなければ、実行するコマンドを利用者に渡し、利用者の端末で実行してもらう。
6. 認証情報(アクセスキー・セッショントークン)をログや出力に貼らない。cwsweep 自身もこれらを出力しない。

## 前提条件

- `cwsweep --version` が動く。インストールは https://github.com/hideokamoto/cwsweep/releases から対応するバイナリを取得する(Linux / macOS、x86_64 / arm64)。ソースからは Rust 1.94.1 以上で `cargo install --git https://github.com/hideokamoto/cwsweep --locked`。
- **Organization の管理アカウントの認証情報**で実行する(AWS SDK の既定の認証情報解決チェーンで解決される)。管理アカウント自身には AssumeRole せず、その認証情報をそのまま使う。Organization のメンバーでないアカウントで実行した場合は、そのアカウントだけが対象になる。
- メンバーアカウントには、管理アカウントから AssumeRole できるロールが必要。既定のロール名は `OrganizationAccountAccessRole`。このロールは Organizations 経由で作成したアカウントに自動で作られるが、招待で参加したアカウントには自分で作る必要がある(出典: https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_accounts_access.html)。名前が違う場合は `--role-name` で指定する。
- 呼び出し元の認証情報に必要な権限は、次の API を呼べること: Organizations の ListAccounts(管理アカウントまたは委任管理者からのみ呼べる。出典: https://docs.aws.amazon.com/organizations/latest/APIReference/API_ListAccounts.html)、各メンバーアカウントのロールへの sts:AssumeRole、`--regions all` または `--regions` 省略時の account の ListRegions。AssumeRole 先のロールには CloudWatch Logs の DescribeLogGroups(`clean` の削除は DeleteLogGroup、retention 変更は PutRetentionPolicy も)が必要。GetCallerIdentity は権限不要(出典: https://docs.aws.amazon.com/STS/latest/APIReference/API_GetCallerIdentity.html)。
- 対象は AWS 標準パーティション(商用リージョン)のみ。GovCloud と中国リージョンは対象外。

## 手順

### 1. 棚卸し(読み取り専用)

```bash
cwsweep scan --regions us-east-1,ap-northeast-1 --output json
```

- `--output json` の出力は `total_bytes`、`log_group_count`、`records[]`(アカウントID・リージョン・ロググループ名・保存バイト数・retention 日数)を持つ。表で見せたいときは `--output table`(既定)。
- `--regions` に `all` を指定すると、`account:ListRegions` で有効化済みの商用リージョンすべてが対象になる。`all` は他の値と一緒に指定しても `all` が優先される。
- `--regions` を省略すると、TTY では対話式の選択になり、TTY でなければ AWS へ接続せずエラーで終了する。エージェントからは必ず `--regions` を明示する。
- 一部のアカウント×リージョンで失敗しても、1件でも成功すれば終了コード0で、失敗分は標準エラーに警告が出る。全件失敗すると終了コード1。警告が出ていたら、出力が完全な棚卸しではないことを利用者に伝える。
- 結果を `total_bytes` の大きい順などに整理して利用者に見せ、削除候補・retention 変更候補を一緒に決める。cwsweep は削除条件の自動判定をしない。対象は人が選ぶ。

### 2a. retention を一括変更する場合

```bash
# dry-run(既定)。AWS API は呼ばれない
cwsweep retention set --regions us-east-1 --days 30

# 本実行(利用者の承認後、利用者の端末で)
cwsweep retention set --regions us-east-1 --days 30 --execute
```

- `--days` は必須で、CloudWatch Logs が許容する値だけ使える: 1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, 3653(出典: https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_PutRetentionPolicy.html)。それ以外は AWS へ接続する前にエラーになる。
- 対話式の選択はなく、スキャンで見つかった**全ロググループ**が対象になる。一部だけ変えたいときは `clean` を使う。
- TTY でない環境では dry-run だけ実行できる。`--execute` を付けるとエラーで終了する。

### 2b. ロググループを選んで削除/retention 変更する場合

```bash
# dry-run(既定)。選択・確認まで進むが、削除も retention 変更もされない
cwsweep clean --regions us-east-1

# 本実行(利用者の承認後、利用者の端末で)
cwsweep clean --regions us-east-1 --execute
```

- 流れ: スキャン結果の表示 → マルチセレクトで対象を選ぶ(初期状態は全てOFF) → アクション(delete または set-retention)を選ぶ → 対象と合計バイト数の確認画面(既定は No) → `--execute` のときだけ実行。
- 標準入力が TTY でないと、スキャン結果を表示して警告を出し、何も実行せず終了コード0で終了する。これは失敗ではない。非対話環境の棚卸しには `scan` を使う。

### 3. 結果を監査ログで確認する

```bash
cwsweep audit --output json
```

- `clean` / `retention set` は、`--execute` のときだけ、API 呼び出しの前に `event: "intent"`、後に `event: "result"` の2行を JSON Lines で追記する(既定のパスはカレントディレクトリの `cwsweep-audit.jsonl`、`--audit-log-path` で変更できる)。dry-run では何も書かれない。
- `intent` 行の `success` は結果が未確定のプレースホルダなので、成否は `result` 行の `success` と `error_message` で判断する。
- `audit` は AWS に接続せず、ログを書き換えない。ファイルがなければ空の結果で終了する。解釈できない行は警告を出してスキップされる。

## よくあるエラー

- `--regions was not specified`: TTY でない環境で `--regions` を省略した。`--regions` を明示する。
- AssumeRole の失敗(AccessDenied など): 対象メンバーアカウントに、指定したロール名のロールがない可能性がある(招待で参加したアカウントなど)。`--role-name` を確認する。
- アカウントIDの不一致: 認証情報の取得先が想定のアカウントではない。処理は失敗として扱われる。認証情報を確認する。
- 認証情報や API の詳細を調べたいときは `--verbose`(`--debug` も同じ)を付ける。環境変数 `RUST_LOG` があればそちらが優先される。
