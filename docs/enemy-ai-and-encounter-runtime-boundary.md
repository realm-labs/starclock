# Enemy AI and encounter runtime boundary

`G01-P4-B9` makes the enemy and encounter contracts executable without adding
enemy IDs or mode rules to `starclock-combat`.

## Definition ownership

`CombatCatalog` owns finite `AiGraphDefinition` records and exact enemy,
phase, link, wave and wave-slot definitions. Builders canonicalize explicit
priority/ID keys and reject missing abilities, selectors, programs, graph
targets, unreachable states, automatic-transition cycles, invalid initial
phases, duplicate formations and missing enemy references. Legacy synthetic
encounters retain wildcard slots only for compatibility; authored waves bind an
exact enemy occurrence to one formation index and optional initial phase.

`starclock-ai` owns controller state and behavior RNG. An `EnemyController`
stabilizes automatic transitions under the authored budget, evaluates
canonical candidate priorities, performs only declared weighted draws, applies
every no-target policy and returns an exact command already offered by the
battle. Per-enemy graph/state/turn cursors advance through explicit
post-action/phase settlement calls. The controller cannot mutate battle state
or synthesize target commitments.

## Authoritative encounter state

Battle construction resolves each hostile participant against its exact wave
slot. The retained occurrence records its enemy definition, current phase and
phase-selected initial AI cursor so immutable views can bootstrap or reset a
controller after a phase replacement. AI choices remain controller-owned;
accepted commands are still the only external mutation input.

Wave advancement is selected by the encounter's `AfterHit`, `AfterPhase`,
`AfterAction` or `Explicit` policy. The resolver refuses a transition at any
other boundary, emits `WaveEnded` before link/departure/carry work, activates
the next exact wave, then emits `WaveStarted`. HP, Energy, Skill Points,
effects, Action Gauge and keyed team-resource carry policies are applied in the
same rollback-safe transaction.

`TransitionEnemyPhase` addresses an exact phase ID. It accepts only the next
ordered phase, applies HP, Action Gauge, effect, Toughness and linked-entity
carry policies, selects targetability and the next graph/state, and emits one
typed `EnemyPhase::Transitioned` fact. `ReplaceLinkedVariant` changes the
generic form/ability definition through the same unit-replacement transaction;
the resolver never branches on a concrete enemy.

## Compatibility decision

Initial team resources and enemy definition/AI/phase cursors are authoritative
inputs to later wave and controller behavior. Their canonical encoding changes
the state byte layout, so this batch advances `state_hash_revision` from
`sha256-v1` to `sha256-v2`. Replay and benchmark golden hashes are reblessed;
command counts, replay framing, workload shapes and performance budgets do not
change.

The Version 4.4 JSON reference pack remains evidence only. This batch adds no
runtime JSON path and promotes no character, Light Cone, enemy or Standard
manifest row to `DataReady`; production Excel/Sora population remains in its
owned content batches.
