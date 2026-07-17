# Battle aggregate boundary

Goal 01 batch `G01-P3-B1` establishes the first executable ownership boundary
for exactly one battle. `Battle` retains an immutable `Arc<CombatCatalog>` and
one private authoritative `BattleState`; callers can mutate it only by selecting
an exact `Command` value from the current `DecisionPoint`.

## Construction input

`ResolvedCombatantSpec` is a combat-domain assembly containing a form, level,
maximum HP, Speed, canonical ability/rule-bundle/modifier bindings and an opaque
digest. It contains no Trace, Eidolon, Light Cone, relic, inventory, account,
Sora or generated-row type. `BattleSpec` binds those assemblies to player/enemy
formation slots, the selected encounter, team Skill Point bounds, a rules/spec
digest and an explicit concession policy.

Construction canonicalizes participants by `(side, formation_index)`, rejects
duplicate or missing sides, and caps the initial player/enemy formation ranges
at the shared schema bounds. Before allocating any runtime identity it verifies
the encounter, unit, ability, rule-bundle, modifier and encounter-enemy source
references. Enemy forms must match their selected encounter definitions.

## Authoritative state and views

Initial runtime unit, timeline-actor, spawn, wave and decision identities are
monotonic non-zero values allocated in canonical formation order. Dense private
stores retain tombstone-capable slots for units and timeline actors; formation
and team stores expose no backend iteration. Units carry independent
`LifeState` and `PresenceState` axes, their generic source/digest and every
selected definition binding so construction never silently discards upstream
combat facts.

`BattleView` borrows the aggregate and exposes explicit stable-ID, formation,
timeline, team, encounter and compatibility projections. It offers no mutable
state/store reference. Compatibility identity includes catalog, rules and spec
digests/revisions, the isolated battle seed and the fixed numeric/RNG revisions.

## Command boundary

Top-level phases are exactly `Initializing`, `AwaitingCommand`, `Resolving`,
`Won`, `Lost` and `Faulted`. `Resolving` is set and drained synchronously inside
`apply`; it is not externally observable as a suspended decision.

The B1 executable path is deliberately real and minimal: an initial
system-owned `StartBattle` decision reaches a player-owned normal-action
decision, and `ConcedePolicy::Allowed` offers `Concede`, which reaches terminal
`Lost`. The no-concession profile is added only when the normal-action pipeline
in B3/B4 supplies another legal command, avoiding an empty or fake decision.
Ability and interrupt command shapes already carry actor, ability, target and
decision identity, but are not offered before their owning action batches.

Every command carries the monotonic decision ID it answers. Validation checks
terminal/resolving phase, staleness and exact offered-value membership before
mutation. Stale, forged, unoffered and terminal commands leave the observable
phase, decision, stores, accepted-command revision and RNG draw count unchanged.
Legal command storage uses an explicit total key rather than enum/container
iteration order.

`Resolution` currently owns the committed phase, optional next decision,
accepted-command revision and RNG draw count with private fields. `G01-P3-B2`
adds the reusable `clone_from`/swap transaction, events/causes, deterministic
faults and canonical state digest without exposing those private backends.
