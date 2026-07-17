# Goal provenance evidence tools

`generate.mjs` binds every frozen Goal 01 manifest entry to the prepared 4.4
reference records and their pinned source files. It also verifies all cached
source hashes, compares a full bootstrap regeneration with the committed pack,
and performs the dedicated Saber/Archer previous-release audit.

The source caches and regenerated pack remain ignored local inputs:

```text
.cache/content-reference/turnbasedgamedata
.cache/content-reference/StarRailRes
.cache/content-reference/regenerated-v4.4
```

Generate and verify the committed reports from the repository root:

```sh
node tools/goal-provenance/generate.mjs
node tools/goal-provenance/generate.mjs --check
node tools/goal-provenance/verify.mjs
```

The reports are evidence maps, not a runtime data path or a competing content
staging model. Production content remains subject to the Excel/Sora contract.
