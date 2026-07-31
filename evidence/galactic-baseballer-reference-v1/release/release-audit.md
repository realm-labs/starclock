# Goal 16 Candidate Release Audit

`G16-P4-B4` freezes the complete Version 4.4 Galactic Baseballer reference
release. The machine-verifiable release record is `release-evidence.json`,
SHA-256
`703ce4ca9ad3905ba6727903db9bc929c4a87cb9ba616ec1f2e3f798de95aeed`.

## Frozen Candidate

- The original Version 2.2 `galactic-baseballer.departure.v2_2` and Version
  3.3 `galactic-baseballer.demon-king.v3_3` profiles remain independently
  selectable over `galactic-baseballer.shared-base.v1`; Demon King does not
  replace Departure.
- All 2,232 frozen obligations close without denominator reduction: 2,207 are
  `DataReady`, 25 reward/presentation locators are `EvidenceOnly`, and none is
  blocking.
- The 40-file normalized pack contains 10,615 rows, including 2,634 source
  receipts, 12 replaceable approximation boundaries, 26 ReferenceOnly rules
  and 35 review fixtures across all 20 required mechanism families.
- Four `openpyxl==3.1.5` workbooks contain all 40 authoring sheets. Independent
  generations are byte-identical, every schema column is covered by the 147
  rendered review bands, and final human visual disposition is
  `PassedHumanInspection`.
- The isolated Sora 0.3.0 release contains 40 tables, 42 Rust reader files and
  a binary bundle at
  `82e600ee2b8aaaaeada82810f223d4f45e193833eb4151c939a5e51950f78848`.
  The standalone reader loads every table and all 10,615 rows.

## Terminal acceptance

The already pushed `G16-P4-B3` commit
`0d60989ea540aa2dec5bbb05789f4f57e9f6a1fe` was checked out as a detached
worktree with tree
`ea0bc915ce67c54d121f72043c0cbab86b4ad280`.

In that clean checkout:

- the ordered Candidate verifier regenerated the two fixed sources, both
  profiles, the complete normalized pack, semantic results, all workbooks and
  the Sora release without drift;
- reference boundaries, shared identities, the synthesis DAG and protected
  roots passed;
- `node tools/repository-check/run.mjs --full --with-source-cache` passed all
  32 generated/source checks with zero skip, format, full-feature Clippy and
  138 workspace test harnesses;
- workspace tests took 204.7 seconds and the complete full gate took 310.6
  seconds; and
- tracked Git status remained clean after every check.

The temporary detached worktree was removed after recording these facts. The
main checkout at `/Users/mikai/CLionProjects/starclock` was never modified.

## Publication boundary

All 19 prerequisite batch commits are pinned in `release-evidence.json` and
were pushed and remotely verified before the next batch began. The terminal
`G16-P4-B4` commit is the release record's containing commit. After it is
pushed, local `HEAD`, the tracking ref and
`git ls-remote origin refs/heads/codex/goal16-galactic-baseballer-reference`
must be identical.

This is Candidate reference data only. Runtime loading, Activity/combat
handlers, shared formulas, CLI, Agent API, MCP and a playable profile remain
unreleased and belong to a later goal.
