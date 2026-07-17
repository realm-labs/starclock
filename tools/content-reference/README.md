# Content Reference Bootstrap

The bootstrap reproduces the committed Version 4.4 reference pack from pinned
released-data caches. Node.js uses only built-in modules.

Fetch the ignored source cache:

```powershell
pwsh -File tools/content-reference/fetch-sources.ps1
```

Regenerate the pack:

```powershell
node --max-old-space-size=4096 tools/content-reference/bootstrap.mjs `
  --repo-root . `
  --turn-data .cache/content-reference/turnbasedgamedata `
  --starrail-res .cache/content-reference/StarRailRes `
  --out content-reference/v4.4
```

The generator:

- preserves 64-bit text hashes before JSON parsing;
- converts gameplay decimals to canonical strings;
- merges Trailblazer body variants by Path;
- uses descriptive Starclock keys and retains source IDs only as locators;
- merges released 4.4 facts with the pinned Saber/Archer 4.3 fallback;
- derives target/operation tags without emitting long source descriptions;
- hashes every referenced source file and generated output.

Review `git diff -- content-reference/v4.4` after every regeneration. Generated
drift is a source/schema migration and must not be accepted as a formatting-only
change.

Verify counts, references, evidence boundaries, and the pack digest:

```powershell
node tools/content-reference/verify.mjs content-reference/v4.4
```
