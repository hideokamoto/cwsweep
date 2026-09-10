# cwsweep

AWS Organization全体のCloudWatch Logsを棚卸しし、対話式に選択したロググループを
安全に削除・retention変更するRust製CLI。

## 使い方

```bash
# スキャンのみ（dry-run既定、削除・retention変更は一切実行されない）
cwsweep --regions us-east-1,us-west-2

# 出力フォーマットをJSONに切り替える（Cursor/Claude Code等のCLIエージェント向け）
cwsweep --regions us-east-1 --output json

# メンバーアカウントへAssumeRoleする際のロール名を上書きする（既定: OrganizationAccountAccessRole）
cwsweep --regions us-east-1 --role-name CustomOrgRole

# 実際に削除・retention変更を実行する（このフラグを明示的に渡さない限り絶対に実行されない）
cwsweep --regions us-east-1 --execute

# 監査ログの出力先を上書きする（既定値は後方互換のためカレントディレクトリ直下
# cwsweep-audit.jsonl のまま。無効化するオプションは存在しない）
cwsweep --regions us-east-1 --audit-log-path /var/log/cwsweep/audit.jsonl
```

- `--regions` は必須。全リージョン自動列挙のフォールバックは存在しない（誤って無関係な
  リージョンをスキャンする事故を防ぐため）。
- `--execute` を渡さない限り、`delete-log-group` / `put-retention-policy` は一切呼び出されない
  （dry-run既定）。
- 対話式マルチセレクトの初期状態は常に全チェックOFF。
- 削除・retention変更を伴う操作は、対象アカウントID・リージョン・ログループ名・実行時刻・
  成功/失敗を監査ログ（既定: カレントディレクトリ直下 `cwsweep-audit.jsonl`。
  `--audit-log-path` で出力先を上書き可能）にJSON Linesで必ず記録する
  （無効化オプションなし）。監査ログの書き込みに失敗した場合、当該操作は中断される。
- 管理アカウント自身へはAssumeRoleせず現在の認証情報をそのまま使用し、メンバーアカウントに
  対してのみAssumeRoleする。
- いずれかのAWSアカウントへのAPI呼び出しの直前（スキャン時、および削除・retention変更の実行
  直前の二重目）に `sts:get-caller-identity` でアカウントID一致を検証し、不一致の場合は
  当該アカウントの処理を即座に失敗させる。
- **終了コード方針（CI/自動化向け）**: 対象アカウント×リージョンの組み合わせが1件以上あり、
  かつその全件でスキャンが失敗した場合、`cwsweep` は非ゼロ終了コードで終了する
  （標準エラーへの警告ログに加え、この場合は正常な「削除対象0件」メッセージを表示しない）。
  1件でも成功した組み合わせがあれば、他が失敗していても終了コードは0（正常終了）とし、
  失敗したアカウント/リージョンは個別に警告ログへ出力するのみとする。これにより
  「スキャンが全滅した」場合と「本当に削除対象ロググループが0件だった」場合を、
  呼び出し元が終了コードだけで区別できる。

## スコープ制約

- v1は AWS 標準パーティション（`aws`、商用リージョン）のみを対象とする。
  Identity検証（`sts:get-caller-identity`）用のSTSクライアントはグローバルサービスの
  性質上リージョンを `us-east-1` に固定しており、これは商用パーティション内であれば
  どの `--regions` を指定してもそのまま機能する。AWS GovCloud (US) や中国リージョン
  （`aws-us-gov` / `aws-cn` パーティション）など、非商用パーティションのメンバー
  アカウントに対する動作は v1 のスコープ外であり、動作確認していない。将来的に
  非商用パーティションをサポートする場合は、パーティションごとのSTSエンドポイント
  リージョン解決（例: `aws-us-gov` → `us-gov-west-1`）を別途実装する必要がある。

## 開発

```bash
# フォーマットチェック
cargo fmt --check

# Lint（警告をエラー扱い）
cargo clippy --all-targets -- -D warnings

# 依存関係の脆弱性・ライセンス/サプライチェーンチェック（要 cargo install cargo-audit cargo-deny）
cargo audit
cargo deny check

# ユニットテスト（本stageが生成する全モジュール）
cargo test --lib

# 統合テスト（モックのみ。実AWSに接触するテストは #[ignore] で分離）
cargo test --test scan_select_execute --test audit_log_format

# カバレッジ計測（要 cargo install cargo-llvm-cov、rustup component add llvm-tools-preview）
cargo llvm-cov --lib --summary-only
```

`Cargo.lock` はリポジトリにコミットしており、CIでは `--locked` フラグでビルド・テストする
（team.md）。

## AI-DLC Workflows v2

This repository ships [AI-DLC Workflows v2](https://github.com/awslabs/aidlc-workflows)
(`awslabs/aidlc-workflows`, v2.8.2), configured for both **Claude Code**
(including Claude Code on the web) and **Cursor** (including its cloud/
background agent). Run `/aidlc` in either harness to start or resume a
workflow.

- `.claude/` — Claude Code harness projection.
- `.cursor/` — Cursor harness projection (same tree serves the Cursor IDE,
  the Cursor CLI, and Cursor's cloud/background agent).
- `aidlc/` — shared workspace (method files, workflow state, audit trail);
  identical across both harnesses.
- `AGENTS.md` — ambient project instructions Cursor auto-reads.

Both harness trees were generated unmodified from upstream's own build
pipeline, with one deliberate change: the AWS Bedrock forcing that upstream's
`.claude/settings.json` ships by default has been removed, so the framework
authenticates through the normal Claude Code/Anthropic account path instead —
required for it to run on Claude Code on the web and other non-AWS-
credentialed runners. Nothing else was touched. Full details, exactly what
was changed and how it was verified, and the third-party license: see
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

## License

This repository's own code is licensed under the GNU GPLv3 — see
[`LICENSE`](LICENSE). The vendored AI-DLC Workflows tree
(`.claude/`, `.cursor/`, `aidlc/`, `AGENTS.md`, `.mcp.json`) is
third-party software under the MIT-0 license; see
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) and
[`licenses/aidlc-workflows-LICENSE.txt`](licenses/aidlc-workflows-LICENSE.txt).
