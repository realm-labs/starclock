# Combat Identity and Dynamic Assembly

This document is normative for Goal 06. It defines the identity split between
one battle's executable input and the outer build/Activity/mode assembly, plus
the Standard Universe boundary that constructs a fresh battle request from the
current authoritative Activity snapshot.

## Problem statement

The Goal 05 runtime executes real battles, but two boundaries remain too weak:

1. `BattleSpecDigest` is supplied by the caller even though it is described as
   the exact battle request identity. `starclock-combat` validates references
   but does not independently prove that the digest covers every visible input.
2. `StandardUniverseRuntimeFactory` composes one battle materialization from
   the entry-time empty inventory. Later Activity acquisitions are
   authoritative, but the selected contributions are not projected into later
   `BattleSpec` values.

The first issue weakens server/replay identity. The second prevents otherwise
executable acquired mechanics from affecting later battles. Neither requires
Universe state inside the combat resolver.

## Identity domains

### Combat input identity

`CombatInputDigest` is computed inside `starclock-combat` from a versioned
canonical encoding of every `BattleSpec` value observable by battle
construction or resolution:

- rules compatibility revision;
- encounter definition;
- canonical participants, source bindings and initial carry;
- resolved combatant definition, abilities, rules, modifiers, resources and
  combatant input identity;
- player/enemy team resources and keyed resources;
- concession and other battle-local policies.

The encoder uses fixed field order, explicit enum tags, fixed-width integers,
length-prefixed text/collections and the project numeric encoding. It does not
hash normal serialization, pointer identity, cache state or outer definitions.
The constructor computes the digest after canonicalization; no public
parameter or setter can override it.

Changing the encoded field set or byte layout creates a new combat-input codec
revision. Tests must mutate each field family independently and prove a digest
change.

### Assembly identity

`AssemblyDigest` is opaque to `starclock-combat`. It is computed by the owner
that combines:

- resolved build/loadout and participant-lock identities;
- current Activity state relevant to the pending battle;
- selected mode profile and content partitions;
- Path, Blessing level, Resonance/Formation, Curio state and Ability Tree
  contribution identities;
- encounter/difficulty/materialization policy;
- approved numeric approximation policies;
- composed handler/rule registry and consumed-component root.

Assembly identity is provenance and compatibility, not authority over combat
input. Equal battle-visible values can intentionally have different
`AssemblyDigest` values. Unequal battle-visible values cannot share a
`CombatInputDigest`.

### State and result identity

Battle canonical state binds both identities:

```text
BattleIdentity
  catalog revision + digest
  combat-input codec revision + CombatInputDigest
  AssemblyDigest
  numeric/RNG/state-hash revisions
  BattleSeed
```

An Activity pending task and `BattleResult` contract bind the same pair.
Result settlement compares exact offered identities before any Activity
mutation. Final state/event commitments do not substitute for input identity.

## Dynamic assembly boundary

Definitions and selected instances have different lifetimes:

```text
Released Sora/components
      -> immutable CombatCatalog and mode definition catalogs
      -> shared across sessions and battles

Current Activity state at pending-battle boundary
      -> immutable contribution snapshot
      -> selected definitions, bindings, values and carry
      -> BattleSpec + result projection
      -> one isolated Battle
```

Catalog composition happens once per exact released component set. It may
contain definitions for mechanics that are not currently owned. Per-battle
assembly selects only the current authoritative contributions and cannot add
unvalidated definitions.

The contribution snapshot includes:

- selected Path;
- exact owned Blessing levels;
- unlocked/selected Resonance and Formations;
- owned Curios with current active/disabled/repaired/replaced state and
  charges where battle-visible;
- selected Ability Tree battle values;
- current participant HP, Energy, life/presence and declared carry;
- current encounter member/difficulty and preparation/technique selection.

Acquiring or changing a contribution after a battle starts never mutates that
battle. It affects the next eligible assembly boundary.

## Atomic preparation

Assembly is part of pending-task preparation, before the no-draw
`BattleStarted` marker commits:

1. read one immutable Activity snapshot and expected state hash;
2. derive the canonical `BattleAssemblyKey`;
3. resolve or compute the immutable assembly;
4. validate all selected definitions, participant locks and result projection;
5. commit the pending handoff identity;
6. start the isolated battle only after the handoff is sealed.

Unknown definitions, stale snapshots, invalid carry, cache corruption, budget
failure or result-contract construction failure return typed errors and leave
Activity canonical bytes, RNG counters, pending decision and replay records
unchanged.

## Cache contract

The assembly cache is an optional bounded optimization:

- the key contains every component/snapshot/encounter/roster input consumed by
  assembly;
- entries are immutable;
- lookup and eviction order are deterministic where observable;
- capacity is bounded and documented;
- cache contents, hit counts and allocation layout are not canonical state;
- disabling, clearing or evicting the cache produces identical handoff,
  commands, events and hashes;
- a cached entry is revalidated against its key/digest before use.

The immutable combat catalog may be shared by concurrent sessions. Mutable
session/Activity state and cache scratch cannot be shared without isolation.

## Replay v3

New production Activity recordings use component-addressed replay v3. Every
nested battle records:

- ordered consumed component root;
- `AssemblyDigest`;
- combat-input codec revision and `CombatInputDigest`;
- exact handoff/result identities;
- accepted controller/command sequence;
- emitted event payloads and state hashes.

Verification reconstructs the Activity and each per-battle assembly from
released components and the recorded command stream. It compares components,
assembly, combat input, command, event, state and result in that order and
reports the first divergence.

Released replay v2 remains decodable and verifiable through its historical
path. V2 is not emitted by new production entry points and its frozen evidence
is never regenerated.

## Public and dependency boundaries

- `starclock-combat` defines `CombatInputDigest`, opaque `AssemblyDigest`,
  `BattleSpec`, `Battle`, commands, events and views.
- `starclock-activity` binds assembly/input identities to pending tasks and
  result settlement.
- `starclock-mode-universe` projects Activity state and selects mode
  contributions.
- `starclock-replay` owns v2/v3 transport and verification codecs.
- CLI, Agent and MCP call one production factory/assembler; none reconstructs
  battle inputs independently.

`starclock-combat` cannot depend on Activity, mode, build, data, replay, AI,
serialization or transport crates. No Path/Blessing/Curio ID branch may enter
shared battle code.

## Required proof

- field-by-field combat-input digest sensitivity and construction-order
  invariance;
- outer-provenance-only assembly changes leave combat input unchanged;
- acquire/upgrade/disable/remove contribution sequences change the next
  assembly exactly;
- an already-running battle remains isolated from later Activity changes;
- cache enabled/disabled/evicted results are identical;
- stale/invalid assembly and executor failure roll back exactly;
- CLI, baseline, Agent, MCP and replay reconstruction produce identical
  assembly/input identities for equal inputs;
- replay-v3 corruption reports the first correct identity/payload boundary;
- replay v2 historical verification still passes;
- per-battle assembly does not rebuild the immutable catalog;
- native platform and service-throughput gates remain within their declared
  budgets.

