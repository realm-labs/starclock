# Goal 02 Status — Agent Control API and MCP Adapter

This is the persistent execution ledger for
[Goal 02](02-agent-control-and-mcp.md). Update it in the same commit as every
implementation batch.

## Goal state

| Field | Value |
|---|---|
| Goal ID | `agent-control-mcp-v1` |
| State | `ReadyToStart` |
| Prerequisite | Goal 01 `Complete` at or after `b23f900` |
| Active phase | Phase 0 — Freeze protocol, capability and threat model |
| Next unblocked batch | `G02-P0-B1` |
| Last completed batch | None |
| Last completed commit | None |
| MCP specification baseline | Proposed `2025-11-25`; freeze in `G02-P0-B2` |
| Agent schema revision | Proposed `agent-api-v1`; freeze in `G02-P0-B3` |
| SDK lock | Pending `G02-P0-B2` capability proof |
| Standard scenario denominator | Pending audit; expected six frozen Goal 01 scenarios |
| Blocking condition | None |

Allowed states are `ReadyToStart`, `InProgress`, `Blocked` and `Complete`.
`Blocked` requires that no independent batch can progress and must name the
external evidence or decision required. Phase completion is not goal completion.

## Phase ledger

| Phase | State | Exit evidence |
|---|---|---|
| Phase 0 — Protocol/capability/threat model | `Pending` | Pending |
| Phase 1 — Types and observation | `Pending` | Pending |
| Phase 2 — Authoritative sessions | `Pending` | Pending |
| Phase 3 — Local MCP | `Pending` | Pending |
| Phase 4 — Remote HTTP | `Pending` | Pending |
| Phase 5 — Hardening/freeze | `Pending` | Pending |

## Batch ledger

Replace `Pending` with `InProgress` only for the one active implementation
batch. A completed row records the containing commit, exact checks and concise
evidence summary.

| Batch | State | Commit | Validation/evidence | Result |
|---|---|---|---|---|
| `G02-P0-B1` | `Pending` | — | — | — |
| `G02-P0-B2` | `Pending` | — | — | — |
| `G02-P0-B3` | `Pending` | — | — | — |
| `G02-P0-B4` | `Pending` | — | — | — |
| `G02-P1-B1` | `Pending` | — | — | — |
| `G02-P1-B2` | `Pending` | — | — | — |
| `G02-P1-B3` | `Pending` | — | — | — |
| `G02-P1-B4` | `Pending` | — | — | — |
| `G02-P1-B5` | `Pending` | — | — | — |
| `G02-P1-B6` | `Pending` | — | — | — |
| `G02-P2-B1` | `Pending` | — | — | — |
| `G02-P2-B2` | `Pending` | — | — | — |
| `G02-P2-B3` | `Pending` | — | — | — |
| `G02-P2-B4` | `Pending` | — | — | — |
| `G02-P2-B5` | `Pending` | — | — | — |
| `G02-P2-B6` | `Pending` | — | — | — |
| `G02-P2-B7` | `Pending` | — | — | — |
| `G02-P3-B1` | `Pending` | — | — | — |
| `G02-P3-B2` | `Pending` | — | — | — |
| `G02-P3-B3` | `Pending` | — | — | — |
| `G02-P3-B4` | `Pending` | — | — | — |
| `G02-P3-B5` | `Pending` | — | — | — |
| `G02-P4-B1` | `Pending` | — | — | — |
| `G02-P4-B2` | `Pending` | — | — | — |
| `G02-P4-B3` | `Pending` | — | — | — |
| `G02-P4-B4` | `Pending` | — | — | — |
| `G02-P4-B5` | `Pending` | — | — | — |
| `G02-P5-B1` | `Pending` | — | — | — |
| `G02-P5-B2` | `Pending` | — | — | — |
| `G02-P5-B3` | `Pending` | — | — | — |
| `G02-P5-B4` | `Pending` | — | — | — |
| `G02-P5-B5` | `Pending` | — | — | — |
| `G02-P5-B6` | `Pending` | — | — | — |

## Frozen identities and budgets

Populate these rows only from committed capability/schema/baseline evidence.

| Identity | Revision/digest | Evidence |
|---|---|---|
| MCP specification | Pending | `G02-P0-B2` |
| MCP Rust SDK/toolchain | Pending | `G02-P0-B2` |
| Agent schema | Pending | `G02-P0-B3` |
| Threat model | Pending | `G02-P0-B4` |
| Standard scenario denominator | Pending | `G02-P0-B1` |
| Observation/event limits | Pending | `G02-P0-B3` |
| Settlement limits | Pending | `G02-P0-B3` |
| Session/registry limits | Pending | `G02-P0-B4` / `G02-P2-B6` |
| Performance workload | Pending | `G02-P2-B7` / `G02-P4-B5` |

## Decision record

| Date | Decision | Rationale/effect |
|---|---|---|
| 2026-07-22 | MCP is an adapter over `starclock-agent-api`, not the combat API or only service protocol. | Keeps deterministic simulation independent from JSON-RPC, transports and model hosts; supports in-process/high-throughput consumers. |
| 2026-07-22 | Agents select opaque offered-action tokens bound to retained exact commands. | Prevents fabricated damage/targets/costs and reuses the authoritative command legality boundary. |
| 2026-07-22 | Default sessions expose player decisions and settle authored enemy/automatic decisions internally. | Keeps tool calls at meaningful turn boundaries and preserves authored enemy behavior. |
| 2026-07-22 | Remote non-loopback service fails closed without authorization and origin policy. | Avoids accidentally publishing an unauthenticated state-mutating MCP server. |
| 2026-07-22 | No model provider or private chain-of-thought is part of Goal 02. | The server is interoperable infrastructure, and replay depends only on accepted commands. |

Add architectural decisions here before implementing a deviation from the goal
or normative design. A decision cannot silently weaken a terminal gate.

## Blockers and research cases

| ID | State | Question/blocker | Owner batch | Resolution/evidence |
|---|---|---|---|---|
| `G02-R01` | `Open` | Which exact official Rust MCP SDK revision and features satisfy the frozen stdio/HTTP/schema/cancellation contract? | `G02-P0-B2` | Pending capability lock. |
| `G02-R02` | `Open` | Which existing Goal 01 view methods are sufficient for player-visible projection, and which narrow queries are missing? | `G02-P0-B1` / `G02-P1-B3` | Pending public-surface audit. |
| `G02-R03` | `Open` | Which current decisions are external player decisions versus authored enemy or automatic orchestration boundaries? | `G02-P0-B1` / `G02-P2-B2` | Pending decision-owner matrix. |

Research does not authorize speculative production behavior. Record primary
sources, executed fixtures and exact limitations. Continue independent work when
a case does not block it.

## Terminal checklist

### Agent API and observation

- [ ] `starclock-agent-api` is protocol-neutral and responsibility-separated.
- [ ] Exact versioned observation/action/error schemas are frozen.
- [ ] Player-visible projection passes hidden-information and bound tests.
- [ ] Debug projection is separately authorized and visibly marked.
- [ ] Offered tokens bind retained exact commands and reject stale/forged use.

### Session and replay

- [ ] Creation accepts validated production identities rather than arbitrary
      untrusted specs/programs.
- [ ] Player actions settle through bounded enemy/automatic decisions.
- [ ] Idempotency, response-loss, race, expiry and close tests pass.
- [ ] All frozen Standard scenarios finish through the agent session loop.
- [ ] Exported replays verify every accepted external/automatic command/hash.

### MCP and remote service

- [ ] MCP protocol and official Rust SDK capability lock are committed.
- [ ] All seven tools and bounded resources have schema/conformance evidence.
- [ ] Stdio end-to-end play passes with protocol-only stdout.
- [ ] Streamable HTTP end-to-end play matches in-process traces.
- [ ] Non-loopback startup, authorization, scopes, origins, tenant ownership,
      limits and rate policies fail closed and pass adversarial tests.

### Engineering and release

- [ ] Windows/Linux/macOS native golden evidence is retained.
- [ ] Fuzz/property/concurrency/security suites pass with reproducible inputs.
- [ ] Performance budgets cover projection, stepping, serialization and memory.
- [ ] Dependency, license, public API, file-size and secret/log audits pass.
- [ ] Clean-checkout acceptance and generated-drift checks pass.
- [ ] Contracts/examples are frozen and `G02-P5-B6` is committed cleanly.

## Completion record

| Field | Value |
|---|---|
| Completion commit | Pending |
| Agent schema digest | Pending |
| MCP capability/conformance evidence | Pending |
| Standard scenario result | Pending |
| Cross-platform evidence | Pending |
| Performance evidence | Pending |
| Clean-checkout evidence | Pending |
