# Player-visible projection

`starclock-agent-api::observation::project_player_visible` is the sole default
conversion from an immutable `starclock-combat::BattleView` into the owned
`agent-api-v1` observation value. The conversion is an allowlist: adding a
field to a combat view does not add it to agent output.

## Stable boundary

Projection succeeds only at `AwaitingCommand`, `Won`, `Lost` or `Faulted`.
`Initializing`, `Resolving` and the transient `Downed` life state are rejected
rather than exposed as externally actionable snapshots. Internally transformed
units retain public presence as `present`; transformation details are not part
of the version-one player schema.

The existing Goal 01 views provide every required public fact. No combat query
or combat-rule change was needed. The public projection deliberately omits:

- enemy AI state, graphs, candidates and authored automatic abilities;
- legal commands and internal command payloads;
- rule/modifier instances, effect source definitions and snapshots;
- private catalog/runtime identities and future RNG state;
- unpublished future intent.

`public_intent` is therefore absent until combat content explicitly authors a
visibility-safe intent. It is never inferred from AI state or future commands.

## Determinism and bounds

Units and effects retain the existing ID order supplied by `BattleView`.
Timeline actors retain canonical actor order. Teams are always emitted as
player then enemy. Exact values use the schema's canonical decimal strings.

The projection rejects more than 128 units, 2,048 effects or 256 timeline
entries. Event projection returns at most 256 family-only summaries and marks a
page truncated when more input remains. Summaries contain only event ID, root
command ID, a stable event family and fixed public prose; typed event payloads
are never formatted or serialized. Complete event facts remain replay-owned.
