# Execution Goals

Execution goals translate the normative Starclock design into resumable,
commit-sized implementation work. Each goal package contains:

- a scope and phased execution plan;
- a persistent status/coverage ledger updated in every batch commit;
- a reusable prompt that starts or resumes a persistent execution loop.

## Active goal packages

| Goal | Scope | State | Plan | Status | Launch prompt |
|---|---|---|---|---|---|
| Goal 01 — Complete Core Combat and Released Character Content | Core battle, Standard encounters, all released character forms/Traces/Techniques/Eidolons and Light Cones; no universe or challenge modes | Ready to start | [Plan](01-core-combat-and-content.md) | [Ledger](01-core-combat-and-content-status.md) | [Prompt](01-core-combat-and-content-prompt.md) |

The plan defines what completion means. The ledger is the resumable source of
truth. The prompt must not override either document; it instructs the executor
to follow them until all terminal gates are evidenced.
