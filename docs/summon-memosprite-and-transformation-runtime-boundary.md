# Summon, Memosprite and Transformation Runtime Boundary

This document records the implemented `G01-P4-B7` boundary. It is subordinate
to the Goal 01 plan and the normative battle, timeline, lifecycle and Rule IR
documents.

## Ownership model

A linked combatant has three distinct identities:

- an owner `UnitId`, retained for attribution and lifecycle policy;
- its own `UnitId`, used for HP, effects, targeting and action actor credit;
- an optional `TimelineActorId`, used only when the linked combatant owns an
  independent gauge or automatic ability.

Summons and memosprites therefore do not borrow the owner's unit identity or
normal turn. Automatic linked actions use the common action envelope and retain
the linked unit as actor while the cause retains the owner independently.
Countdowns are timeline-only links and have no fabricated combatant unit.

The canonical link store records entity kind, owner, active state and the
authored owner-defeat, owner-departure and wave policies. Runtime allocation is
monotonic and bounded. Formation, unit, actor, rule and link mutations are
journaled before events are emitted.

## Presence and lifecycle axes

`LifeState` and `PresenceState` remain independent. `Present`, `Linked` and
`Transformed` are active; only explicitly active primary participants count for
battle victory or loss. A linked unit cannot keep a defeated player party alive
or prevent a wave from ending.

Target selection and timeline eligibility use the closed presence predicates.
Reserved, Departed and Untargetable units cannot enter an ordinary target pool.
An authored linked formation index supplies deterministic ordering and blast
adjacency without adding a character-specific scheduler path.

Zero HP still emits `Downed` then `Defeated`. Owner-link settlement follows the
defeat event before a later explicit revive operation. Revival restores the
authored HP, life/presence and one of three declared gauge policies: preserve,
reset or immediate. Departure deactivates the linked actor and link without
deleting historical state.

## Transformation model

Transformation is an explicit reversible state on the existing unit. It saves
the original form, abilities and presence, installs the authored replacement
form and complete replacement ability set, and may create one owned countdown
actor. End policy is declared independently for defeat and wave boundaries.

Teardown deactivates the countdown/link, restores the saved form, abilities and
presence, clears the transformation record and emits exactly one
`TransformationEnded` event. Repeated teardown is rejected rather than silently
duplicating restoration. No character ID appears in this path.

The generic catalog validates every linked combatant, automatic ability,
replacement form, replacement ability and countdown ability before battle
execution. Rule IR and generated Sora readers expose typed `Summon`, `Despawn`,
`Transform`, `ReplaceAbility` and `ChangePresence` proposals. The resolver
remains the only state mutator.

## Boundary ordering

The implemented event order is:

1. allocate and insert linked state, then emit `Summoned`;
2. execute a linked automatic action through action/phase/hit boundaries;
3. on owner defeat, emit `Downed`, `Defeated`, then `LinkSettled`;
4. if authored in the same program, emit `Revived` after link settlement;
5. after the last primary enemy is defeated, emit `WaveEnded`;
6. apply link and transformation wave policies;
7. depart old-wave primary enemies and settle their owner links;
8. activate the next primary enemy set and emit `WaveStarted`.

Invalid lifecycle operations fail inside the existing rollback/fault policy.
They cannot consume RNG, reuse an identity or partially retain a transformation.

## Canonical and compatibility impact

Canonical state now includes the link store, transformation snapshots and the
timeline actor's optional unit, linked kind, automatic ability and active flag.
Existing deterministic combat and nine Phase 3 benchmark hashes are reblessed
because the encoded state shape changed. Workload identities, command counts,
replay byte counts and performance budgets are unchanged.

## V1a probe evidence

Pinned Sora 0.3.0 exports two dedicated disabled workbook scopes twice:

- `config/probes/v1a/aglaea-memosprite` binds the frozen Aglaea source payload
  and lowers memosprite summon, linked presence and departure proposals;
- `config/probes/v1a/firefly-transform` binds the frozen Firefly source payload
  and lowers ordered form, ability and transformed-presence proposals.

Both bundles pass through the generated reader and domain lowering in fixture
mode. The production loader rejects the same bundles and they grant zero
`DataReady` coverage.

The probes and runtime goldens are partial executable evidence only. The five
named Aglaea/Firefly teardown, resummon, SPD-stack and contribution observation
envelopes remain `Researching`; no missing live observation is converted into a
convenient default.
