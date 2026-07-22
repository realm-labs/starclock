# Goal 02 — Agent Control API and MCP Adapter

## 1. Objective

Implement a protocol-neutral external-control boundary for Starclock Standard
battles and expose it through a conformant MCP server. An AI host must be able
to create a validated battle, observe authorized state, select one exact offered
action, receive the next decision, export a replay and verify the result without
access to combat internals.

Goal 02 builds on completed Goal 01. It does not add combat content or redesign
`Battle::apply`; it packages the existing decision/view/replay boundary for
external agents. The normative design is
[Agent control and MCP integration](../22-agent-control-and-mcp.md).

## 2. Terminal outcome

Goal 02 is complete only when all of the following are evidenced:

- `starclock-agent-api` provides versioned observations, offered-action tokens,
  step results, exact errors and an ephemeral authoritative session;
- an external controller completes every frozen Standard scenario by choosing
  only offered actions while authored enemy decisions settle internally;
- stale, forged, racing and idempotent retry cases prove non-mutation and
  exactly-once behavior;
- replays contain all external and automatic commands and verify without the
  external model;
- `starclock-mcp` exposes the documented tools/resources over stdio and
  Streamable HTTP under a frozen MCP/SDK capability lock;
- local stdio works end-to-end and emits no log bytes on stdout;
- non-loopback HTTP fails closed without authorization, origin, ownership and
  limit policies;
- in-process, stdio and HTTP paths produce identical authoritative traces;
- schema, property, concurrency, security, replay, performance and
  cross-platform gates pass from a clean checkout;
- the persistent ledger contains committed evidence for every batch.

## 3. Scope

### Included

- validated Standard-scenario creation through production catalogs;
- player-visible and trusted omniscient observation policies;
- exact fixed-point-safe JSON values and canonical ordering;
- session-scoped actions mapped privately to exact `Command` values;
- player control plus bounded authored enemy/automatic settlement;
- idempotency, decision/hash preconditions and serialized session mutation;
- replay export, controller diagnostics and verification;
- MCP tools, resources, optional prompt, stdio and Streamable HTTP;
- authorization integration, scopes, ownership, quotas, rate/body limits,
  origin validation, health and graceful shutdown;
- performance baselines for projection, stepping and serialization.

### Excluded

- combat formula/content changes except a proven narrow read-only query needed
  by observation projection;
- model-provider SDKs, hosted model calls, chain-of-thought capture or agent
  orchestration;
- optimal-play claims, RL training or vectorized environments;
- durable/distributed sessions, accounts, matchmaking, billing, TLS termination
  or operating an OAuth authorization server;
- universe/challenge modes, new content, Bevy/UI work;
- automation or screen/input control of the official game client.

## 4. Architecture constraints

1. `starclock-agent-api` contains no MCP/HTTP types, provider SDK or required
   async runtime.
2. `starclock-mcp` depends on agent API; domain crates never depend on MCP.
3. Only `Battle::apply` commits battle state. Agents select retained commands.
4. Operational IDs, clocks, leases and auth never enter hashes or combat RNG.
5. Default observations are player-visible; debug data requires a capability.
6. Exact transport uses scaled integers/canonical decimals, never JSON floats.
7. One agent action settles to the next external decision, not one hit/event.
8. One session mutates serially; independent sessions may run concurrently.
9. Response loss after commit is recovered through idempotency.
10. Files stay below 1,200 physical lines unless reviewed; split by
    responsibility and avoid broad convenience `pub use` exports.

## 5. Commit and progress contract

Each batch is one responsibility-bounded commit. Update
[the Goal 02 ledger](02-agent-control-and-mcp-status.md) in the same commit with
commands, evidence, decisions and blockers.

Commit subjects use:

```text
<type>(<scope>): <batch-id> <imperative summary>
```

Examples:

```text
feat(agent): G02-P2-B3 enforce idempotent action application
feat(mcp): G02-P3-B2 expose battle control tools
```

## 6. Delivery phases

### Phase 0 — Freeze protocol, capability and threat model

**Exit gate:** exact MCP/SDK behavior, public schema, trust profiles, limits and
licenses are recorded; executable fixtures prove assumed SDK features.

| Batch | Atomic deliverable |
|---|---|
| `G02-P0-B1` | Audit Goal 01 public surfaces and freeze use cases, Standard scenario denominator, dependency directions and forbidden core changes. |
| `G02-P0-B2` | Freeze MCP revision `2025-11-25`; evaluate the current official Rust SDK; pin its exact version/checksum/features/licenses; add a golden project proving tools, structured output, resources/templates, stdio, Streamable HTTP, cancellation and errors. Record unsupported assumptions explicitly. |
| `G02-P0-B3` | Freeze `agent-api-v1` observation/action/error JSON schemas, exact numeric encoding, visibility policies, event paging and size/settlement budgets with ordinary and trigger-heavy goldens. |
| `G02-P0-B4` | Commit a threat model for ownership, forged/stale actions, payload/replay abuse, prompt/data boundaries, response loss, races, origins, auth scopes, rate limits and fail-closed startup. |

### Phase 1 — Protocol-neutral types and observation

**Exit gate:** battle state projects into exact, bounded, stable and
visibility-correct owned values; each action binds one retained command.

| Batch | Atomic deliverable |
|---|---|
| `G02-P1-B1` | Add `starclock-agent-api`, dependency guards and responsibility-split schema, observation, action, session and error modules. |
| `G02-P1-B2` | Implement revisioned IDs, exact numeric DTOs, battle/unit/effect/timeline/status views, errors and deterministic JSON/debug conversion. |
| `G02-P1-B3` | Implement `PlayerVisible` projection with canonical ordering, public-intent policy, bounded events and hidden-information tests. Add only proven-missing narrow core queries. |
| `G02-P1-B4` | Implement offered actions, opaque decision-scoped tokens and a private exact-command table; test canonical order, summaries and forged/cross-decision rejection. |
| `G02-P1-B5` | Implement separately gated `OmniscientDebug`, mark it in output and prove default output cannot expose its fields. |
| `G02-P1-B6` | Publish JSON Schema/goldens and property tests for ordering, exact number round trips, bounds, unknown revisions and container-order independence. |

### Phase 2 — Authoritative ephemeral sessions

**Exit gate:** an in-process external controller safely and replayably finishes
all frozen Standard scenarios through session methods alone.

| Batch | Atomic deliverable |
|---|---|
| `G02-P2-B1` | Implement validated scenario/build/seed creation and `AgentSession` ownership of one battle plus replay recorder over immutable catalogs. Untrusted creation accepts IDs/policies, not arbitrary Rule IR or `BattleSpec`. |
| `G02-P2-B2` | Settle creation/actions to the next external player decision, including authored enemy and explicit automatic decisions. Record every accepted command/controller. |
| `G02-P2-B3` | Enforce expected decision/hash, idempotency keys and cached committed responses. Simulate response loss and prove retries cannot double-commit or consume RNG. |
| `G02-P2-B4` | Implement cursor-based events, bounded retention and terminal/fault results; keep complete accepted facts in replay. |
| `G02-P2-B5` | Complete replay export/verification with external/automatic diagnostics without redesigning Goal 01's canonical envelope. |
| `G02-P2-B6` | Add an in-memory registry with serialized mutation, injected operational clock/ID source, expiry, close and quotas. Prove isolation under scheduler reordering. |
| `G02-P2-B7` | Run every frozen Standard scenario through a scripted external controller and establish projection/step/memory performance baselines. |

### Phase 3 — Local MCP adapter

**Exit gate:** an independent MCP client discovers Starclock, finishes a battle
over stdio, exports its replay and verifies identical hashes.

| Batch | Atomic deliverable |
|---|---|
| `G02-P3-B1` | Add `starclock-mcp` over the capability lock, protocol metadata and structured error mapping, with no command construction. |
| `G02-P3-B2` | Implement list scenarios, create, observe, play action, export replay, close and verify replay tools with typed structured output. |
| `G02-P3-B3` | Implement bounded catalog/rules/scenario/character resources and optional usage prompt; exclude workbook/generated/cache/private data. |
| `G02-P3-B4` | Add `starclock mcp serve --transport stdio`; reserve stdout for frames and send diagnostics to stderr. |
| `G02-P3-B5` | Add scripted-client and MCP Inspector/conformance fixtures for discovery, play, stale/malformed calls, cancellation, replay and shutdown. |

### Phase 4 — Remote Streamable HTTP

**Exit gate:** the remote adapter is interoperable, bounded, tenant-isolated and
cannot accidentally start an unsafe non-loopback service.

| Batch | Atomic deliverable |
|---|---|
| `G02-P4-B1` | Implement Streamable HTTP with protocol/session headers, body/response limits, allowed origins, bounded workers and explicit loopback-only development mode. |
| `G02-P4-B2` | Integrate frozen MCP authorization scopes for catalog, create, observe, act, replay and debug. Reject non-loopback startup with incomplete auth. |
| `G02-P4-B3` | Bind sessions/idempotency to tenant authority, enforce per-tenant quotas/rates and prove cross-tenant operations fail without leakage. |
| `G02-P4-B4` | Add health/readiness, nonauthoritative metrics, graceful shutdown and bounded draining without affecting hashes. |
| `G02-P4-B5` | Run an HTTP client conformance and multi-session load suite; prove trace equivalence and record adapter latency, throughput, allocations and peak bytes/session. |

### Phase 5 — Hardening and documentation freeze

**Exit gate:** every terminal outcome has clean-checkout, cross-platform and
security evidence and public contracts are frozen.

| Batch | Atomic deliverable |
|---|---|
| `G02-P5-B1` | Fuzz malformed schemas/tokens, idempotency conflicts, cursors, replays, settlement limits and races with reproducible corpora. |
| `G02-P5-B2` | Verify schema bytes and command/event/hash traces on native Windows, Linux and macOS CI; distinguish compile-only evidence. |
| `G02-P5-B3` | Audit dependencies, API, file size, unsafe code, SDK/license lock, secrets/logs, origins, scopes, limits and absence of provider/core coupling. |
| `G02-P5-B4` | Freeze library/MCP/CLI contracts and publish in-process, stdio and authorized remote examples, including when not to use MCP. |
| `G02-P5-B5` | Run isolated clean-checkout acceptance and retain capability, conformance and performance evidence. |
| `G02-P5-B6` | Mark the ledger complete only after all gates pass and commit the final Goal 02 completion record. |

## 7. Acceptance suites

### Domain and determinism

- every selection resolves to an exact command in the active decision;
- invalid/stale/forged input leaves state, replay and RNG unchanged;
- idempotent retries return the original committed result;
- automatic commands are controller-identified and replayed;
- all transports produce identical authoritative traces;
- operational metadata cannot change canonical state or hash.

### Observation and information policy

- exact values round-trip without JSON floating point;
- units, effects, timeline and actions use canonical order;
- hidden AI/RNG/future-intent/internal fields are absent from player output;
- debug output requires explicit policy and remote scope;
- event/resource payloads are bounded, paged and visibly truncated;
- resources expose no proprietary long text or authoring/generated records.

### Session and replay

- every Goal 01 Standard scenario reaches its expected result;
- two racing actions cannot both commit one decision;
- response-loss retry, expiry, close and tenant isolation pass;
- replay verifies without an LLM and detects command/catalog/policy/hash drift;
- live stepping never replays growing prefixes from battle start.

### MCP and remote security

- tool/resource/prompt schemas match capability-lock goldens;
- stdio stdout contains only protocol traffic;
- HTTP validates origin, body, session and protocol headers;
- non-loopback startup fails closed without authorization;
- scopes and ownership are independent for observe, act, replay and debug;
- malformed/oversized/rate-limited input fails before unbounded allocation or
  authoritative mutation.

### Engineering and performance

- domain crates have no MCP/HTTP/provider dependency;
- fmt, denied-warning clippy, tests, generated drift and policy checks pass;
- no unreviewed file exceeds 1,200 LOC or adds broad re-exports;
- benchmarks separate combat, projection, JSON/MCP and registry costs;
- sessions share immutable catalogs, never mutable simulation state;
- load evidence records latency, commands/second/core, allocation and peak
  bytes/session against reviewed stable-runner budgets.

## 8. Progress accounting

The [status ledger](02-agent-control-and-mcp-status.md) records the active batch,
SDK lock, schemas, threat model, checks, evidence, cross-platform results,
performance, decisions, blockers and terminal checklist.

The [launch prompt](02-agent-control-and-mcp-prompt.md) instructs an executor to
select one unblocked batch, implement and validate it, update the ledger, commit
atomically and immediately continue until every terminal gate is proven.
