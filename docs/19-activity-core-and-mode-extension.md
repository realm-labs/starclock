# Activity Core and Mode Extension

This document is normative for every layer above a single battle. It replaces separate generic `run-core` and `challenge-core` state machines with one deterministic `starclock-activity`. Standard encounters, rotating challenges, Simulated Universe families, combat events, survival modes, boss rushes, drafting modes, and future rule sets are profiles over the same activity model.

## Design objective

`starclock-combat` answers one question: given a validated `BattleSpec`, state, seed, and legal command, what ordered battle events and new state result?

`starclock-activity` answers a different question: which battles and nonbattle decisions occur, what participants/resources/clocks/scores persist between them, and when is the activity complete?

```text
                       starclock-combat
              BattleSpec |     | BattleResult
                         v     v
                       starclock-activity
          graph / scopes / roster / clocks / metrics /
          objectives / decisions / persistence / hash
             ^               ^                ^
             |               |                |
      starclock-mode-standard   starclock-mode-challenge   starclock-mode-universe
             \               |                /
                      starclock-mode-event/future
```

Mode crates provide definitions, rule bundles, validation, and optional statically registered extensions. They do not own another command processor, RNG implementation, replay format, or battle-result protocol.

## Public activity API

Mutation remains command-based:

```rust
pub struct ActivityResolution {
    pub events: Vec<ActivityEvent>,
    pub next_decision: ActivityDecisionPoint,
    pub battle_spec: Option<BattleSpec>,
    pub state_hash: [u8; 32],
}

impl Activity {
    pub fn apply(
        &mut self,
        command: ActivityCommand,
    ) -> Result<ActivityResolution, ActivityCommandError>;
}
```

Initial command families are:

- select an offered option or route;
- configure a roster/team/loadout within the current policy;
- buy, sell, enhance, replace, or discard an offered item/modifier;
- start the pending battle;
- submit a verified `BattleResult`;
- submit an offered external outcome for a presentation/minigame node;
- retry, resume from an authored checkpoint, or abandon when offered.

Invalid commands preserve the complete activity hash. Accepted commands settle synchronously to another decision, a pending `BattleSpec`, completion, failure, or `Faulted`.

## Activity graph

An `ActivityDefinition` contains a directed, validated flow graph. Nodes are immutable definitions; `ActivityState` records instances, visits, decisions, and scoped data.

Initial node kinds are:

| Node | Responsibility |
|---|---|
| `Battle` | Bind an encounter/template, participants, clocks/rules, and result projection into a `BattleSpec`. |
| `Choice` | Offer deterministic authored options and apply the chosen activity program. |
| `Reward` | Generate/select activity-owned rewards or modifiers. |
| `Shop` | Offer bounded purchases, upgrades, replacement, or services. |
| `Roster` | Draft, borrow, ban, lock, swap, deploy, or transform participants/loadouts. |
| `ExternalOutcome` | Accept one offered result from a UI/action minigame without simulating presentation. |
| `Checkpoint` | Capture an explicitly restorable activity snapshot. |
| `ForkJoin` | Run logically independent child branches and merge results in stable branch order. |
| `Terminal` | Complete, fail, or abandon with a typed outcome. |

Edges declare condition, priority, stable edge ID, consumption/once policy, and visit budget. Loops are valid only with explicit per-node and total activity visit limits. No node executes arbitrary code from data.

`ForkJoin` is logical, not shared-state parallel mutation. Child branches receive declared snapshots/substreams; results merge by stable branch ID through an authored policy. Implementations may execute isolated battles concurrently only when canonical results are identical to sequential execution.

## Generic scope model

All activity state uses one lifetime hierarchy:

```text
Activity -> Section -> Node -> Attempt -> Battle -> Wave -> Turn -> Action -> Hit
```

- `Activity` is one complete standard/challenge/run/event session.
- `Section` represents a challenge stage, universe plane, event round, or boss-rush bracket.
- `Node` represents a challenge side, universe domain, shop, choice, or battle location.
- `Attempt` separates retry-local state from state retained by the activity.

Mode-facing names such as Run, Plane, Domain, Stage, or Side are authoring aliases with metadata; they map to these generic scopes before runtime. The rule interpreter and state hash use generic scope IDs only.

A state slot declares value type, owner scope, default, bounds, reset/carry policy, visibility, and provenance. A shorter-lived scope cannot write directly into a longer-lived slot unless an explicit projection/aggregation operation permits it.

## Typed activity state

Activity data is not an unrestricted `HashMap<String, Value>`. Each `ActivitySlotDefinition` has a stable typed ID and one value domain:

- bounded signed/unsigned integer;
- fixed scalar, ratio, probability, or Action Value;
- boolean or small enum;
- stable content/participant/node ID;
- optional ID;
- ordered bounded ID set/list;
- bounded counter map whose key/value domains are declared.

Runtime storage may use a tagged `ActivityValue` union internally, but catalog validation ensures every read, write, comparison, hash, and projection matches the slot definition.

## Participants, rosters, and loadouts

`ParticipantPool` separates available combat forms from deployed battle units. A roster policy declares:

- minimum/maximum teams and slots;
- uniqueness scope: team, node, section, or whole activity;
- fixed, owned, trial, borrowed, drafted, generated, or transformed participants;
- eligibility predicates over path, element, tags, rarity, or authored groups;
- ban, pick, reserve, substitution, and ordering rules;
- loadout source and lock boundary;
- whether a form/loadout can change between nodes or attempts;
- deployment/formation constraints and support slots.

The activity layer validates participant identity and whole-loadout lock/replacement boundaries, but treats each `ResolvedCombatantSpec` and build digest as opaque. Exact `CombatantBuildSpec` editing and Trace/Eidolon/Light Cone/relic compilation belong to `starclock-build` as defined in [Character builds, Traces, and equipment](21-build-traces-and-equipment.md). A mode/application compiles or selects resolved specs before binding a Battle node; neither `starclock-activity` nor `starclock-combat` interprets peripheral build fields. Future drafting, trial-character events, restricted rosters, team rotation, and borrowed supports therefore do not require changes to battle formulas.

## Persistence and carry policies

Every projected value between scopes or battles selects one policy:

- `Reset` to the destination default;
- `CarryExact`;
- `CarryClamped` to destination bounds;
- `Project` through a typed expression;
- `Accumulate` through Sum/Min/Max/ordered append with caps;
- `Replace` by priority and stable source ID;
- `Snapshot` at a named boundary;
- `Discard` after audit events are emitted.

Policies apply independently to HP, Energy, resources, effects, cooldowns, roster state, clocks, metrics, mode modifiers, and custom slots. No mode receives a blanket “persist battle state” switch.

## Clocks, metrics, scores, and objectives

`ActivityClock` supports cycles, remaining/elapsed Action Value, action counts, turn counts, wave counts, and bounded authored counters. It declares scope, initialization, decrement observations, reset/carry, expiry timing, and terminal/tick program.

Battle-visible clocks compile into a battle-owned clock binding in `BattleSpec`. The final typed value returns through `BattleResultProjection`; activity state is never mutated from inside a battle.

`MetricDefinition` declares a typed accumulator sourced from specific battle/activity events or state projections. `ScoreProgram` combines metrics using checked fixed-point/integer expressions, caps, and explicit rounding. `ObjectiveDefinition` evaluates typed predicates at declared boundaries.

This supports cycle stars, Pure Fiction points, Apocalyptic Shadow boss progress, survival time, damage races, target protection, limited actions, combo/tally events, boss rush totals, and event-specific secondary objectives without introducing mode branches into `starclock-combat`.

## Spawn and encounter programs

`EncounterDefinition` still owns ordinary waves and enemies. An activity Battle node may additionally bind a `SpawnProgram` for continuous refill, endless/generated groups, escalating difficulty, or boss sequences.

A spawn program declares finite source pools/generators, deterministic ordering and RNG, simultaneous capacity, refill boundary, escalation state, termination, and hard budgets. “Endless” means bounded by a clock/score/maximum-spawn budget for one simulation; truly unbounded resolution is invalid.

## Battle handoff

Activity-to-battle handoff is the only orchestration boundary:

```text
ActivityState
  -> validated participant/loadout binding
  -> encounter + difficulty + SpawnProgram
  -> battle-scoped RuleBundle and clock/metric bindings
  -> derived battle RNG seed
  -> immutable BattleSpec

BattleResult
  -> identity/hash verification
  -> typed metrics and projected state
  -> carry/aggregation policies
  -> activity events and next graph transition
```

`BattleResultProjection` is declared before battle start. A result cannot return arbitrary activity mutations or undeclared metrics. Activity validation rejects a result whose activity/node/attempt/battle sequence, config digest, spec digest, seed derivation, or final hash differs.

## Activity programs and extensions

Activity nodes use a typed operation IR parallel in discipline to the battle rule IR. Initial operations include slot/resource changes, option generation, graph transition, roster/loadout mutation, modifier inventory changes, clock/metric/objective updates, BattleSpec request, checkpoint, and terminal outcome.

Mode-native extensions use a static `ActivityHandlerId` registry. A handler receives read-only validated context and returns ordinary activity operations/options. It cannot mutate state, launch a battle directly, call external services, use untracked RNG, or bypass graph/visit/slot/result validation.

Do not add a new core node kind for one event until composition plus a registered handler has proven insufficient. Do not add `if mode_id == ...` to `starclock-activity`.

## Mode profiles

Mode crates are composition libraries and validators:

- `starclock-mode-standard`: one Battle node for story, farming, weekly boss, tutorial, and synthetic scenarios.
- `starclock-mode-challenge`: Forgotten Hall, Memory of Chaos, Pure Fiction, Apocalyptic Shadow, and future fixed-stage/score/boss-rush profiles.
- `starclock-mode-universe`: Standard SU, Swarm Disaster, Gold and Gears, Unknowable Domain, and current Divergent Universe.
- `starclock-mode-event`: reusable limited-event profiles such as survival, defense, trial roster, drafting, branching battles, escalating waves, and custom objectives.
- future modes such as auto-battler/team-building or tournament structures add a profile and, only if necessary, isolated extension handlers.

A profile may define authoring aliases and focused Excel tables, but it compiles to generic activity graph/state/operations plus normal battle rules.

## Capability boundary

The architecture must support without changing `starclock-combat`:

- single/multi-wave and multi-phase battles;
- sequential or logically forked multi-team nodes;
- shared or independent clocks, scores, and objectives;
- continuous/endless-under-budget spawns, survival, defense, and boss rush;
- branching routes, shops, rewards, upgrades, currencies, and meta inputs;
- trial/borrowed/drafted/banned participants and locked/swappable loadouts;
- per-node selectable buffs and seasonal rule bundles;
- configurable state/resource carry between battles;
- external minigame outcomes and deterministic baseline automation.

Network authority, real-time physics/action gameplay, account inventory/rewards, matchmaking, and unrestricted user scripting remain outside scope. They may submit validated commands/outcomes through adapters but cannot become authoritative activity inputs implicitly.

## Determinism and hashing

One activity is logically single-threaded. The master seed derives labeled graph, reward, shop, spawn, external-outcome-test, and per-battle streams from activity/node/attempt/battle sequence identities. A draw in one stream cannot shift another.

The canonical activity hash includes definition/config digests, phase, graph position and visit counts, scopes/slots, participants/loadouts/locks, modifier inventories, clocks, metrics, objectives, RNG streams/draw indexes, pending decisions/options, pending `BattleSpec` identity, checkpoints, and completed result digests. Caches, presentation, wall-clock schedules, and account state are excluded.

## Crate boundaries

The target workspace is:

```text
starclock-combat       single-battle state machine and domain definitions
starclock-build        progression/equipment definitions -> ResolvedCombatantSpec
starclock-activity     generic cross-battle workflow and BattleSpec/Result orchestration
starclock-data       Sora records -> validated combat/build/activity catalogs
starclock-rules     static battle/activity native-handler registries
starclock-mode-standard     ordinary encounter profiles
starclock-mode-challenge    fixed-stage, scoring, seasonal challenge profiles
starclock-mode-universe     roguelike universe profiles and unique content types
starclock-mode-event        reusable event/profile extensions when implementation begins
starclock-ai         deterministic battle and activity controllers
starclock-replay     canonical battle/activity codec and verifier
starclock-cli           headless orchestration and diagnostics
engine adapters   Bevy/other presentation integration
```

`run-core` and `challenge-core` are not separate target crates. Their generic responsibilities belong to `starclock-activity`; universe/challenge terminology stays in mode definitions and user-facing APIs only.

## Required tests

- every node/edge graph has valid entry, terminal reachability, and bounded loops;
- invalid commands and rejected `BattleResult` values preserve the activity hash;
- generic scopes reset/carry exactly at Activity/Section/Node/Attempt/Battle boundaries;
- roster uniqueness, trial/borrow/draft, loadout locks, and substitutions are deterministic;
- clock/metric/objective bindings agree between battle projection and activity aggregation;
- isolated branch execution order does not affect merged result/hash;
- retries and checkpoints restore only authored state and derive distinct or reused RNG as declared;
- one golden profile covers Standard, each classic challenge family, each universe family, survival, boss rush, drafting/trial roster, and continuous spawn;
- adding a new data-only profile requires no `starclock-combat` change;
- platform golden tests compare activity hashes after every accepted command and battle submission.
