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
room-resolution, content, encounter-member, battle, reward, Formation-gate and
route nodes. A room is eligible when its section list is empty, contains `0`,
or contains the source
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
instead offer an `ExternalOutcome` and then cross the Formation gate directly.
They never receive an implicit battle Blessing reward. The exact service,
occurrence or fixed-content effect behind that outcome is specialized by
P4-M13–M15.
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

The selected Path contributes its stable Path identity, buff classification and
unlock policy as a typed passive source. A bounded counter map tracks unique
owned Blessings per Path. Three selected-Path Blessings expose the active
Resonance contribution; six, ten and fourteen expose one Formation choice each.
Every reward crosses a generic Formation-gate node. When no threshold is due it
offers one deterministic continue command; when due it offers only the unowned
Formations for the selected Path. The accepted choice writes a one-stack entry
to the Activity-owned 27-entry Formation inventory.

The active Resonance contribution retains its rule key, released stage-ability
binding, mechanic tags, exact parameter vector, initial energy and maximum
energy. Its engine-independent resource state uses checked signed 64-bit values
at six decimal places, clamps gains to the authored maximum, becomes actionable
only at the maximum and consumes exactly that maximum on activation. P4-M02–M10
compile the nine Path-specific effects, and P4-M01 attaches the resulting
generic rule/action contributions to each immutable `BattleSpec`; until then,
availability and contribution compilation must not be described as effect
execution.

Curios use a separate one-stack Activity inventory, so ownership is unique and
never inferred from an effect list. Two player-visible bounded counter maps,
keyed by `CurioId`, hold the current `CurioStateId` and remaining charge count.
Acquisition writes ownership, initial state and initial charges in one checked
transaction. Charge consumption uses an expected remaining value; the last
charge performs the authored state transition in that same transaction.
Repair, replacement and teardown likewise update ownership, state and charge
together. A removed Curio may leave canonical zero-valued counter tombstones,
but contributes no rule and cannot be observed as owned.

Each owned Curio projects only its current state as an immutable contribution:
Curio/state identity, positive or negative pool tags, Curio and state rule keys,
source effect, exact parameter vector, and current/maximum charges. Orphaned,
duplicated, cross-Curio or out-of-range state is rejected. The six released
Error Codes begin in a three-charge Repairing state and transition to their
Fixed state; the frozen source currently defines no cross-Curio replacement
edge, while the generic atomic replacement operation remains available for
later occurrence/service rules. P4-M11/M12 execute individual Curio effects and
P4-M01 materializes the typed contributions into each `BattleSpec`.

Cosmic Fragments are a player-visible Activity-scoped bounded integer in the
inclusive range `0..=4_294_967_295`. Generic credit and spend programs use
checked Activity operations; a failed affordability requirement rolls back the
entire command. Combat cannot write this slot directly.

The run runtime compiles all 321 Occurrence choices into typed definitions that
preserve variant identity, stable order, conditions, next-node keys, exact
parameter vectors, costs and outcomes. It similarly compiles all 94 services
with kind, currency, price formula, offer pool, rule key and parameters. These
definitions are immutable catalog input, not executable strings. P4-M13 owns
the per-choice Occurrence operation lowering and the versioned policy for the
52 outcomes whose released weights are unknown. P4-M14 owns service-specific
shop, enhancement, respite, revival, downloader and roster operations.

Selected Ability Tree nodes compile to a canonical typed contribution set and a
closed M01 execution catalog. Its 22 targets and seven condition forms are Rust
domain enums; all ten operations use checked signed six-place arithmetic and
retain the source node. A caller requests an immutable projection for RunStart,
BattleStart, elite/boss-domain entry or post-battle boundaries. Node 17's
initial Cosmic Fragments are written into the Activity slot during profile
compilation; later service and reward partitions consume the remaining run
targets through the same projection. Battle-visible values are generic
stat/resource/unlock contributions, so Ability Tree or service vocabulary never
enters the combat resolver. Path, Blessing and Curio rules join that boundary in
their owning partitions before the complete battle integration gate.

Non-spatial interactions are identified by public opaque external-outcome IDs.
Submission succeeds only when both the decision and outcome are currently
offered; stale and unoffered submissions preserve state and RNG. An accepted
outcome records a bounded consumed marker and advances through the ordinary
graph transaction. Physics, timing and navigation never authorize an outcome.

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
