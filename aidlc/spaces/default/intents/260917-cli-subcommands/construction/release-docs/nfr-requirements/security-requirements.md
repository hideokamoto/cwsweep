# security-requirements.md — Unit: release-docs

release-docs はドキュメント（README.md / CHANGELOG.md）と `Cargo.toml` のバージョン
メタデータのみを変更する standalone Unit である。実行可能物には寄与しないため、
本Unitに固有のセキュリティ実装要件はなく、Inception NFR5（既存NFRの継続）と
NFR4（後方互換の意図的放棄）を「文書が安全機構を正しく伝えること」として具体化する。

## NFR4: 後方互換の意図的放棄（文書側）

### NFR4.1 移行ガイドの完全性

- **要件**: README.md と CHANGELOG.md に旧フラグ → 新サブコマンドの対応表を掲載し、
  旧形式（`cwsweep --regions ... [--execute|--scan-only]`）が互換エイリアスなしに
  エラーとなることを明示する。
- **脅威モデル（STRIDE: 否認 / 誤操作）**: 旧 README のまま `--execute` の位置を誤解した
  利用者が、意図しない形で clean を実行する。
- **検証**: README に旧フラグ形式のコマンド例が残っていないこと（`grep -- '--scan-only'`
  が 0 件）、対応表に `--scan-only` / トップレベル `--execute` の行があること。
- **出典**: FR6.1、FR6.2、NFR4。

## NFR5: 既存セキュリティ機構の継続（文書側）

### NFR5.1 安全機構の記述維持

- **要件**: dry-run 既定（`clean` は `--execute` 明示時のみ破壊的 API を呼ぶ）、
  監査ログ必須（無効化オプションなし）、二重 Identity 検証、管理アカウントへの
  AssumeRole 禁止、`--regions` 必須・全リージョン列挙なし、終了コード方針の各記述を
  サブコマンド体系に書き換えたうえで維持する。
- **検証**: README の各項目がサブコマンド名（`scan` / `clean` / `audit`）を伴って
  記述されていること。
- **出典**: NFR5、project.md Mandated / Forbidden。

### NFR5.2 audit サブコマンドの読み取り専用性の明示

- **要件**: `audit` が監査ログを一切書き換えず、AWS へも接続しないことを README に明記する。
- **出典**: FR4.5、FR4.6、NFR2。

### NFR5.3 バージョンメタデータ

- **要件**: `Cargo.toml` の `version` を `0.2.0` にし、`Cargo.lock` を `--locked` 検証が
  通る状態に更新する。新規依存は追加しない（`cargo audit` / `cargo deny` ゲート不変）。
- **出典**: FR6.3、NFR5。
