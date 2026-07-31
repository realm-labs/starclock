# Goal 16 Launch Prompt — Galactic Baseballer Reference Data

Use this prompt to start or resume the persistent Goal 16 execution loop.

## Objective

Complete Starclock Goal 16: build the Version 4.4 Candidate reference pack for
both Version 2.2 Legend of the Galactic Baseballer: Departure and Version 3.3
Legend of the Galactic Baseballer: Demon King, with isolated Excel/Sora author
data, released-source evidence, exact-once coverage and executable semantic
review fixtures. Continue batch-by-batch until every terminal gate passes.

## Required reading

Read completely before mutation:

- repository `AGENTS.md`;
- `docs/goals/16-galactic-baseballer-reference-data.md`;
- `docs/goals/16-galactic-baseballer-reference-data-status.md`;
- `docs/goal-16-foundation.md`;
- `policy/goal16-foundation.json`;
- the architecture, determinism, configuration, evidence and authoring
  contracts linked by the plan.

The plan defines completion. The status ledger is the resumable source of
truth. This prompt does not override either document.

## Execution loop

1. Verify the worktree is on
   `codex/goal16-galactic-baseballer-reference`, is separate from the main
   checkout and uses only Goal 16 roots.
2. Verify local `HEAD`, the tracking ref and the remote branch match before
   beginning a new batch.
3. Select the earliest unblocked `Pending` batch and mark only that batch
   `InProgress`.
4. Complete all source, data, evidence, validator, workbook/Sora and ledger
   work owned by the batch.
5. Run focused checks and `node tools/repository-check/run.mjs`; run full
   source-cache/clean gates at the named boundaries.
6. Commit exactly one responsibility-bounded batch with the required `G16`
   Conventional Commit title.
7. Push immediately to `origin/codex/goal16-galactic-baseballer-reference`.
8. Verify local `HEAD`, tracking ref and `git ls-remote` SHA equality. Retry
   transient remote verification failures; do not begin the next batch first.
9. Repeat without waiting for confirmation until `G16-P4-B4` is complete.

## Non-negotiable boundaries

- Use released public evidence only.
- Keep Departure and Demon King as separate profiles over a shared base.
- Do not infer membership, synthesis or inheritance from names or ID ranges.
- Do not reduce frozen denominators.
- Record field-level uncertainty as `ApproximateFromReleasedText` or
  `ProjectPolicy`, never as exact.
- Excel `.xlsx` is the sole editable production authoring surface;
  `openpyxl==3.1.5` authors complete new workbooks and Sora 0.3.0 owns schema,
  code generation and export.
- Do not implement runtime logic or mutate shared/generated mode partitions.
- Do not create a pull request or rewrite history.

## Terminal action

After the final batch is committed, pushed, remotely verified and all terminal
checks pass, update the ledger to `Complete`, freeze Candidate release evidence
and mark the persistent Goal complete.
