# Anomaly Arbitration source inventory

`source-inventory.json` is the deterministic Version 4.4 **file closure** for
Goal 13. It retains all 2,646 Goal 03 source files for later shared-pool
reachability audits and adds the focused `ChallengePeak` tables, shared source
tables, TextMaps, StageConfig, enemy closure, mechanical configuration and
released bilingual indexes needed by Anomaly Arbitration research.

This inventory does not admit an active-period row, prove mode ownership or
freeze a content denominator. Group `8`, aliases `801`–`804` and the five stage
IDs are planning selectors only until `G13-P0-B3` proves their explicit
reference chain. Prefixes, nearby IDs and matching names remain
non-authoritative.

The generator hashes raw checked-out bytes only after proving that their Git
blob OID matches the pinned tree. A second clean cache at the same fixed
revision may be supplied as a read-only Git object alternate when a partial
clone has not materialized a required blob:

```text
node tools/anomaly-arbitration-reference/inventory.mjs \
  --source-cache .cache/content-reference \
  --fallback-source-cache <clean-fixed-cache>

node tools/anomaly-arbitration-reference/verify-inventory.mjs \
  --source-cache .cache/content-reference \
  --fallback-source-cache <clean-fixed-cache>
```

Both cache roots must be clean and resolve to the exact pinned commits. The
fallback is read-only; the generator disables lazy fetches and never changes
sparse-checkout state.

`content-manifest.json` is the `G13-P0-B3` row-level closure. Released public
observations identify Version 4.4 rotation `8` and its bilingual title; the
pinned group row then selects aliases `801`–`804`, five StageConfig rows and
every shared reference transitively admitted by those rows. Generate and
verify it with:

```text
node tools/anomaly-arbitration-reference/manifest.mjs \
  --source-cache .cache/content-reference \
  --fallback-source-cache <clean-fixed-cache>

node tools/anomaly-arbitration-reference/verify-manifest.mjs \
  --source-cache .cache/content-reference \
  --fallback-source-cache <clean-fixed-cache>
```

The manifest freezes 392 active obligations across 28 categories, split into
76 `AnomalyArbitration` and 316 `Shared` records. It separately receipts 106
historical/account/presentation exclusions and proves six gameplay content
pools empty. Every active shared row records a source path, stable row locator,
evidence digest and explicit selector/reference summary.

The P0-B4 machine contracts are:

- `normalized-schema.json`: 37 normalized files, bilingual/provenance
  envelope, canonical encoding, field-level approximation and reconciliation
  receipt shape;
- `authoring-contract.json`: the three complete mode-owned workbooks,
  openpyxl/Sora authority, isolated paths and deterministic no-overwrite
  acceptance; and
- `fixture-contract.json`: 18 non-shrinking semantic families, including six
  separate empty-pool cases.

Verify all three against the frozen manifest with:

```text
node tools/anomaly-arbitration-reference/verify-contracts.mjs
```
