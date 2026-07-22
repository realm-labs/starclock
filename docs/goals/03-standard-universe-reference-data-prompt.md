# Goal 03 Launch Prompt

```text
Create or resume the persistent goal whose objective is to complete Starclock
Goal 03: prepare the complete Version 4.4 Standard Simulated Universe reference
pack, Excel/Sora authoring data, provenance, coverage and review fixtures before
runtime implementation. Continue batch-by-batch until every terminal gate is
proved. Do not set a token budget.

Read completely before acting:

- docs/goals/03-standard-universe-reference-data.md
- docs/goals/03-standard-universe-reference-data-status.md
- docs/23-standard-simulated-universe-reference.md
- docs/07-configuration-pipeline.md
- docs/08-engineering-standards.md
- docs/09-determinism-and-numerics.md
- docs/11-rule-ir-and-native-handlers.md
- docs/14-run-core-and-universe-modes.md
- docs/15-content-data-and-coverage.md
- docs/19-activity-core-and-mode-extension.md
- docs/content-reference/README.md
- docs/content-reference/schema.md
- docs/content-reference/authoring-contract.md
- docs/sources.md
- content-reference/README.md
- tools/content-reference/README.md

Execution loop:

1. Inspect the clean worktree, current goal and full ledger.
2. Select the earliest unblocked Pending batch and mark only it InProgress.
3. Implement its source inventory, normalized data, schema, workbook, evidence,
   tests and documentation as one responsibility-bounded change.
4. Use only released/public evidence. Record exact URLs/revisions/paths/hashes;
   preserve gaps as field-level Approximate or ProjectPolicy decisions.
5. Use Python openpyxl for workbook creation and inspection. Regenerate a full
   initial workbook; do not patch designer-edited workbooks. Sora 0.3.0 remains
   the validation/codegen/export authority. Never add JSON runtime loading.
6. Run batch gates and universal repository/prior-release checks. Update the
   ledger with exact commands, digests, counts and decisions.
7. Commit exactly one completed batch using Conventional Commits and the exact
   batch ID, for example:
   data(universe): G03-P1-B3 import main-world blessings
8. Immediately continue with the next unblocked batch. Do not stop at a phase,
   workbook export or partial coverage.

Completeness comes from frozen manifests, not estimated Wiki counts. Shared
tables require explicit main-world reachability proof. Exclude DLC-only mode
mechanics, rewards, story and assets. Do not mark the goal complete until all
required rows are DataReady, Excel/Sora regenerates without drift, every sheet
has passed visual and structural QA, semantic fixtures cover all mechanic
families, clean-checkout acceptance passes and G03-P4-B4 is committed cleanly.
```
