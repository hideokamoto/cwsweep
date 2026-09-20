# Unit Test Instructions — Unit: release-docs

## テストフレームワーク

該当なし（ドキュメント・メタデータのみ）。自動テストは追加しない。

## 本Unitのテスト実行コマンド（厳密スコープ）

```bash
cargo build --locked                      # Cargo.toml / Cargo.lock の整合
grep -c -- '--scan-only' README.md        # 期待値 1（移行表の行のみ）
cargo run -q -- --version                 # cwsweep 0.2.0
```

## 期待されるテストケース

| 確認 | 期待 |
|---|---|
| バージョン | `cwsweep 0.2.0` |
| README 旧フラグ例 | 移行表以外に存在しない |
| README 安全機構 | dry-run 既定 / 監査ログ必須 / 二重 Identity 検証 / 管理アカウント非 AssumeRole / 終了コード方針 が全件残存 |

## カバレッジ目標

該当なし（lib カバレッジ floor は cli-foundation / build-and-test で検証）。

## モック/スタブ方針

該当なし。
