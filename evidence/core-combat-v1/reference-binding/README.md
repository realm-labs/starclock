# Frozen reference binding evidence

This directory proves the Goal 01 Phase 0 evidence closure. It maps every
required frozen character form, Light Cone, Standard enemy, encounter, scenario
and profile to the prepared reference pack without copying source descriptions.

- `provenance-map.json` is the one-to-one goal-entry map and required record
  closure.
- `source-cache-report.json` records pinned revisions, the complete source-file
  hash verification and the file-for-file bootstrap regeneration result.
- `saber-archer-audit.json` records why Saber and Archer retain explicit
  `ExactPreviousRelease` provenance.
- `evidence-index.json` hashes the generated reports.

These files are deterministic bootstrap evidence only. They do not authorize a
JSON-direct runtime path; production facts must be authored in `.xlsx`, exported
with pinned Sora 0.3.0 and loaded through generated readers.
