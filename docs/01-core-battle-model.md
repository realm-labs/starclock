# Core Battle Model

## Design objective

Combat is a deterministic state machine. A caller submits a legal command; the core resolves it into an ordered stream of domain events. An engine may visualize those events, but animation timing must never decide game state.

```text
BattleConfig + Seed
        |
        v
  BattleState <--- Command
        |
        v
 Resolver / trigger queue
        |
        +----> BattleEvent[]
        |
        v
  New BattleState
```

## Root state

`BattleState` should own, at minimum:

- encounter phase: initializing, awaiting command, resolving, won, lost, or faulted;
- current wave and pending waves;
- ordered player and enemy formations;
- all active units and optional timeline actors;
- current Action Gauge for every timeline actor;
- team Skill Points;
- active effects, shields, weaknesses, Toughness, and resources;
- interrupt/follow-up queue and trigger recursion guards;
- deterministic RNG state;
- monotonically increasing event and action IDs.

A unit needs a stable `UnitId` independent of its position. Formation indices are mutable spatial metadata used by Blast adjacency and presentation.

## Unit state

Separate authored data from mutable battle state.

Authored data includes level, element, path/archetype, base stats, maximum resources, ability definitions, innate weaknesses/resistances for enemies, and behavior rules. Runtime state includes current HP, Energy, Toughness layers, Action Gauge, effect instances, shield instances, counters, life state, and battlefield presence.

Life and presence are independent axes. Life is `Alive`, `Downed`, or `Defeated`. Presence is `Present`, `Reserved`, `Departed`, `Untargetable`, `Linked`, or `Transformed`. A linked actor can be alive without occupying a formation slot; a boss can be untargetable while phase-transition actors resolve; a defeated unit can remain recorded for revival. Legal-action, target, and timeline queries must test both axes rather than infer them from HP alone.

Do not store a single permanently calculated stat block. Derived stats must be recomputed through a stat query or invalidated cache because effects can modify base values, percentages, flat values, and conditional final values at different stages.

## Element and damage classification

The seven base combat elements are Physical, Fire, Ice, Lightning, Wind, Quantum, and Imaginary.

Keep element separate from damage tags. One damage instance can be, for example:

- `element = Lightning`;
- `ability_kind = Ultimate`;
- tags containing `Attack`, `FollowUp`, and perhaps `AdditionalDamage` only when explicitly applicable.

This separation is required because bonuses and vulnerabilities query different dimensions. A Follow-Up ATK may simultaneously count as Ultimate DMG for a character-specific effect; tags should support that without changing the universal formula.

Recommended base damage classes:

- ordinary direct damage;
- ordinary DoT;
- additional damage;
- Break damage / Break DoT / Break additional damage;
- Super Break damage;
- true damage.

## Ability and target patterns

The core patterns are:

| Pattern | Resolution |
|---|---|
| Single target | One selected legal target. |
| Blast | A primary target plus its currently valid adjacent formation neighbors. Primary and adjacent coefficients may differ. |
| AoE | Every legal target on the opposing or allied side. |
| Bounce | A configured number of hits; each hit selects from the current legal pool, commonly at random. Repeats are allowed unless the ability says otherwise. |
| Support | One or more allies selected by the ability's targeting rule. |
| Enhance | Changes the user or an ability and may not select another unit. |

Target selection and effect resolution must be separate. Resolve a `TargetSet` first, then emit effect operations. If a primary target dies mid-action, do not silently retarget remaining committed hits unless the authored ability explicitly has a retarget policy.

## Action and hit decomposition

Model the hierarchy explicitly:

```text
Action
  -> ability phases
      -> target groups
          -> hits
              -> damage / toughness / effect applications
```

Multi-hit attacks distribute authored ratios of total damage, Toughness reduction, and sometimes Energy across hits. Each hit makes its own CRIT roll. This behavior is documented by the community [Damage reference](https://honkai-star-rail.fandom.com/wiki/Damage#Hit_Split).

Do not infer hit ratios from animation count. They are ability data.

## Resolution transaction

For an ordinary attacking action, use the following default sequence, while allowing authored phases to insert operations:

1. Validate actor, action availability, cost, and selected target.
2. Reserve or spend action costs.
3. emit `ActionStarted`.
4. Resolve each hit in authored order:
   1. choose/confirm targets;
   2. calculate and apply HP damage;
   3. calculate and apply Toughness reduction when eligible;
   4. resolve Weakness Break immediately if Toughness reaches zero;
   5. apply on-hit effects and generate hit-level events;
   6. enqueue reactions made eligible by this hit.
5. Resolve post-hit and post-attack triggers according to priority.
6. Generate authored Energy and Skill Points at their defined timing.
7. emit `ActionFinished`.
8. If this was a normal turn action, perform turn-end processing and reset timeline progress.
9. Drain queued follow-ups/interrupts before advancing timeline time.
10. After each state-changing atomic operation, settle defeat replacements and update terminal-candidate flags. Do not spawn the next wave in the middle of a committed action unless the encounter or ability explicitly selects a nondefault boundary.
11. At the action boundary, resolve the configured wave transition, then battle termination, and expose the next decision point.

The exact position of a kit-specific effect must be authored as a trigger phase rather than embedded in the global sequence.

## Trigger vocabulary

A compact but expressive initial set is:

- battle/wave start and end;
- turn start, before action, after action, turn end;
- action selected, action started, hit started, damage calculated, damage applied, hit ended, action finished;
- attacked, hit, HP damaged, shield damaged, healed;
- Toughness damaged, Weakness Broken, recovered from break;
- effect applying/applied/refreshed/removed/expired;
- unit defeated, enemy defeated, ally defeated;
- Skill Point or Energy changed.

Distinguish `Attacked`, `Hit`, and `HPDamaged`. A shielded attack may hit without reducing HP; Break damage is damage but is documented as not itself being an attack. This distinction prevents incorrect counters and on-hit triggers.

## Defeat, waves, and termination

After any damage or HP consumption:

1. clamp current HP to its legal range;
2. resolve explicit death-prevention/revive replacements;
3. mark unresolved zero-HP units downed;
4. emit defeat events and remove them from legal target/timeline pools;
5. resolve defeat triggers;
6. if no enemies remain, mark a pending wave transition or victory candidate;
7. if no controllable allied combatants remain, lose.

The default wave boundary is `AfterAction`. `AfterHit`, `AfterPhase`, and `Explicit` are opt-in policies for documented cross-wave or scripted abilities. Wave initialization is an encounter concern layered on the same combat state. It may add units and run `WaveStarted` triggers, but should not reconstruct surviving allies or reset resources unless the encounter configuration says so.

## Explicit project policies

These rules are implementation policies until golden observations establish stricter fidelity:

- no floating-point equality is used to settle ordering; stable IDs break exact ties;
- all target pools are sorted by stable formation order before an RNG index is drawn;
- command legality errors leave the state unchanged; an internal numeric, budget, or invariant failure either commits a stable fault after completed atomic operations or rolls back the uncommitted journal and then commits `Faulted` from the pre-command state;
- trigger recursion has fixed rules-revision budgets and faults deterministically instead of hanging;
- invalidated queued actions are canceled with an event, not silently dropped;
- final display rounding is outside the simulation, while state-changing integer amounts use one documented rounding policy covered by tests.

The normative ordering, wave, fault, and event-cause rules are specified in [Lifecycle and resolution](10-lifecycle-and-resolution.md).

The concrete Rust ownership, identity, store, lowering, operation, transaction, and module contracts are specified in [Core implementation design](20-core-implementation-design.md).
