# Goal 06 Agent and MCP Dynamic Assembly

This document is normative for `G06-P3-B2`.

## Agent session boundary

Every newly created Standard Universe Agent session owns a shared
`StandardUniverseBattleAssembler` reference obtained from
`StandardUniverseRuntimeInstance::into_dynamic_parts`. The session does not
retain or select a frozen entry-time combat catalog.

When an opaque activity action reaches a pending battle, the session delegates
the entire boundary to
`UniverseNestedBattleExecutor::execute_dynamic_pending_activity_battle`. That
operation:

1. snapshots current Activity contributions and participant carry;
2. assembles and atomically starts the selected battle;
3. executes against the immutable combat catalog paired with that assembly;
4. settles the projected result into the Activity;
5. restores the pre-start Activity on execution or settlement failure.

The Agent API continues to expose only ordered opaque action tokens. Clients
cannot supply an encounter, contribution set, combat catalog, combat-input
digest or assembly digest.

## Replay contract

New Agent sessions create a component-addressed replay-v3 header and export
with `encode_standard_universe_trace_parts_v3`. Fresh verification uses
`verify_standard_universe_replay_v3_dynamic`, which reconstructs every battle
from the recorded Activity boundary through the same assembler.

Historical replay-v2 support remains a separate compatibility surface. It is
not used to record or verify a newly created Universe Agent session.

## MCP boundary

MCP remains a transport adapter over `ActivityAgentSessionRegistry`. Its
Universe create, play, export and verify tools do not construct battles and do
not own a second assembly implementation. Consequently MCP, direct Agent
calls and the Universe runtime share the same current-state assembly path.

This migration does not revise:

- `agent-api-v1` request or response schemas;
- ownership checks or information-hiding behavior;
- activity create/read/act/replay/close authorization scopes;
- tenant, principal or global quotas;
- idempotency-key conflict and retry behavior;
- opaque action-token authority;
- session expiry, close or replay-import limits.

## Evidence

The Agent integration fixture completes a real Standard Universe session,
asserts that exported bytes decode as replay v3, verifies those bytes through a
fresh factory and rejects corruption without mutating the live session. Its
concurrent fixture proves mutable Activity state remains isolated while the
immutable catalog and assembly cache are shared.

The MCP tool fixture creates a Universe session, applies and retries the same
opaque action, and exports its activity replay through the same Agent registry.
Authorization mapping tests continue to bind each tool to its existing scope.
