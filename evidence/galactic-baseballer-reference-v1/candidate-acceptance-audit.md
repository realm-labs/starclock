# Goal 16 Candidate Acceptance Audit

`G16-P4-B3` completes the source-cache and clean-checkout acceptance boundary.
The machine record is `candidate-acceptance-results.json` with SHA-256
`1e931218094eacb230cd6fdbc9ac3d49aae1d711855ab01e3149f95d9b7fd009`.

## Full source-cache gate

The current Goal 16 worktree ran:

```text
node tools/repository-check/run.mjs --full --with-source-cache
```

The gate passed all 32 generated/source checks with zero cache-dependent
skips, immutable release snapshots, the merged Candidate integration audit,
Clippy with warnings denied, and all 138 workspace test harnesses. Workspace
tests took 78.8 seconds and the complete gate took 123.0 seconds.

Both source repositories remained clean at the fixed revisions:

- `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93`.

## Goal-specific Candidate verifier

`verify-candidate.mjs` reproduces the Goal-owned chain in dependency order. It
regenerates all Departure facts before its semantic fixtures, verifies every
Demon King fragment against the fixed source cache, assembles the 40-file pack,
executes semantic and boundary audits, independently authors and byte-compares
all four workbooks, regenerates the complete Sora release, and loads all 40
tables and 10,615 rows through the isolated reader.

The first end-to-end run exposed a real generator-order hazard: Departure
fixtures were generated before Departure encounters and could observe the
previous combined pack's Demon King first row. The verifier now fixes the
authoritative order as facts → encounters/progression → fixtures → combined
pack. A fresh complete run then passed with no normalized, workbook or Sora
drift.

## Clean-checkout acceptance

A detached worktree was created from the already pushed P4-B2 artifact commit:

| Field | Value |
|---|---|
| Commit | `ccc0c108f8fbdee5940dce63fb2676258c7dd613` |
| Tree | `fad9760201233b849e428424e6ed4ca5f4dd71bf` |
| Source/tool access | ignored links to fixed read-only source and tool caches |
| Full generated/source checks | 32 passed; 0 skipped |
| Clippy | passed |
| Workspace harnesses | 138 passed in 203.7 seconds |
| Complete full gate | passed in 316.1 seconds |
| Goal reference/semantic/workbook/Sora checks | all passed |
| Standalone reader | 40 tables / 10,615 rows / 0 empty |
| Tracked status after checks | clean |

The temporary worktree was removed through Git after the acceptance record was
captured. The main checkout was never entered or modified.

## Reproduction

```text
fnm exec --using 24.15.0 node \
  tools/galactic-baseballer-reference/verify-candidate.mjs \
  --source-cache .cache/galactic-baseballer-source \
  --python /Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3

node tools/repository-check/run.mjs --full --with-source-cache
```

For a detached final checkout, add `--allow-detached --require-clean` to the
Goal verifier. The terminal Candidate freeze reruns that exact mode against
the pushed P4-B3 commit.
