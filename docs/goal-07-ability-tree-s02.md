# Goal 07 Ability Tree Partition S02

`G07-P2-M01-S02` completes 16 frozen Ability Tree records, their 16
mechanic-rule bindings and the `AddResource` and `SetRatio` operation fixtures:

- records `universe.ability-tree.3` and `24` through `38`;
- 18 typed effects covering party stats, battle energy, Path Resonance energy
  and damage, and the third Resonance Formation slot;
- no native handler, enemy variant, or encounter-member admission.

## Authoritative data

`Universe.xlsx` owns the node, prerequisite, cost, effect and parameter rows.
`UniverseBindings.xlsx` owns each `AbilityTreeContribution` rule.
`UniverseEvidence.xlsx` owns the content audit, source provenance and the two
operation fixtures. `tools/goal07/author-ability-tree-partition.py` verifies the
assigned rows with openpyxl, rejects formulas/errors or unresolved references,
compares them with the committed Sora debug output and freezes the partition
semantic golden.

The production `.xlsx` workbooks remain the authoring source and
`config/universe-generated/config.sora` remains the runtime transport. No JSON
row is loaded as a production-side substitute.

## Runtime closure

The Ability Tree projection now reaches the actual battle and path runtimes:

1. Battle-start Energy ratios replace each player participant's initial Energy.
   Elite/Boss entry is a composite boundary, so ordinary battle-start effects
   and the elite/boss full-Energy effect execute together. Explicit Activity
   carry cannot overwrite the authored boundary refill.
2. Party damage-taken reduction lowers into the generic `Mitigation` formula
   stage for direct, DoT, additional, Elation, Break and Super Break damage.
3. Path Resonance initial Energy initializes the shared Resonance resource, and
   Path Resonance damage ratio multiplies the authored scaling coefficient
   using checked fixed-point arithmetic.
4. The third Resonance Formation remains locked at 14 chosen-Path Blessings
   unless the formal Ability Tree projection grants the capability. The
   topology gate and `PathContributionSet` validation enforce the same rule.

Encounter-option bindings retain room and domain kind, allowing battle
snapshot compilation to select the elite/boss boundary without any 3D scene
state. All new behavior remains in mode compilation/materialization; the
engine-independent combat core receives ordinary modifiers, resources,
abilities and participant initial state.

## Determinism and compatibility

The changed semantic contracts advance Ability Runtime, Path Runtime, Battle
Contribution, Battle Snapshot, Battle Materialization, Topology and Entry
revisions. Canonical state, replay and materialization goldens were regenerated
from the new revisions. Rejected commands and invalid snapshots retain their
existing transactional behavior.

## Verification

The focused test proves exact assigned-row/rule coverage, both operation
fixtures, composite elite/boss projection, materialized full Energy, six
damage-mitigation purposes, increased Path Resonance scaling and capability-
gated third Formation selection. The full `starclock-mode-universe` suite,
production workbook verification and the openpyxl/Sora partition check pass.
The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M01-S02.json`.
