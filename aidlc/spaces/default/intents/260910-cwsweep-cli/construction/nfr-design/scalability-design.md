# Scalability Design

## スケーリング方針

横スケール/縦スケールの概念は適用しない（単一プロセスのCLIバイナリとして実行される）。

## データ保持

アカウント×リージョンの組み合わせをイテレーション単位とし、`ScanAggregator`内で`Vec<LogGroupRecord>`としてメモリ上に保持する（NFR3.3の規模想定：1アカウント×1リージョンあたり数百件）。

## キャッシュ・シャーディング

不要と判断（NFR3.1〜3.3の規模想定に対し導入コストが見合わない）。

## Assumptions & Open Questions

None.
