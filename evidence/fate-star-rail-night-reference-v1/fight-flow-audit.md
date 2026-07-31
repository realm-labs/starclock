# Goal 19 Fight Flow and Evidence Boundary Audit

`G19-P1-B7` normalizes 418 records at digest
`4f62f92e3a9597efac6b85a526c58994229d85fe74e7ddb608ca990c7514768b`.
Seven monster-pool rows are enabled. The remaining 411 records are explicitly
evidence-only: 22 broadcasts, 329 Master-talk rows, three display rows, six
mission locators, five resident-reward locators, thirteen day-talk locators
and 33 layout-only AI/ability/event files.

This closes Phase 1 at 1,805 direct records: 1,392 enabled mode rows and all
413 manifest evidence-only obligations. Together with the six exact-zero
records, the 1,398 Fate-owned denominator is accounted exactly. The remaining
93 manifest obligations are shared Stage/BattleArea/enemy/battle-rule closure
owned by Phase 2.

No dialogue, reward payload, asset path or upstream program is copied into the
mechanical pack. Evidence-only locators may prove unlock/order boundaries but
cannot become mechanics without a typed selector and reconciliation decision.

```text
node --max-old-space-size=4096 tools/fate-star-rail-night-reference/normalize.mjs \
  --source-cache .cache/fate-star-rail-night-sources --batch G19-P1-B7 --check
```
