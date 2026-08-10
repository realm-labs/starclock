# Standard Battle and Challenge Modes

This document defines the default non-roguelike battle profile and the three classic rotating challenge families: Memory of Chaos, Pure Fiction, and Apocalyptic Shadow. It specifies simulation behavior, clocks, scoring, node aggregation, and seasonal rule injection. UI, matchmaking, account rewards, unlock quests, and calendar scheduling are outside the combat runtime.

## Architecture boundary

These modes are profiles over the generic activity orchestrator rather than a separate challenge state machine:

```text
Challenge.xlsx + Activity.xlsx
              |
              v
        starclock-mode-challenge
              |
     ActivityDefinition + profile metadata
              |
              v
        starclock-activity ---- BattleSpec/Result ---- starclock-combat
```

`starclock-activity` owns team locking, section/node/attempt progression, shared or independent clocks, score/objective aggregation, decisions, result verification, and hashes. `starclock-mode-challenge` supplies Standard, Forgotten Hall/Memory of Chaos, Pure Fiction, and Apocalyptic Shadow profiles and validation. It does not fork graph execution, damage, timeline, enemy AI, effect, or Toughness logic.

A production ordinary battle is a one-Battle-node Standard activity. Formula tests and low-level tools may still construct `Battle` directly.

## Common data model

```rust
pub struct ChallengeSeasonProfile {
    pub family: ChallengeFamily,
    pub activity: ActivityDefinitionId,
    pub stages: Vec<ChallengeStageAlias>,
    pub seasonal_rules: RuleBundleId,
}

pub struct ChallengeStageAlias {
    pub section: ActivitySectionId,
    pub node_aliases: Vec<ChallengeNodeAlias>,
}
```

The profile adds user-facing names and seasonal content over generic Activity Sections/Nodes. A node binds one team slot, encounter or spawn program, initial resources, selectable buff, clock/metrics, and battle-result projection. A stage fixes loadouts and the config digest before Node 1 begins.

Ordinary stages use two teams. Starward stages use three teams and three nodes.
Both use Section-level participant uniqueness. Each Node receives its own units
and Battle state; HP, Energy, Skill Points, effects, action gauge, and RNG do
not carry between unrelated teams. Only explicitly Section-owned slots such as
remaining cycles or aggregate score carry between Nodes.

## Standard battle profile

`starclock-mode-standard` provides the default one-Battle-node profile for story, overworld, material, boss, and synthetic test encounters unless another profile is selected.

Default semantics are:

- one player team and one encounter containing one or more ordered waves;
- no global cycle/AV timeout, score, star objective, seasonal buff, or second node;
- victory when all hostile units required by the encounter are defeated;
- loss when no controllable allied combatant remains, unless an authored replacement/revival is pending;
- normal action-gauge timeline, commands, interrupts, enemy AI, waves, rewards handoff, and deterministic RNG;
- `AfterAction` wave transitions unless the encounter overrides them;
- initial Skill Points, Energy, HP, Technique entry programs, and persistence are encounter data rather than hidden mode constants.

Standard mode does not mean a single wave or weak enemies. Echo-of-War-style bosses, multi-phase story bosses, farming waves, tutorial restrictions, and original golden fixtures remain Standard encounters with explicit rules. Exploration movement, Technique Point consumption, consumables, and post-battle drops occur outside `starclock-combat`; only a validated battle-entry program and final result cross the boundary.

## Challenge clocks

Activity clocks are deterministic scoped state, not unit turns. A challenge profile binds them at Battle, Node, or Section scope and declares:

- value domain: cycle count, remaining Action Value, elapsed Action Value, or authored counter;
- ownership: battle, wave, node, or stage across nodes;
- initial value and per-boundary reset/carry policy;
- decrement points and whether allies, enemies, linked actors, extra actions, or interrupts consume it;
- expiry timing relative to queued reactions and wave/phase transitions;
- events/programs fired on tick or expiry;
- terminal behavior: fail, finalize score, or continue without an objective.

The commonly observed cycle window is 150 AV for a first wave window and 100 AV afterward, but it is a `ClockPolicy` preset, not a universal scheduler rule. A seasonal/stage record must select the preset and reset behavior explicitly.

## Forgotten Hall and Memory of Chaos

Static Forgotten Hall Memory and rotating Memory of Chaos share one challenge family. Static Memory uses stage-authored encounters/objectives without a seasonal record. Memory of Chaos additionally references a versioned season.

The current Version 4.4 Memory of Chaos profile contains 12 ordinary stages and
one selectable Starward form of Stage 12. Ordinary stages have two sequential
nodes; Starward has three. The profile requires:

- two or three sequential nodes with the same number of locked, disjoint teams;
- a stage-owned remaining-cycle budget carried through every node;
- normal defeat-all-required-enemies victory in each node;
- completion plus stage-authored star objectives, commonly survival and remaining-cycle thresholds;
- a seasonal `MemoryTurbulence` rule bundle with explicit cycle/start/end trigger timing;
- exact enemy lineups, waves, initial resources, cycle budget, objective thresholds, and turbulence values in season/stage data.

Each later node starts a new battle and timeline but receives the stage's carried
integer cycle value. Ordinary stages start at 30 cycles; the released Starward
record starts at 45 and has remaining-cycle objectives at 15 and 30 plus the
no-defeated-participant objective. Partial cycle-window reset at a node or wave
boundary is controlled by `ClockPolicy`; the combat scheduler does not infer
it. A failed node terminates the attempt while preserving a deterministic audit
result, not account rewards.

The active Memory Turbulence is executable Rule IR. The Starward Tierce row
does not expose a separate Turbulence selector, so applying the active
Turbulence to its third node is an explicit Medium-confidence `ProjectPolicy`.
Twenty-two of 41 current enemy identities still borrow named same-rank behavior
donors while retaining exact mode-owned identity, placement and projected
stats. The donor mapping is visible in configuration and must be replaced as
exact AI/ability closures are lowered.

Publisher material describes Forgotten Hall as cycle-limited and Memory of Chaos as the periodically updated harder branch. Exact historical and current thresholds are not universal rules and remain snapshot data.

## Pure Fiction

Pure Fiction is a score challenge with independently clocked nodes. The current
Version 4.4 profile contains four ordinary stages plus a selectable three-node
Starward form of Stage 4. Its profile requires:

- two ordinary or three Starward locked, disjoint teams and one selected
  `Cacophony` option per team/node;
- a seasonal `Whimsicality` rule bundle shared according to the season definition;
- continuously replenished or scripted enemy groups during a limited cycle/AV budget;
- points awarded by a typed `ScoreProgram` for damage, defeats, wave/group completion, elite/boss conditions, or season-specific events;
- each node retaining its finalized score when its clock expires or encounter terminates;
- stage score equal to the explicitly configured aggregation, normally the sum of both node scores;
- stage completion/stars evaluated from stage-authored score thresholds.

Continuous appearance is represented by `SpawnProgram`, not ordinary next-wave victory. It declares group quotas, slot refill timing, spawn ordering, maximum simultaneous enemies, end-of-pool behavior, and whether defeating a required final group can end the node before clock expiry.

Each current node has four cycles and a 40,000-point cap. Wave 1 awards 400 per
committed defeat up to 8,000; waves 2 and 3 each award floor-rounded target-HP
progress up to 16,000. Ordinary stages aggregate two nodes with
30,000/40,000/50,000/60,000 objectives. Starward aggregates three nodes with a
45,000 clear threshold and 60,000/75,000/90,000 objectives. The separate
99,000 Prismatic Star is an account/reward concern and is not an Activity star
objective.

Grit, Surging Grit and all three current Cacophonies execute through shared
Rule IR and typed resources. Unknown fixed-damage level scaling, equal-boundary
ordering, per-applier Indulgence identity and the upstream Punchline cap remain
one inspectable `ProjectPolicy`; the current deterministic substitutions are
recorded in the workbook and covered by mechanics tests.

Scoring uses applied authoritative values/events, never displayed rounded
damage. Caps and partial-credit rules belong to the season/stage score program.
Current or future changes therefore update data without changing the combat
resolver.

## Apocalyptic Shadow

Apocalyptic Shadow is a boss challenge scored using boss progress and remaining
Action Value. The current Version 4.4 profile contains four ordinary stages and
a selectable three-node Starward form of Difficulty 4. Its profile requires:

- two ordinary or three Starward locked, disjoint teams facing authored
  boss-mirage variants;
- one node-specific selected `Finality's Axiom` option per team;
- seasonal `Ruinous Embers`, boss traits, weakness/Toughness mechanics, and Axiom rule bundles;
- an Action Value timer whose decrement policy explicitly identifies which allied/enemy actions consume value;
- score finalization from boss completion/progress plus remaining AV according to a typed season/stage formula;
- separate node scores followed by configured stage aggregation and score thresholds.

The official overview states that a successful defeat receives full boss-HP credit plus remaining-AV credit, while an unsuccessful attempt receives boss-HP-progress credit only. Exact initial AV, coefficient/scale, caps, phase HP mapping, thresholds, and later revisions are data. A multi-phase boss must define which logical HP pool feeds partial-progress scoring; the engine must not sum visible phase bars heuristically.

Every node starts with 2,000 Action Value and contributes up to 4,000 points:
2,000 from floor-rounded authored boss-HP progress plus up to 2,000 remaining
AV after victory. Ordinary objectives are 4,000/5,200/6,600; Starward
objectives are 6,000/7,800/9,900.

Ruinous Embers and all nine released Finality's Axioms execute as mode-owned
Rule IR. Three limits remain explicit policy rather than claimed parity:
Knowledge and Decorum is unconditional after selection because resolved combat
specs do not retain Path, Oppose With Tenderness maps War Armor break to boss
Weakness Break, and Ruinous Embers can ready only Energy-based Ultimates. Enemy
AI/phase closure and mode-local boss stats also use named deterministic donors
until exact programs are lowered.

Boss-specific mechanics use normal enemy phases, linked actors, Toughness
layers, and rule IR. Axiom or Ruinous Embers effects do not bypass ordinary
event attribution merely because they are mode rules.

## Seasonal content separation

Stable family rules and rotating content are separate:

```text
ActivityDefinition       stable orchestration contract
ChallengeSeasonProfile  version/date identity, aliases, and global rule bundles
ActivitySection          stage scope, objectives, and aggregation
ActivityNode             side/team, encounter/spawn, selectable buff, and bindings
```

`ChallengeSeasonProfile` records game version, public season key, Activity
aliases/bindings, active mechanics, source records, and digest. Calendar dates
may be metadata but are not simulation inputs. Replays identify the
activity/profile/config digest, never “latest.” The repository retains only the
current Version 4.4 configuration; Git history is the historical record.

Season-specific names and summaries are bilingual project metadata. Long effect prose is not copied; exact coefficients, caps, durations, triggers, target filters, and score rules are normalized rows with provenance.

## Commands and results

Generic activity decisions include selecting teams/loadouts, selecting a node buff, starting the next node, and submitting the verified `BattleResult`. Interactive battle commands remain `Battle::apply` commands rather than being wrapped by the activity layer.

`ActivityResolution` returns ordered activity events, the next decision or `BattleSpec`, accumulated objectives/scores, and a canonical activity hash. A node result is accepted only when activity/profile/section/node/attempt identity, battle-spec digest, config digest, seed derivation, and final battle hash match.

Account star history, quick-clear state, rewards, stamina, and schedule rotation remain outside this API.

## Excel and Sora tables

The configuration boundary adds focused tables:

| Table | Purpose |
|---|---|
| `ActivityDefinition/Section/Node/Edge` | Generic workflow, stage/node scopes, branching, and bounded visits. |
| `ParticipantPolicy` | Team slots, Section uniqueness, loadout locks, and substitutions. |
| `ActivityClock` | Cycle/AV ownership, decrement, reset, tick, carry, and expiry. |
| `MetricObjective` | Completion, survival, clock, score terms, thresholds, caps, and rounding. |
| `SpawnProgram` / `SpawnGroup` | Continuous refill and finite group ordering. |
| `ChallengeSeason/StageAlias/NodeProfile` | Stable challenge family and rotating user-facing content over generic activity IDs. |
| `ChallengeBuffOption` | Memory Turbulence, Whimsicality, Cacophony, Ruinous Embers, and Finality's Axiom rule references. |

Do not store a score formula or spawn script as arbitrary code/JSON in one cell. It compiles from generic typed Activity child rows and shared battle/activity IR. `Challenge.xlsx` must not redefine graph, clock, metric, objective, participant, or result-projection semantics.

## Coverage boundary

The Version 4.4 content manifest tracks:

- one stable Standard profile plus representative standard encounter archetypes;
- static Forgotten Hall rules only where needed for the shared challenge contract;
- the active 4.4 Memory of Chaos, Pure Fiction, and Apocalyptic Shadow seasons, stages, nodes, buffs, objectives, score/clock programs, enemies, and encounters;
- exact provenance and coverage state for every enabled row.

Past rotating seasons and account reward schedules are excluded from active completeness. They may be retained only as archived replay dependencies.

The production adapter validates the complete challenge bundle with:

```text
starclock challenge config validate [--json]
```

Validation lowers all three profiles and compiles their complete mode-owned
combat catalogs over the production catalog. It therefore rejects missing
behavior donors, malformed encounters and invalid rule composition before a UI
creates an attempt.

## Required tests

- Standard battle has no implicit clock, score, season rule, or resource override;
- multi-wave Standard victory uses normal `AfterAction` boundaries;
- two-node roster locking rejects duplicate combat-form instances and loadout mutation;
- Memory of Chaos carries the authored stage clock while resetting node battle state;
- cycle tick/turbulence order is stable at action, wave, and node boundaries;
- Pure Fiction refill order, simultaneous defeats, score caps, timeout, and early final-group completion are deterministic;
- Apocalyptic Shadow defeated and undefeated score paths use the configured logical boss-HP progress and remaining AV;
- every selectable buff is offered only for its authored node/season;
- stage objective/score aggregation is independent of UI ordering;
- battle, node, stage, and season digests reproduce across platforms.
