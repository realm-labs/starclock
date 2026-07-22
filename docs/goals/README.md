# Execution Goals

Execution goals translate the normative Starclock design into resumable,
commit-sized implementation work. Each goal package contains:

- a scope and phased execution plan;
- a persistent status/coverage ledger updated in every batch commit;
- a reusable prompt that starts or resumes a persistent execution loop.

## Goal packages

| Goal | Scope | State | Plan | Status | Launch prompt |
|---|---|---|---|---|---|
| Goal 01 — Complete Core Combat and Released Character Content | Core battle, Standard encounters, all released character forms/Traces/Techniques/Eidolons and Light Cones; no universe or challenge modes | Complete | [Plan](01-core-combat-and-content.md) | [Ledger](01-core-combat-and-content-status.md) | [Prompt](01-core-combat-and-content-prompt.md) |
| Goal 02 — Agent Control API and MCP Adapter | Protocol-neutral battle sessions, exact observations/offered actions, replayable external control, local stdio MCP and authorized remote Streamable HTTP | Ready to start | [Plan](02-agent-control-and-mcp.md) | [Ledger](02-agent-control-and-mcp-status.md) | [Prompt](02-agent-control-and-mcp-prompt.md) |

The plan defines what completion means. The ledger is the resumable source of
truth. The prompt must not override either document; it instructs the executor
to follow them until all terminal gates are evidenced.
