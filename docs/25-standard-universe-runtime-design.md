# Standard Simulated Universe Runtime Design

This document is normative for the headless Version 4.4 main-world Standard
Simulated Universe runtime. It turns the complete Goal 03 Excel/Sora dataset
into one deterministic `starclock-activity` profile without introducing a 3D
scene, a second combat engine, or a universe-specific command processor.

The generic activity contract remains
[Activity core and mode extension](19-activity-core-and-mode-extension.md). The
frozen content denominator remains
[Standard Simulated Universe reference data](23-standard-simulated-universe-reference.md).

## Runtime outcome

```text
Universe.xlsx / UniverseBindings.xlsx / UniverseEvidence.xlsx
                         |
                 Sora 0.3.0 export
                         |
        private generated readers + validated lowering
                         |
              StandardUniverseCatalog
                         |
          starclock-mode-universe compiler
                         |
 ActivityDefinition + Activity programs + battle RuleBundles
                         |
                   starclock-activity
                         |
               BattleSpec / BattleResult
                         |
                    starclock-combat
```

`starclock-mode-universe` owns Standard Universe terminology, catalog
validation and profile compilation. `starclock-activity` owns authoritative run
state and commands. `starclock-combat` continues to own every battle mutation.
Generated Sora row types, `openpyxl`, workbook fields and normalized research
JSON are not public runtime types.

The Goal 03 universe bundle remains independently versioned. Runtime catalog
identity composes its digest with the combat/build catalog digest; loading the
universe bundle must not silently replace or reinterpret the Goal 01 bundle.

## Public boundaries

The intended application flow is:

```rust
let profile = standard_universe.compile(entry)?;
let mut activity = Activity::new(profile.definition, entry.seed)?;

let resolution = activity.apply(command)?;
if let Some(spec) = resolution.battle_spec {
    let result = battle_host.execute(spec)?;
    activity.apply(ActivityCommand::SubmitBattleResult(result))?;
}
```

The exact exported Rust names may be refined before the Goal 04 API freeze,
but these ownership rules are fixed:

- `Activity::apply` is the only authoritative run mutation boundary;
- `Battle::apply` is the only authoritative battle mutation boundary;
- mode code compiles definitions, programs and contributions but does not own
  another mutable run aggregate;
- callers select canonically ordered offered commands and cannot submit raw
  currency, reward, enemy, RNG or state deltas;
- invalid commands and rejected battle results preserve the complete activity
  hash.

## Spatial-free domain model

A Standard Universe domain is an authored finite interaction graph, not a
positioned map. Coordinates, navigation meshes, collision, line of sight,
monster patrols and real-time aggro are presentation concerns and never enter
the authoritative state.

Mode authoring may describe a domain visit using stable interaction slots:

```text
DomainVisit
  entry policy
  ordered Encounter slots
  ordered Choice/Reward/Service/Interactable slots
  exit gate
```

The profile compiler lowers that description to ordinary generic Activity
nodes. A typical combat domain becomes:

```text
DomainEntry
    |
    v
DomainHub Choice
    |-- Engage encounter A --> Battle A --> Reward A --|
    |-- Engage encounter B --> Battle B --> Reward B --|--> DomainHub
    |-- Use abstract interactable ---------------------|
    `-- Leave domain (only when the exit gate passes) --> next domain
```

The hub is a bounded loop. Each interaction owns an explicit consumed/cleared
slot and stable option ID. Mandatory encounters gate the exit; optional or
one-of-group encounters use authored policies. `OnDomainEnter` encounters skip
the hub and enter preparation immediately. Multiple monster objects are
represented by multiple encounter slots or by one multi-wave encounter, not by
world-space entities.

The runtime exposes only legal decisions such as:

```text
ChooseRoute
EngageEncounter
UsePreBattleTechnique
UseService
ChooseOccurrenceOption
ClaimReward
LeaveDomain
SubmitExternalOutcome
```

An engine adapter may map walking into an enemy to the exact offered
`EngageEncounter` command. A CLI, baseline AI or MCP client submits the same
command directly. Presentation timing cannot change which options exist or the
resulting hash.

### Encounter preparation

The pre-battle phase is also decision-based. It may offer normal engagement or
validated techniques from the locked participant builds. Technique Point cost,
non-attacking technique accumulation, attacking-technique engagement and
initial battle contributions are compiled into declared operations. The caller
cannot invent a technique effect or bypass its cost. An authored enemy-initiative
or preemptive policy is data, not a simulated collision outcome.

Destructibles and spatial bonuses are represented only when they have a frozen
mechanical row. They become bounded abstract interactables whose result is an
Activity program. Decorative objects and traversal are omitted.

## State and scope ownership

Standard Universe authoring aliases map to generic scopes:

| Authoring concept | Runtime scope |
|---|---|
| complete run | `Activity` |
| world stage/plane | `Section` |
| domain and its hub | `Node` |
| domain retry/entry instance | `Attempt` |
| combat | `Battle` and combat-owned shorter scopes |

Activity-scope state includes selected Path, Blessing inventory and enhanced
state, Curios and lifecycle state, Cosmic Fragments, Resonance Formations,
Ability Tree contributions, participant locks and run metrics. Section and Node
slots contain only values whose lifetime is actually shorter.

State uses typed definitions and bounded collections. No generic
`HashMap<String, Value>`, workbook column name or source-project ID is an
authoritative identity.

## Graph generation and progression

World, difficulty, topology, room and domain rows compile into an immutable
validated graph. Graph RNG uses a labeled stream independent from rewards,
occurrences, shops, encounter spawning and battles. Stable IDs order every
candidate set before weighted selection.

Graph validation requires:

- one valid entry and at least one reachable terminal;
- bounded visits for every loop and hub;
- valid section/domain ownership for every edge;
- no exit whose mandatory clear condition is impossible;
- no decision point with zero legal choices unless it is a typed terminal or
  fault;
- deterministic resolution of every pool and authored fallback.

For the frozen 4.4 Standard snapshot, the 37 released base topology templates
have no retained World-specific distribution or weights. Starclock therefore
uses the explicit project policy `StableUniformOrderedCandidates` on the Graph
RNG stream, with templates ordered by stable topology ID. This is a declared
approximation and must be replaced by a new policy revision if authoritative
weights become available.

Each source topology node compiles to a spatial-free domain micrograph with
room-resolution, content, encounter-member, battle, reward and route nodes. A room is
eligible when its section list is empty, contains `0`, or contains the source
node's section index. Room selection is stable-uniform because the released
snapshot contains no authoritative room weights. The room's exact primary
condition resolves its content; encounter groups then select one authored
member using exact weights and stable option order. Member waves remain an
ordered sequence rather than simulated monster objects.

The selected encounter becomes a preparation decision backed by a
roster-specific immutable overlay. That overlay contains only validated
`EncounterPreparationDefinition`, `BattleSpec` and result-contract values; its
digest is included in the compiled Activity identity. A Won result returns to a
reward node, and claiming that reward records the bounded hub-clear counter
before route handles become legal. Fixed content and external-decision rooms
use the same content/reward seam and are specialized by later profile batches.
Mandatory, optional, sequential and one-of encounter-slot policies are explicit
domain types. Coordinates, collision and traversal timing never enter this
contract.

The Blessing reward seam is concrete. One Activity-scope inventory stores
`BlessingId -> level`, where stack `1` is the base level and stack `2` is the
enhanced level. A reward node filters already-owned entries plus explicit rarity
and prerequisite eligibility, then samples at most three options without
replacement from the independent Reward stream. The released profile is
compiled as fully unlocked; callers that model earlier progression use the
explicit prerequisite-token set. One reroll per reward node is recorded in a
private bounded counter map. Stale and exhausted rerolls change neither state
nor RNG.

Acquisition, enhancement and replacement are ordinary checked Activity
programs. Replacement removes the complete old level before adding the new
base level in the same transaction. Each owned entry projects to an immutable
typed contribution carrying Path, rarity, mechanic tags, the selected level's
rule key, source binding and exact decimal parameters. P4-M02–M10 compile those
typed values into executable Path-specific combat rules; until those partitions
land, no document or coverage report may claim that all Blessing effects run in
battle. Ordinary reward candidates currently use the explicit stable-uniform
policy because exact released rarity/Path-biased probabilities are not proven;
P4-M01 owns replacement of that policy when evidence is available.

The activity terminates with a typed completion, defeat, abandonment or fault.
Account rewards, weekly points, achievements and inventory payout remain
outside the result.

## Universe mechanics

The Standard profile must execute, rather than merely load:

- Path selection, Path passive, Resonance energy/action and selected
  Resonance Formations;
- Blessing offers, prerequisites, rarity, replacement, enhancement and exact
  battle/run contributions;
- Curio acquisition, uniqueness, charges, negative/error states, repair,
  replacement and teardown;
- Occurrence conditions, choices, costs and outcomes, including explicitly
  versioned `ProjectPolicy` random choices;
- Cosmic Fragments, rerolls, shops, Blessing enhancement, respite, revival and
  downloader services;
- battle-affecting Ability Tree prerequisite/effect input;
- all frozen encounter groups, waves, enemy bindings and difficulty policies.

Shared behavior uses typed Activity operations or battle Rule IR. Static native
handlers are permitted only for mechanics that cannot reasonably be expressed
by those IRs. They return ordinary validated operations and may not mutate
state, draw untracked RNG, branch on scattered content IDs, or launch battles.

## Battle handoff and persistence

A pending encounter compiles an immutable `BattleSpec` containing:

- locked participant/build digests and current permitted carry state;
- exact encounter waves, enemy variants and difficulty bindings;
- a battle-scoped `RuleBundle` compiled from Path, Blessings, Curios, Ability
  Tree, environment and encounter sources;
- initial technique contributions and battle-visible resources;
- a purpose-derived battle seed;
- a declared `BattleResultProjection`.

Only the declared projection may return outcome, surviving HP/Energy/presence,
mode counters, metrics and verification hashes. Combat never grants Blessings,
Curios or Cosmic Fragments and never mutates the Activity directly.

Carry policies are explicit per field. Defeat, revival and downloader changes
are applied at the activity boundary before the next `BattleSpec`. Rejected
identity, configuration, seed, projection or final-hash values leave the
Activity byte-identical.

## Decisions, AI and external control

The deterministic baseline activity controller receives only the ordered legal
commands. It scores path alignment, modifier synergy, guaranteed resource
delta, survival, risk and graph progress with integer/fixed-point scores and
stable-ID tie breaking. It is a reproducible completion policy, not an optimal
solver.

Agent and MCP adapters expose owned, versioned Activity observations and opaque
tokens for exact offered commands. They settle automatic activity steps and
enemy battle decisions until the next external player decision. Universe tools
extend the existing authority, idempotency, tenant and replay boundaries; they
do not accept uploaded rules, arbitrary outcomes or raw resource edits.

Adventure/action domains use `ExternalOutcome`. Only a currently offered,
bounded outcome ID may be submitted. Physics, timing and aiming are not
simulated.

## Replay, RNG and hashing

The activity replay records every accepted Activity command, automatic
controller selection, nested battle start/result identity, RNG revision and
post-command hash. It verifies the exact combat and universe catalog digests.

Independent labeled streams cover at least graph, encounter, reward, shop,
occurrence/policy, external-outcome test and each battle. Draws in one stream
must not perturb another. Canonical activity encoding uses fixed field order,
bounded lengths and stable-ID ordering; normal Serde output is never hashed.

## Performance shape

Validated catalogs are immutable and shared by `Arc`. Each run owns only its
mutable Activity/Battle state, RNG counters and reusable transaction scratch.
An accepted command must not clone the complete catalogs or replay prefix.
Server verification supports incremental session execution and linear one-shot
replay; it must not rebuild all prefixes into quadratic work.

Goal 04 freezes release workloads after the first vertical slice, including a
complete run, trigger-heavy battle handoffs, many concurrent runs sharing one
catalog, invalid-command rejection and replay verification. Allocation,
semantic-copy and throughput regressions become policy-gated before release.

## Explicit exclusions

- all 3D scenes, movement, collision, aggro radius, patrol and rendering;
- Swarm Disaster, Gold and Gears, Unknowable Domain, Divergent Universe and
  historical/temporary variants;
- story presentation, dialogue playback, assets and action-minigame physics;
- account progression, weekly rewards, achievements, inventory and networked
  save synchronization;
- arbitrary scripting or a universe-only battle/runtime/replay protocol.

## Acceptance summary

- all 2,201 Goal 03 DataReady records and 786 rule bindings have an explicit
  runtime disposition;
- all 78 mechanic-family fixtures execute against runtime domain values;
- all nine Worlds, nine Paths and 33 difficulties construct valid profiles;
- frozen seeded runs complete deterministically with nested battle verification;
- invalid commands/results preserve hashes and RNG isolation perturbations do
  not change unrelated streams;
- CLI, in-process agent and MCP clients select only offered commands;
- Windows, Linux and macOS execute identical golden commands, events and hashes;
- the combat core contains no universe content-ID branches or dependency on the
  universe mode crate.
