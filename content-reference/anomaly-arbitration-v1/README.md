# Anomaly Arbitration normalized reference

This directory is the deterministic, reviewable staging surface for Goal 13.
It is generated from the frozen Version 4.4 manifest and released evidence; it
is not a runtime input and no row claims runtime executability.

`G13-P1-B1` owns `profiles.json`, `periods.json`, `stages.json` and
`terminal-outcomes.json`. Regenerate them with:

```text
node tools/anomaly-arbitration-reference/import-profile.mjs \
  --source-cache .cache/content-reference \
  --fallback-source-cache /Users/mikai/.codex/worktrees/7c74/starclock/.cache/content-reference
```

Every normalized row carries bilingual names and independent summaries,
manifest obligation IDs, exact source locators and evidence/mechanism quality.
Field-level uncertainty retains alternatives, affected fixture IDs and a
stronger-evidence replacement condition.
