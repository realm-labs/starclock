# Agent Control and MCP Integration

This document defines the protocol-neutral control boundary through which an
external AI agent can play Starclock battles. It also defines the Model Context
Protocol adapter built on that boundary. The agent layer is an integration
surface over the existing deterministic simulation; it is not a second combat
runtime and never mutates battle state directly.

## Design outcome

```text
in-process controller ─┐
HTTP/RPC adapter ──────┼─> starclock-agent-api ─> Battle/Replay
MCP host / LLM ────────┘
```

`starclock-agent-api` owns observations, offered-action handles and an ephemeral
authoritative session facade. `starclock-mcp` converts the same facade into MCP
tools and resources. Existing domain crates do not depend on either crate.

MCP is a supported AI adapter, not Starclock's only service protocol.
High-throughput verification or training may use the Rust facade or a separate
batch/RPC adapter without translating every decision through MCP.

The protocol baseline is MCP revision `2025-11-25`. MCP uses JSON-RPC, exposes
tools/resources/prompts, and supports stdio and Streamable HTTP; see the
official [architecture overview](https://modelcontextprotocol.io/docs/learn/architecture)
and [protocol specification](https://modelcontextprotocol.io/specification/2025-11-25).
The exact Rust SDK is pinned only after Goal 02 executes a capability lock.

## Dependency and authority boundary

The intended workspace additions are:

```text
starclock-agent-api
  owns: exact agent DTOs, observation projection, offered actions, sessions
  excludes: MCP/HTTP types, provider SDKs, required async runtime

starclock-mcp
  owns: MCP schemas, tools, resources, transports and transport authorization
  excludes: combat formulas, command construction and authoritative mutation
```

`Battle::apply(Command)` remains the only battle commit boundary. An agent
selects one exact command already offered by `DecisionPoint`; it cannot submit
damage, targets, costs, selectors, RNG results or a hand-authored equivalent.

Operational session IDs, leases, request IDs, wall-clock expiry and network
authentication are nonauthoritative. They never enter the battle state hash or
alter RNG consumption. Accepted commands and hashes remain replay-verifiable.

## Agent session contract

The public application surface is conceptually:

```rust
pub struct AgentSession { /* private battle and replay recorder */ }

impl AgentSession {
    pub fn create(
        catalogs: AgentCatalogs,
        request: CreateBattleRequest,
        policy: AgentSessionPolicy,
    ) -> Result<(Self, StepResult), AgentSessionError>;

    pub fn observe(&self) -> AgentObservation;

    pub fn apply(
        &mut self,
        request: AgentActionRequest,
    ) -> Result<StepResult, AgentActionError>;

    pub fn export_replay(&self) -> Result<ReplayArtifact, AgentSessionError>;
}
```

A hosting registry assigns opaque `SessionId` values, serializes mutation per
session and shares immutable catalogs. The domain session remains usable
in-process without a network server or async runtime.

Production creation resolves a validated Standard scenario, build bindings and
seed policy. An untrusted caller does not upload an arbitrary `BattleSpec`,
catalog or Rule IR program. A separate trusted embedding API may accept exact
validated specs, but public MCP tools do not imply that authority.

## Observation model

`AgentObservation` is an owned, versioned projection rather than serialized
`BattleView` internals:

```rust
pub struct AgentObservation {
    pub schema_revision: AgentSchemaRevision,
    pub session_id: SessionId,
    pub scenario_id: ScenarioId,
    pub catalog_digest: CatalogDigest,
    pub decision_id: DecisionId,
    pub state_hash: StateHash,
    pub event_cursor: EventCursor,
    pub status: AgentBattleStatus,
    pub battle: AgentBattleView,
    pub legal_actions: Box<[OfferedAction]>,
}
```

The projection includes only information visible under an explicit policy:

- units in canonical formation/stable-ID order;
- exact HP, Toughness, Energy, team resources and action-order values;
- visible effects, stacks, durations, weaknesses and battlefield presence;
- encounter progress and public enemy intent when the profile exposes it;
- the current external decision and its ordered offered actions;
- bounded event summaries after a requested cursor.

Authoritative numbers use exact scaled integers or canonical decimal strings.
JSON floating point is not an authoritative transport. Internal stores, hidden
enemy-controller state, future RNG draws and unrevealed intent are absent from
the default `PlayerVisible` policy.

`OmniscientDebug` may exist for trusted tests. It is a separate capability and
authorization scope, is marked in output and is disabled by default.

Large histories are cursor-paged and bounded. An action response returns a
concise ordered summary and the next observation; complete accepted facts stay
in the replay. No MCP response embeds an unbounded log or complete catalog.

## Offered actions

Every `OfferedAction` describes one exact current `Command` retained privately:

```rust
pub struct OfferedAction {
    pub token: ActionToken,
    pub ordinal: u32,
    pub actor: UnitId,
    pub ability: Option<AbilityId>,
    pub primary_target: Option<UnitId>,
    pub cost: AgentResourceCost,
    pub tags: Box<[AgentActionTag]>,
    pub summary: String,
}
```

`ActionToken` is opaque and scoped to one session and decision. A request
contains:

```text
session_id
expected_decision_id
expected_state_hash
action_token
idempotency_key
```

Unknown, stale or cross-session tokens are rejected before `Battle::apply`. A
repeated idempotency key with the same payload returns the cached result; reuse
with another payload is rejected. Racing requests cannot both commit one
decision.

## Decision settlement

The default `StandardPlayer` session exposes only player tactical decisions.
Creation and every player action settle until the next external decision:

1. accept the selected exact player command;
2. execute the complete combat resolution synchronously;
3. let authored enemy AI answer enemy-owned decisions;
4. execute explicitly automatic orchestration decisions;
5. stop at the next player decision, terminal outcome or deterministic fault.

All automatically selected commands carry controller identity in replay
diagnostics. An explicit test policy may expose both sides, but default MCP does
not ask an LLM to reproduce enemy AI.

One MCP action means one external decision, not one hit, trigger, event or
resolver operation. Settlement has fixed command/event/operation budgets.

## MCP surface

Version 1 exposes these tools:

| Tool | Purpose |
|---|---|
| `starclock_list_scenarios` | Return bounded Standard scenario summaries. |
| `starclock_create_battle` | Create an authorized ephemeral session. |
| `starclock_observe_battle` | Return the current observation and bounded events. |
| `starclock_play_action` | Select one offered action and settle to the next decision. |
| `starclock_export_replay` | Export/reference the canonical audit replay. |
| `starclock_close_battle` | Close an owned session. |
| `starclock_verify_replay` | Verify a bounded replay against an exact catalog. |

Tools return structured content and stable machine errors. Descriptions help a
model choose, but the server validates every field independently.

Static context is exposed as bounded resources:

```text
starclock://catalog/manifest
starclock://rules/core-combat
starclock://scenario/{scenario_id}
starclock://character/{form_id}
```

Resources provide concise original summaries and exact public mechanics. They
do not return workbooks, Sora rows, caches, proprietary text or hidden runtime
state. An optional usage prompt may explain the loop, but prompts never grant
authority or change rules.

## Transport profiles

### Local stdio

```text
starclock mcp serve --transport stdio
```

Stdout is reserved for MCP frames; logs use stderr. The process serves its local
invoker, reads explicit/environment configuration and opens no listener.

### Remote Streamable HTTP

Remote support uses MCP Streamable HTTP and is disabled unless selected.
Loopback development requires an explicit local-only mode. Non-loopback startup
requires authorization, origin validation, request limits, ownership and rate
limits.

The adapter follows the frozen official
[authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)
rather than inventing authentication inside combat. Operating an OAuth server,
TLS termination, accounts and distributed storage remain deployment concerns.

The frozen resource-server scopes are:

```text
starclock:scenario:read
starclock:battle:create
starclock:battle:read
starclock:battle:act
starclock:battle:replay
starclock:battle:close
starclock:replay:verify
starclock:debug:omniscient
```

Remote sessions are tenant-bound, use opaque IDs, have injected expiry and
enforce quotas. Expiry is never read during `Battle::apply`; it only determines
whether the adapter accepts the next request.

Every authenticated HTTP request reconstructs its owner from validated tenant
and principal claims; an MCP transport session never grants battle authority.
The frozen active-session quotas are 1,024 global, 64 per tenant and 16 per
principal. One-minute admission windows allow 30 creates per principal, 600
mutations per tenant and 1,200 reads per tenant, using an injected monotonic
clock and bounded generic retry metadata.

## Replay and diagnostics

Replay records every player, enemy and automatic accepted command and every
resulting state hash. Non-authoritative diagnostics may record controller
version/digest, MCP request identity, action ordinal, idempotency digest and
bounded score or short user-supplied note.

Starclock never requires or stores private model chain-of-thought. Diagnostics
do not alter state, events, RNG or hashes. Verification replays accepted
commands without rerunning the external model.

## Performance and operational rules

- One battle mutates serially; independent sessions run concurrently.
- Exact validated catalogs are shared immutably through `Arc`.
- MCP/JSON conversion is measured separately from `Battle::apply`.
- Observation projection is bounded and avoids unrelated catalog rescans.
- Live services retain incremental state; they do not replay growing prefixes.
- Bulk verification/training uses the in-process or batch API, not MCP per step.
- Cancellation cannot interrupt an accepted command. Idempotency recovers a
  committed response whose delivery was lost.

## Failure model

Errors distinguish invalid schema/revision, unknown/expired/not-owned session,
stale decision/hash, invalid action token, idempotency conflict, unauthorized
policy, configuration rejection, combat rejection/fault, budget/rate limits,
replay divergence and internal adapter failure.

Invalid requests never mutate state. Delivery failure after commit retains the
result for retry. Adapter errors do not convert a healthy battle to `Faulted`;
only the combat fault contract may do that.

## Acceptance contract

- an external client completes every frozen Standard scenario using only
  offered actions;
- in-process, stdio and HTTP paths produce identical commands/events/hashes;
- JSON schemas contain no authoritative floats or internal types;
- projections prove stable order, exact values, bounds and hidden-data safety;
- stale, forged, cross-session and racing requests do not mutate or consume RNG;
- idempotent retry returns the original result after response loss;
- automatic enemy settlement is bounded and fully replayed;
- stdio emits no non-protocol stdout bytes;
- non-loopback remote startup fails closed without authorization/origin policy;
- replay verification detects command, policy, catalog and hash divergence;
- load reports separate combat, projection, serialization and registry costs;
- domain crates acquire no MCP, HTTP or model-provider dependency.

## Excluded scope

This boundary does not implement model inference, provider orchestration,
optimal play, reinforcement-learning training, vectorized environments,
durable/distributed saves, matchmaking, accounts, billing, Bevy UI,
universe/challenge content or automation of the official game client.
