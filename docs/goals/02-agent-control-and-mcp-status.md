# Goal 02 Status — Agent Control API and MCP Adapter

This is the persistent execution ledger for
[Goal 02](02-agent-control-and-mcp.md). Update it in the same commit as every
implementation batch.

## Goal state

| Field | Value |
|---|---|
| Goal ID | `agent-control-mcp-v1` |
| State | `InProgress` |
| Prerequisite | Goal 01 `Complete` at or after `b23f900` |
| Active phase | Phase 1 — Protocol-neutral types and observation |
| Next unblocked batch | `G02-P1-B2` |
| Last completed batch | `G02-P1-B1` |
| Last completed commit | This row's containing commit |
| MCP specification baseline | Frozen `2025-11-25` |
| Agent schema revision | Frozen `agent-api-v1` / `1746004f…6725` |
| SDK lock | Official `rmcp 2.2.0` / tag `rmcp-v2.2.0` / Apache-2.0 |
| Standard scenario denominator | Six frozen `scenario.standard-v1.*` production scenarios |
| Blocking condition | None |

Allowed states are `ReadyToStart`, `InProgress`, `Blocked` and `Complete`.
`Blocked` requires that no independent batch can progress and must name the
external evidence or decision required. Phase completion is not goal completion.

## Phase ledger

| Phase | State | Exit evidence |
|---|---|---|
| Phase 0 — Protocol/capability/threat model | `Complete` | Surface audit; MCP/SDK capability lock; `agent-api-v1` schema/budgets; 19-case threat model and fail-closed startup policy |
| Phase 1 — Types and observation | `InProgress` | Protocol-neutral crate boundary complete; exact types/projection/actions/debug/schema implementation pending |
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
| `G02-P0-B1` | `Complete` | This row's containing commit | `node tools/agent-control/verify-surface-audit.mjs`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Frozen seven use cases, six production Standard scenarios, dependency layers, decision ownership, three narrow application seams and forbidden core changes; policy is bound to the Goal 01 production bundle. |
| `G02-P0-B2` | `Complete` | This row's containing commit | `node tools/agent-control/verify-mcp-sdk-lock.mjs`; `cargo test --manifest-path tools/mcp-sdk-capability/Cargo.toml --locked`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Frozen MCP `2025-11-25` and official `rmcp 2.2.0` with exact tag/checksums/features/Apache-2.0 licenses; executable goldens prove stdio, Streamable HTTP, tools/schema/structured output, resources/templates, cancellation and errors, with unsupported assumptions explicit. |
| `G02-P0-B3` | `Complete` | This row's containing commit | `node tools/agent-control/verify-agent-api-v1.mjs`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Frozen observation/action/error schemas, canonical string numerics, default/debug visibility policy, cursor semantics, response/retention/settlement bounds and ordinary/trigger-heavy/error goldens at schema bundle `1746004f…6725`. |
| `G02-P0-B4` | `Complete` | This row's containing commit | `node tools/agent-control/verify-threat-model.mjs`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Frozen 19 threats and controls for ownership, forgery/staleness, payload/replay abuse, prompt/data separation, response loss, races, origins, auth/scopes/tenancy, rate/quota/expiry/cancellation, redaction, stdio, drift, visibility and adapter isolation; three startup profiles include fail-closed non-loopback requirements. |
| `G02-P1-B1` | `Complete` | This row's containing commit | `node tools/workspace/verify-dependencies.mjs`; `cargo test -p starclock-agent-api --all-targets --all-features`; `cargo clippy -p starclock-agent-api --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-targets --all-features`; `git diff --check` | Added dependency-free, protocol-neutral `starclock-agent-api` with public `schema`, `observation`, `action`, `session` and `error` responsibilities; workspace guard forbids unreviewed dependencies and reverse/protocol coupling. |
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
| MCP specification | `2025-11-25` | [`mcp-sdk-lock.json`](../../policy/mcp-sdk-lock.json) |
| MCP Rust SDK/toolchain | `rmcp 2.2.0`; Rust `1.97.0`; Apache-2.0 | [`mcp-sdk-capabilities.json`](../../evidence/agent-control-mcp-v1/protocol/mcp-sdk-capabilities.json) |
| Agent schema | `agent-api-v1` / `1746004f…6725` | [`agent-api-v1.json`](../../evidence/agent-control-mcp-v1/schema/agent-api-v1.json) |
| Threat model | `starclock.agent-control-threat-model.v1` / `4080b72e…c45a` | [`threat-model.json`](../../evidence/agent-control-mcp-v1/security/threat-model.json) |
| Standard scenario denominator | 6 scenarios / Goal 01 bundle `abd84f70…0440` | [`agent-control-surfaces.json`](../../policy/agent-control-surfaces.json) |
| Observation/event limits | 256 KiB observation; 256 events/page; 8,192 retained summaries | [`agent-api-v1.json`](../../policy/agent-api-v1.json) |
| Settlement limits | 4,096 commands; 65,536 events; 262,144 operations | [`agent-api-v1.json`](../../policy/agent-api-v1.json) |
| Session/registry limits | 1,024 global / 64 tenant / 16 principal; idle 1,800 s / max 14,400 s | [`agent-control-threat-model.json`](../../policy/agent-control-threat-model.json) |
| Performance workload | Pending | `G02-P2-B7` / `G02-P4-B5` |

## Decision record

| Date | Decision | Rationale/effect |
|---|---|---|
| 2026-07-22 | MCP is an adapter over `starclock-agent-api`, not the combat API or only service protocol. | Keeps deterministic simulation independent from JSON-RPC, transports and model hosts; supports in-process/high-throughput consumers. |
| 2026-07-22 | Agents select opaque offered-action tokens bound to retained exact commands. | Prevents fabricated damage/targets/costs and reuses the authoritative command legality boundary. |
| 2026-07-22 | Default sessions expose player decisions and settle authored enemy/automatic decisions internally. | Keeps tool calls at meaningful turn boundaries and preserves authored enemy behavior. |
| 2026-07-22 | Remote non-loopback service fails closed without authorization and origin policy. | Avoids accidentally publishing an unauthenticated state-mutating MCP server. |
| 2026-07-22 | No model provider or private chain-of-thought is part of Goal 02. | The server is interoperable infrastructure, and replay depends only on accepted commands. |
| 2026-07-22 | Only player-owned decisions are external; system and enemy decisions settle inside the session. | Reuses exact offered commands while keeping authored automation deterministic and replay-visible. |
| 2026-07-22 | Goal 02 may add only narrow application/data composition seams over Goal 01. | Scenario lookup/construction and controller coordination do not authorize combat-rule, lifecycle, RNG, hash or replay redesign. |
| 2026-07-22 | MCP revision `2025-11-25` and official Rust SDK `rmcp 2.2.0` are frozen. | The locked executable fixture proves the used surface; newer SDK capabilities and protocol revisions remain opt-in future work. |
| 2026-07-22 | `agent-api-v1` transports every authoritative number as a canonical integer string. | Avoids JSON floating-point and cross-language safe-integer loss while preserving exact fixed-point backing values. |
| 2026-07-22 | Event summaries are bounded/cursor-paged while complete facts remain replay-owned. | Keeps observations finite without weakening authoritative audit or silently truncating settlement. |
| 2026-07-22 | Non-loopback HTTP startup requires the complete remote security prerequisite set. | Missing TLS/proxy attestation, token validation, exact origins, identity/scopes, proxy trust, quotas/rates/limits or audit sink is a startup error. |
| 2026-07-22 | Expiry/cancellation are checked outside an atomic domain commit. | Operational time and cancellation cannot create platform-dependent partial battle state; idempotent retry resolves delivery ambiguity. |

Add architectural decisions here before implementing a deviation from the goal
or normative design. A decision cannot silently weaken a terminal gate.

## Blockers and research cases

| ID | State | Question/blocker | Owner batch | Resolution/evidence |
|---|---|---|---|---|
| `G02-R01` | `Resolved` | Which exact official Rust MCP SDK revision and features satisfy the frozen stdio/HTTP/schema/cancellation contract? | `G02-P0-B2` | Official `rmcp 2.2.0`, default-off features `client`, `macros`, `server`, `transport-io`, `transport-streamable-http-server`; exact checksums, licenses and passing fixture are committed. |
| `G02-R02` | `PartiallyResolved` | Which existing Goal 01 view methods are sufficient for player-visible projection, and which narrow queries are missing? | `G02-P0-B1` / `G02-P1-B3` | Public views cover battle/unit/effect/timeline/status projection; P1-B3 must prove whether any visibility-safe query is still missing before adding one. |
| `G02-R03` | `Resolved` | Which current decisions are external player decisions versus authored enemy or automatic orchestration boundaries? | `G02-P0-B1` / `G02-P2-B2` | `Team(Player)` is external; `System` and `Team(Enemy)` settle internally from exact offered commands; automatic timeline work drains in `Battle::apply`. |

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
