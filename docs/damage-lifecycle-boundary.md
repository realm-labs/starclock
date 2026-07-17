# Damage, Healing, Defeat and Wave Boundary

`G01-P3-B5` turns the structural action slice into the first HP-changing combat
path. It remains deliberately smaller than the Phase 4 stat/modifier/rule
pipeline: catalog definitions carry already-resolved synthetic formula inputs,
and later lowering may produce the same typed operations from stat queries and
Rule IR without changing battle mutation ownership.

## Authored and runtime structure

An `AbilityActionDefinition` owns one to 64 ordered `ActionHitDefinition`
values. Each hit owns an ordered list from the closed initial operation family:

- `Damage(OrdinaryDamageDefinition)`;
- `Heal(HealingDefinition)`.

Action lowering allocates monotonic `OperationId` values and copies the finite
templates into the private action plan. Each operation records a journal
snapshot before calculation. The pure formula service receives explicit
fixed-point inputs, cannot inspect battle state, and returns a raw six-decimal
intermediate plus its once-floored integral result. Only the operation executor
can apply that result to HP and emit facts.

Ordinary damage evaluates the documented blocks in this order: base,
original-damage, CRIT, DMG Boost, Weaken, DEF, RES, vulnerability, mitigation
and broken multiplier. The normative synthetic vector remains exactly `648`
against an unbroken target and `720` against the otherwise identical broken
target. Healing evaluates `base * (1 + outgoing + incoming - reduction)`, floors
once, then bounds the effective amount by missing HP. Damage and healing events
retain the raw result, calculated integral amount, effective applied amount,
before/after HP, and damage clamp or overheal information.

## Defeat and terminal settlement

After damage clamps HP, an unreplaced zero-HP living unit enters `Downed`, then
`Defeated`; both transitions are journaled and emitted before the hit continues.
Defeat credit uses the operation cause's explicit `applier`. Defeated actors
remain recorded but are excluded from target and timeline eligibility.

The initial slice has no death-prevention or revival proposal source, so the two
life transitions are adjacent. Phase 4 can insert replacement work between them
without moving HP mutation into character-specific code.

At the action boundary, player extinction settles `Lost` first. If players
remain and the current wave has no living present hostile, the encounter either
settles `Won` or performs its default `AfterAction` transition. Terminal states
clear decisions, interrupt state and active-turn ownership and accept no further
commands.

## Ordered waves

`EncounterDefinition` now owns non-empty ordered enemy-authorization waves.
`ParticipantSpec::with_wave` binds each resolved enemy occurrence to a one-based
wave; players are fixed to wave one. Future enemies are created as `Reserved`
records so runtime IDs, source digests and canonical ordering are frozen at
battle creation, but they are neither targetable nor timeline-eligible.

On transition the resolver emits `WaveEnded`, marks prior-wave enemy records
`Departed`, makes the next wave `Present` in stable unit-ID order, allocates a
new `WaveInstanceId`, updates canonical encounter progress, then emits
`WaveStarted`. Surviving players, Skill Points, Energy and gauges persist. A
two-hit golden proves that a kill in hit one only empties hit two under its
target-invalidation policy; the reserved next wave cannot receive that hit.

## Evidence

The black-box fixture in
[`damage_lifecycle.rs`](../crates/starclock-combat/tests/damage_lifecycle.rs)
pins command-to-state hashes for damage/healing, one-wave victory, wave-two
entry, final multi-wave victory and player loss. It also proves terminal command
rejection preserves the complete state hash and RNG draw count. Calculator unit
tests pin the `648`/`720` ordinary-damage vectors and additive healing/flooring
boundary.

The canonical codec includes each unit's entry wave, current/total encounter
wave and the operation sequence allocator. Earlier B3/B4 hashes were refreshed
because those fields are now authoritative; no catalog definition body or
event log was added to state hashing.
