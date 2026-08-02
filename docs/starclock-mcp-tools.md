# Starclock MCP control tools

Batch `G02-P3-B2` exposes exactly seven MCP tools over the protocol-neutral
`starclock-agent-api`. The adapter never constructs combat commands. It accepts
only validated scenario/session values and opaque actions that the active
decision already offered, then delegates ownership, mutation, idempotency,
settlement, replay and close behavior to the application registry.

Every successful result uses MCP `structuredContent` and has a discovered
`outputSchema`. Observation, action-response and replay-diagnostic schemas are
fully nested objects rather than untyped JSON placeholders. All authoritative
numbers remain canonical decimal strings, as required by `agent-api-v1`.

| Tool | Input | Structured success result |
|---|---|---|
| `starclock_list_scenarios` | Empty object | Frozen schema revision and the six production scenario identities, definition/encounter IDs and default seeds. |
| `starclock_create_battle` | Schema revision, scenario ID and optional exact decimal seed | Complete first player-visible observation for a newly owned session. |
| `starclock_observe_battle` | Schema revision, session ID and optional event cursor | Current player-visible observation with the bounded page strictly after the cursor; omission means `event_0`. |
| `starclock_play_action` | Schema revision, session/decision IDs, expected state hash, opaque action token and idempotency key | Commit/idempotency facts, bounded settlement summary and complete next observation. |
| `starclock_export_replay` | Schema revision and session ID | Canonical replay as lowercase hex, SHA-256, command count and nonauthoritative controller diagnostics. |
| `starclock_close_battle` | Schema revision and session ID | Exact closed session identity and `closed:true`; active quota capacity is released. |
| `starclock_verify_replay` | Schema revision, scenario ID, optional exact seed and lowercase replay hex | Fresh verification command count, final state hash and battle phase. No live session or model is used. |

`schema_revision` is exactly `agent-api-v1`. Optional creation/verification seed
values are exact unsigned decimal strings; omission selects the scenario's
authored default. Replay import accepts only even-length lowercase hexadecimal
and is capped at 64 MiB after decoding. Transports may impose a tighter complete
request bound; local stdio accepts at most a 16 KiB JSON-RPC frame. A verifier
constructs a fresh frozen Standard scenario from the supplied identity and seed,
so exported bytes remain verifiable after their original session is closed.

Application failures are MCP tool errors with `isError:true` and the exact
bounded `AgentError` in `structuredContent`. JSON-RPC errors are reserved for
routing, parameter decoding and adapter infrastructure failures. Tool success
content never includes retained commands, Rule IR, AI state, RNG state,
authorization material or omniscient debug data.

The six Activity tools extend this surface without adding mode-specific tool
names:

| Tool | Input | Structured success result |
|---|---|---|
| `starclock_create_universe` | Schema revision, optional mode, exact seed and mode entry fields | Complete first external Activity observation for a newly owned session. |
| `starclock_observe_activity` | Schema revision and session ID | Current external Activity observation. |
| `starclock_play_activity_action` | Schema revision, session/boundary IDs, expected state hash, opaque action token and idempotency key | Commit/idempotency facts, bounded nested-battle settlement and next observation. |
| `starclock_export_activity_replay` | Schema revision and session ID | Canonical replay hex, SHA-256, action count and completeness. |
| `starclock_close_activity` | Schema revision and session ID | Exact closed session identity and `closed:true`; shared Activity quota capacity is released. |
| `starclock_verify_activity_replay` | Schema revision, optional mode, exact seed, mode entry fields and replay hex | Fresh action/battle counts, final state hash and terminal summary. |

Omitting `mode` or using `standard` preserves the Standard contract and requires
`world` plus `difficulty_index`. `mode:"gold-and-gears"` selects the fixed Gold
and Gears entry; `world` and `difficulty_index` may be omitted, but if present
must be `401` and `0`. `mode:"swarm-disaster"` selects the fixed Swarm entry
and accepts only `201` and `0` when those fields are present. Standard, Gold
and Swarm sessions share the same owner binding, opaque identities, quotas,
leases, idempotency rules and replay-import limit.
Unknown modes and incompatible fixed-entry fields are rejected before session
creation or replay verification.

The adapter exposes eight inert static resources: core catalog and rules,
Standard manifest and rules, Gold and Gears manifest and rules at
`starclock://universe/gold-and-gears/manifest` and
`starclock://rules/gold-and-gears`, and Swarm manifest and rules at
`starclock://universe/swarm-disaster/manifest` and
`starclock://rules/swarm-disaster`. The mode resources and tools use the
existing Activity-read and Activity-tool scopes; the fixed thirteen-scope
authorization matrix and thirteen tool names are unchanged. Two existing RFC
6570 resource templates and the `starclock_battle_loop` prompt are unchanged.
