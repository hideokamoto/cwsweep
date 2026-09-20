# scalability-design.md — Unit: cli-foundation

- **D-SCALE-1（NFR5.7）**: アカウント × リージョンの逐次走査とバルクヘッド分離は
  既存 `scan_all` に閉じており、本Unitは呼び出し側として再利用する。並行化は非目標。
- **D-SCALE-2（NFR5.8）**: `--regions` は `Vec<String>` のまま、上限なし、
  `dedup_regions` の初出順 O(n) 除去を `Scan` / `Clean` 両バリアントに適用する。
- **D-SCALE-3（NFR5.9）**: `audit` は `AuditReadOutcome.entries` を全件メモリ保持して
  整形する。ストリーミングは非目標（Contract 1）。
