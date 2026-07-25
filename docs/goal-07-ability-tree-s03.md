# Goal 07 Ability Tree Partition S03

`G07-P2-M01-S03` completes the final ten frozen Ability Tree records, their ten
mechanic-rule bindings, and the `Set` and `UnlockFormationSlot` operation
fixtures:

- records `universe.ability-tree.4` through `9` and `39` through `42`;
- party ATK, DEF, maximum HP and Effect Hit Rate;
- Path Resonance damage and Formation-slot capabilities;
- Reviver authorization with a 100% restored-HP ratio;
- the run-level consumable-use authorization boundary;
- no native handler, enemy variant, or encounter-member admission.

## Authoritative data

`Universe.xlsx` owns nodes, prerequisites, costs, effects and parameters.
`UniverseBindings.xlsx` owns each `AbilityTreeContribution` rule.
`UniverseEvidence.xlsx` owns the content audit, source provenance and operation
fixtures. `tools/goal07/author-ability-tree-partition.py` verifies the assigned
rows with openpyxl, rejects formulas, spreadsheet errors and unresolved
references, compares the workbooks with committed Sora debug output, and
freezes the partition semantic golden.

Production authoring remains `.xlsx` plus openpyxl. Sora 0.3.0 remains the
schema and transport boundary; runtime code does not load the staging JSON.

## Generic Formation capability

Ability Tree `UnlockFormationSlot` effects compile to a bounded 0–3 slot
capacity. Formation selection must pass two independent checks:

1. the chosen Path owns at least 6, 10 or 14 Blessings for Formation positions
   one, two and three;
2. the corresponding Ability Tree slot is unlocked.

The topology offer condition and `PathRuntimeCatalog` contribution validation
consume the same capacity. An empty Ability Tree authorizes zero Formations;
the complete tree authorizes three. This replaces the earlier third-slot-only
special case with one reusable capability model.

## Reviver and participant carry

Each Respite domain compiles one deterministic Reviver option per locked
participant. The option requires all of the following:

- the formal Ability Tree `service.reviver` capability;
- the target participant is defeated in the Activity carry ledger;
- sufficient Cosmic Fragments;
- the authored restored-HP ratio matches the private Ability Tree projection.

The service handler only validates and lowers the request. It returns ordinary
generic `Require(ParticipantDefeated)` and `RestoreParticipant` Activity
operations. The transaction restores HP with checked fixed-point floor
rounding, keeps carried Energy, changes life to `Alive`, changes presence to
`Present`, increments service usage and charges currency atomically. Missing,
alive or otherwise invalid targets reject without changing canonical state.

Topology-template reuse is participant-lock-aware because Reviver options
embed participant identities. A profile may reuse the cached template only
when the participant-lock digest matches; another roster receives a freshly
compiled topology.

## Consumable boundary

The Ability Tree exposes `run.consumable_use` through
`StandardUniverseRunCapabilities`. This is authorization, not an account
inventory implementation. An engine, service or future inventory adapter owns
available item instances and item effects, then submits validated commands
through the Activity boundary. Goal 07 does not invent unavailable consumable
catalog data or let account state mutate the run implicitly.

## Determinism and verification

Entry, topology, path and service-interaction revisions advance because their
canonical definitions or payloads changed. Focused tests prove:

- all ten assigned records and rules execute from the formal catalog;
- exact S03 stat, ratio and capability totals;
- 0–3 Formation authorization combined with Blessing thresholds;
- participant-specific topology compilation is safe across profile-cache
  reuse;
- Reviver lowering and generic carry restoration are transactional;
- consumable authorization is externally observable without coupling account
  inventory to the simulation.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M01-S03.json`.
