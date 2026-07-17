# Enemy AI and Encounters

This document defines executable enemy content, deterministic enemy decisions, boss phases, variants, and encounter assembly. Enemy animation and story presentation remain outside the combat core.

## Enemy definition

An `EnemyDefinition` contains:

- stable variant ID and bilingual metadata;
- rank/classification and level/stat curve;
- element, weaknesses, base resistances, control/debuff resistances, and aggro behavior;
- one or more ordered Toughness layers;
- abilities and passive rule definitions;
- AI graph and initial state;
- summon, phase, transformation, and teardown references;
- provenance and Version 4.4 coverage state.

A visually similar enemy with different stats, weaknesses, abilities, AI, or mode behavior is a separate variant definition. Encounters reference exact variants; they do not patch an ambiguous base enemy at runtime.

## Enemy abilities

Enemy abilities use the same target programs, hit plans, operations, effects, rule IR, and action envelope as player abilities. They additionally declare telegraph state, cooldown, charge/setup requirements, phase availability, AI tags, and fallback action.

An enemy ability cannot encode target selection in presentation text. Every random or weighted target choice records candidate order, weights, draw purpose, and fallback. Scripted target locks and taunts precede ordinary AI selection according to [Effects and resources](05-effects-and-resources.md).

## Deterministic AI graph

An `AiGraph` is a finite state machine. Each state provides ordered action candidates and ordered transitions:

```text
AiState
  entry program
  candidate ability + condition + target policy + priority
  post-action transition + condition + priority
  turn counter/reset policy
```

At an enemy decision:

1. run legal automatic transitions until stable, under a transition budget;
2. collect legal ability candidates whose conditions pass;
3. sort by priority and stable ability/candidate IDs;
4. select the first deterministic candidate, or perform an explicitly authored weighted draw;
5. resolve its target program;
6. if no target exists, use the candidate's declared fallback or the graph's mandatory fallback;
7. log selected graph state, candidate, target decision, and RNG trace.

Graphs cannot rely on sheet order unless that order is an explicit integer field. A graph with a reachable state lacking a legal fallback is invalid.

## Boss phases

A boss phase definition declares entry/exit conditions, replacement priority, ability/AI set, Toughness layers, targetability, linked actors, field rules, and carryover policy for:

- HP and maximum HP;
- action gauge and queued actions;
- effects, debuffs, marks, and target locks;
- Toughness/broken state;
- summons and phase-owned timeline actors;
- per-phase and per-battle counters.

Phase transitions are replacement work at the lifecycle boundary. They may make the boss temporarily untargetable, queue a transition action, transform the same unit, or replace it with a linked variant. The data must choose one; global code does not infer a model from rank or HP thresholds.

## Summons and linked actors

Enemy summons use stable unit IDs and explicit owner/phase links. Definitions declare slot selection, maximum simultaneous count, overflow behavior, initial action gauge, owner-death behavior, wave persistence, and whether their defeat contributes to encounter victory.

Countdowns, hands/parts, shared-HP components, and untargetable mechanics use linked actors rather than hidden timers when they need timeline or target semantics. Shared HP and damage transfer are authored resolver operations with unambiguous event credit.

## Encounter definition

An `EncounterDefinition` contains:

- stable ID, level/difficulty parameters, environment and mode rule references;
- player-team constraints and initial resources;
- ordered waves and formation slots;
- wave transition policy;
- victory, loss, turn/cycle, and optional score conditions;
- persistence/reset policy between waves;
- encounter clock actors and scripted boundary programs;
- provenance and coverage state.

Each wave lists exact enemy variant IDs, spawn sequence, formation index, level override if allowed, and optional phase initialization. Formation overflow or duplicate exclusive slots is invalid.

## Encounter clocks and mode overrides

Cycles and countdowns are separate actors or run/encounter clocks; they never alter the base action-gauge formula. A mode may inject a `RuleBundle`, initial effects, difficulty modifiers, enemy replacements, or completion rules. It may not mutate hidden combat state outside commands and operations.

Encounter completion produces a `BattleResult` containing outcome, state fields and typed metrics declared by its projection, event/state hash, and terminal fault information. Reward generation and cross-battle aggregation belong to `starclock-activity` plus the selected mode profile, not the combat resolver.

An ordinary encounter is bound by a one-Battle-node Standard activity. Multi-node Memory of Chaos, Pure Fiction, and Apocalyptic Shadow stages are Activity profiles; their participants, clocks, metrics, objectives, spawn programs, and seasonal rules are defined in [Standard battle and challenge modes](18-standard-and-challenge-modes.md), not in enemy AI.

## Data tables

The Excel/Sora boundary needs focused tables for Enemy, EnemyStat, EnemyResistance, EnemyAbility, AiGraph, AiState, AiCandidate, AiTransition, EnemyPhase, EnemyLink, Encounter, Wave, and WaveSlot. Large AI programs and phases are child rows, not JSON cells.

Every numerical ability field includes level/variant context and provenance. Raw extracted descriptions and assets are not required or committed.

## Coverage policy

The Version 4.4 enemy manifest enumerates every public enemy and mechanically distinct variant. Terminal statuses are:

- `Documented`: identity and public behavior summarized, not executable;
- `Researching`: exact values or ordering remain unresolved;
- `DataReady`: complete validated definition exists;
- `GoldenVerified`: representative commands match captured observations;
- `Disabled`: official public placeholder that cannot enter normal encounters.

“Enemy name listed” and “all enemies supported” are not synonyms. Completion is 100% manifest accounting plus `DataReady` for every enabled entry; golden verification is tracked independently.

The prepared [Version 4.4 content reference pack](content-reference/README.md)
currently supplies 613 enemy templates, 2,591 variants, 3,611 abilities, pinned
AI/config evidence, and 1,471 ordinary encounter candidates. These are
pre-implementation facts, not validated runtime definitions; the Goal 01
`standard-v1` subset still requires Rule IR lowering and golden fixtures.

## Required tests

- graph candidate/transition order is stable under row/container reordering;
- every reachable AI state has a legal deterministic fallback;
- taunt, scripted locks, aggro, random selection, and no-target behavior use documented precedence;
- summon overflow, owner defeat, shared HP, and linked teardown are covered;
- boss transitions test every carryover field;
- multi-wave actions obey encounter transition policy;
- encounter victory ignores non-victory-linked actors and includes required summons;
- seeded enemy turns and complete encounters reproduce event/state hashes.
