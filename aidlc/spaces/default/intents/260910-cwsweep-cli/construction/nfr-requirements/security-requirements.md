# Security Requirements

| ID | 要件 | 詳細 |
|---|---|---|
| NFR2.1 | クレデンシャル非ログ出力 | AssumeRoleで取得した一時クレデンシャル（アクセスキー・シークレット・セッショントークン）はログ・標準出力・パニックメッセージに一切出力しない。`secrecy`クレートでのマスキングを実装する |
| NFR2.2 | 認証・認可の委譲 | 認証・認可はAWS IAMに完全に委譲し、本ツール独自のユーザー認証機構は持たない |
| NFR2.3 | Identity検証必須 | API呼び出し直前に毎回`sts:get-caller-identity`で実アカウントIDを検証し、不一致時は即座に失敗する（IdentityVerifierコンポーネント、警告のみでの続行は禁止）。「即座に失敗」はそのアカウントの処理のみを打ち切ることを指し、NFR4.2（部分障害時の継続）に従って他アカウントの処理は継続する |
| NFR2.4 | 書き込み操作のdry-run既定 | `delete-log-group`/`put-retention-policy`等の書き込みAPIは`--execute`明示時のみ実行し、既定はdry-run（一覧表示のみ） |
| NFR2.5 | 削除直前の二重Identity検証 | スキャン時点と削除実行直前の両方でアカウントID再検証を行う |
| NFR2.6 | 依存関係の脆弱性スキャン | `cargo audit`をCI必須ゲートとする（出典: Practices Discovery discovered-rules.md） |
| NFR2.7 | 供給網チェック | `cargo deny check`（advisories/licenses/bans/sources）をCI必須ゲートとする（出典: Practices Discovery discovered-rules.md） |
| NFR2.8 | unsafeコード禁止 | `#![forbid(unsafe_code)]`をクレート全体に適用する（出典: Practices Discovery discovered-rules.md） |
| NFR2.9 | 対話式選択の初期状態 | マルチセレクトUIの初期状態は全チェックOFFとし、一括全選択をデフォルトにしない（出典: PRD非機能要件#6） |

## 脅威モデル（概要）

主な脅威は「誤ったアカウント・リージョン・ログループに対する破壊的操作」であり、外部からの攻撃者を主対象とした脅威モデルではない（ローカルCLIとして実行され、ネットワーク越しの攻撃面を持たない）。NFR2.1〜NFR2.5がこの脅威への構造的対策である。

## コンプライアンス

本ツールは個人・チーム利用のOSS的ツールであり、特定の規制（PCI-DSS, HIPAA等）への準拠は要件としない。AWS IAM権限は利用者自身のポリシーに従う。

## Assumptions & Open Questions

None.
