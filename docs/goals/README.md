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
| Goal 02 — Agent Control API and MCP Adapter | Protocol-neutral battle sessions, exact observations/offered actions, replayable external control, local stdio MCP and authorized remote Streamable HTTP | Complete | [Plan](02-agent-control-and-mcp.md) | [Ledger](02-agent-control-and-mcp-status.md) | [Prompt](02-agent-control-and-mcp-prompt.md) |
| Goal 03 — Complete Standard Simulated Universe Reference Data | Version 4.4 main-world manifests, normalized mechanics/provenance, Excel/Sora schemas and complete authoring workbooks; no universe runtime | Complete | [Plan](03-standard-universe-reference-data.md) | [Ledger](03-standard-universe-reference-data-status.md) | [Prompt](03-standard-universe-reference-data-prompt.md) |
| Goal 04 — Standard Simulated Universe Runtime | Deterministic spatial-free Standard SU Activity runtime, complete mechanics, nested battles, replay, baseline AI, CLI and MCP; no expansion-mode runtime | Complete | [Plan](04-standard-universe-runtime.md) | [Ledger](04-standard-universe-runtime-status.md) | [Prompt](04-standard-universe-runtime-prompt.md) |
| Goal 05 — Standard Universe End-to-End Integration | Real nested combat, atomic run effects, composed registries, logical domain scope and component-aware replay over the Goal 04 Standard SU release | Complete | [Plan](05-standard-universe-end-to-end.md) | [Ledger](05-standard-universe-end-to-end-status.md) | [Prompt](05-standard-universe-end-to-end-prompt.md) |

The plan defines what completion means. The ledger is the resumable source of
truth. The prompt must not override either document; it instructs the executor
to follow them until all terminal gates are evidenced.
