# Universe Mode Profiles

This document defines how permanent Simulated Universe families compile into the generic [activity core](19-activity-core-and-mode-extension.md). It covers mechanical state and decisions, not story presentation, account rewards, or action-minigame reproduction.

## Architecture boundary

Universe modes do not own a second run state machine:

```text
Universe.xlsx + generic Activity.xlsx
                 |
                 v
             mode-universe
                 |
       ActivityDefinition + mode components
                 |
                 v
             activity-core ---- BattleSpec/Result ---- combat-core
```

`activity-core` owns graph traversal, generic scopes/slots, participants, inventories, clocks, metrics, decisions, RNG streams, battle handoff, checkpoints, and completion. `mode-universe` owns universe terminology, focused data types, profile compilation, and validation for Audience Dice, Scepters, Equations, and similar systems.

Mode code may produce generic activity operations and battle `RuleBundle` entries. It may not fork graph execution, command atomicity, formulas, effects, timeline, RNG, replay, or hashing.

## Scope mapping

Universe authoring names map to generic scopes:

| Universe concept | Runtime scope |
|---|---|
| one complete run | `Activity` |
| plane/stage | `Section` |
| domain/location | `Node` |
| retry of a node | `Attempt` |
| combat | `Battle` and shorter combat scopes |

Run-scope blessings, curios, equations, currencies, and progression are typed Activity slots/inventories. Plane countdowns and map effects are Section state. Domain beacons/dice effects are Node state. Battle-visible effects compile into a battle-scoped `RuleBundle` and never reach back into live activity state.

## Graph and decisions

Linear Standard SU worlds and map-based expansions are graph-generator/profile differences over the same immutable ActivityNode/ActivityEdge model.

Universe decisions use generic activity commands to:

- choose path/alignment/initial loadout;
- choose the next visible domain/node;
- select, replace, enhance, discard, or purchase a modifier;
- choose an occurrence branch;
- reroll, cheat, configure, or accept a die result;
- insert, synthesize, or upgrade a component/mode item;
- use a workbench, respite, shop, downloader, revival, or roster node;
- submit an offered abstract Adventure/minigame outcome;
- start a pending battle and submit its verified result.

Activity decisions expose canonical legal options. Universe code cannot accept arbitrary resource deltas or node IDs from a caller.

## Battle handoff

A universe Battle node binds encounter, party/loadouts, difficulty, initial resources, environment, mode rules, clocks/metrics, and a derived battle seed into an immutable `BattleSpec`.

The declared `BattleResultProjection` returns only allowed survival state, mode tallies, metrics, outcome, and hashes. Activity carry policies determine whether HP, Energy, defeated participants, consumable charges, or other values persist. Combat never rolls run rewards or mutates universe state directly.

## RNG isolation

The activity master seed derives graph, reward, shop, occurrence/dice, spawn, and per-battle streams using the canonical activity/node/attempt/battle identity. Candidate sets are ordered by stable IDs before selection. Adding a reward roll cannot shift later combat CRIT rolls.

Mode-specific randomness requests a labeled activity stream; it does not create its own RNG implementation.

## Abstract noncombat outcomes

Adventure and action-minigame domains are `ExternalOutcome` nodes. The caller submits one offered outcome ID such as success tier, timeout, or failure. The activity applies its program and records the result in the replay.

Movement, aiming, physics, timing input, and presentation are not simulated. Tests and baseline AI use a configured deterministic outcome policy; an interactive adapter may obtain the same result from a separate minigame.

## Shared universe content

Generic content includes blessing, enhanced blessing, curio with state/charges, occurrence/choice, path/alignment, resonance/active ability, currency, shop offer, domain modifier/beacon, difficulty modifier, meta-progression effect, and battle rule bundle.

Definitions may own Activity/Section/Node slots and contribute Battle rules. Teardown, replacement, carry, and scope are explicit. Unique universe records reference generic activity IDs rather than duplicating graph, choice, inventory, clock, or objective structures.

## Supported families

| Family | Required profile/components |
|---|---|
| Standard Simulated Universe | Worlds/difficulties, linear domains, path selection, blessings/enhancements, curios/states, Cosmic Fragments, shops/respite/revival, Path Resonance/formations, occurrences, roster download, and battle-affecting Ability Tree input. |
| Swarm Disaster | Three-plane map, domains/beacons, countdown and Planar Disarray, Audience Die by path, reroll/cheat, Communing Trail mechanical input, Pathstrider input, resonance interplay, boss-choice consequences, and Swarm content pools. |
| Gold and Gears | Plane map, Cognition/intra-cognition, customizable dice and six face slots, dice categories/passives, Knowledge, Secrets/Neural Network input, Conundrum, resonance extrapolation, Adventure outcomes, and mode pools. |
| Unknowable Domain | Extrapolation Alignment, Scepters, activation/charge/speed behavior, Component shapes/slots, Decision Components, insertion, synthesis/upgrades, stages/difficulties, and mode curios/occurrences. |
| Divergent Universe, current 4.4 snapshot | Equations/thresholds, blessings, curios/Weighted Curios, occurrences, workbench transformations, Synchronicity mechanical input, Inspiration where applicable, Golden Blood's Boon, Astronomical Division/protocols, Stable Computing Arrays, save-file/loadout snapshots, and active pools. |

Official announcements establish the permanent boundaries and headline systems: [Swarm Disaster](https://www.hoyolab.com/article/21275174), [Gold and Gears](https://www.hoyolab.com/article/23850968), [Unknowable Domain](https://www.hoyolab.com/article/34422433), and the [Protean Hero update](https://www.hoyolab.com/article/38864547). Exact effects and values require row-level provenance and live-version verification.

Currency Wars and historical Divergent Universe data are outside the current import target, but no longer outside the architectural capability target. A future auto-battler/team-building profile should reuse Activity participants, inventories, graph, metrics, and Battle nodes, adding isolated mode components/handlers only where its combat model genuinely differs.

Story dialogue, index/achievement rewards, weekly/account payouts, and collection completion are excluded. Meta-progression is included only when it changes activity decisions or combat.

## Profile extensions

`mode-universe` may:

- register focused schemas and compilers for unique content types;
- generate graph annotations, decisions, modifier inventories, and reward pools;
- contribute validated activity programs and battle rules;
- define authoring aliases for Run/Plane/Domain;
- validate cross-table mode invariants and BattleResult projections;
- register a static activity handler when generic operations cannot express a mechanic reasonably.

It may not introduce `UniverseActivity::apply`, a universe-only replay, untracked state, generic string variables, or a second BattleSpec protocol.

## Baseline controller

The generic deterministic activity controller scores legal options using authored tags, party synergy, resource delta, risk, alignment/path focus, and progress. A universe profile supplies scoring hints, not another controller protocol. Scores use fixed-point/integer values; ties use stable option IDs.

For abstract outcomes the controller selects only an offered fixture outcome. Every selection and optional score breakdown can be replay diagnostics.

## Validation and tests

- generated graphs have valid entry, terminal reachability, and bounded loops;
- universe aliases map unambiguously to Activity/Section/Node scopes;
- every decision has a legal option or typed terminal/fault transition;
- currencies, modifier inventories, dice/components, and unique items obey caps, ownership, replacement, and carry policies;
- BattleSpec/Result identities and projections are complete;
- graph/reward/battle RNG isolation passes perturbation tests;
- each family has a seeded end-to-end golden activity covering its primary subsystem;
- adding a universe data-only profile does not change `activity-core` or `combat-core`;
- enabled content reaches 100% of its frozen manifest.
