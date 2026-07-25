# Goal 07 Ability Tree Partition S01

`G07-P2-M01-S01` completes the first 16 frozen Ability Tree records, their 16
mechanic-rule bindings and six operation fixtures:

- records `universe.ability-tree.1`, `2`, and `10` through `23`;
- operations `AddChoice`, `AddCurrency`, `AddLimit`, `AddStat`, `Enable`, and
  `Unlock`;
- no native handler, enemy variant, or encounter-member admission.

## Authoritative data

`Universe.xlsx` owns the node, prerequisite, cost, effect and parameter rows.
`UniverseBindings.xlsx` owns each `AbilityTreeContribution` rule.
`UniverseEvidence.xlsx` owns the content audit, source provenance and semantic
fixtures. `tools/goal07/author-ability-tree-partition.py` loads those workbooks
with openpyxl, rejects formulas/errors or missing cross-references, compares
the assigned identities with committed Sora debug output, and freezes a
partition semantic golden.

The production workbook bootstrap remains the only writer of complete
Universe workbooks. The focused partition checker does not patch an existing
workbook or treat JSON as the authoring source.

## Runtime closure

All assigned effects execute through `AbilityRuntimeCatalog` at RunStart,
BattleStart, elite/boss-entry or AfterBattle according to their authored
scope/condition. Post-battle run values are committed through a state-only
boundary program before the reward decision is generated.

Two previously retained behaviors are now production-owned:

1. Ability Tree node 11 enables exactly one deterministic reroll for each
   Blessing offer. Without the node, the command fails as `RerollDisabled`
   without changing state.
2. Ability Tree node 21 grants one extra Blessing selection after the first
   won battle. A generic conditional Activity operation consumes the bonus,
   leaves the run at the same Reward node once, then performs the ordinary
   reward settlement and traversal.

The conditional primitive is final in its enclosing program, validates both
branches recursively and executes inside the ordinary Activity transaction.
The implementation therefore does not add content-specific branching to
`starclock-activity`, does not increase the 579-hub graph node count, and does
not expose Universe data types to the battle core.

## Verification

The focused tests prove exact assigned-row/rule coverage, all six operation
fixtures, post-battle delta projection, reroll gating, two-stage first-battle
reward settlement, rejected-result byte identity and unchanged historical
empty-tree replay hashes. The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M01-S01.json`.
