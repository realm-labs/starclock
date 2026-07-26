# Goal 07 Curio Partition S04

`G07-P3-M11-S04` executes eight positive Standard Simulated Universe Curios:

- Sealing Wax of Elation (`universe.curio.23`);
- Sealing Wax of The Hunt (`universe.curio.24`);
- Sealing Wax of Destruction (`universe.curio.25`);
- Sealing Wax of Remembrance (`universe.curio.26`);
- Sealing Wax of Nihility (`universe.curio.27`);
- Sealing Wax of Abundance (`universe.curio.28`);
- Warping Compound Eye (`universe.curio.3`);
- Fruit of the Alien Tree (`universe.curio.4`).

The partition owns 16 records and 16 rules. The definitions, states,
parameters, rules and provenance remain authored in the formal
`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx`
workbooks. The openpyxl partition command verifies those rows against the
committed Sora 0.3.0 bundle and emits the derived partition golden.

## Table-driven Sealing Wax policy

All nine Standard Universe Sealing Waxes now share one closed offer policy.
The policy table maps Path stable keys to Curio inventory content and derives
the eligible option subset from the compiled Blessing catalog. Acquisition
uses the authoritative Reward RNG stream and grants exactly one unowned
Blessing of the Wax's Path. Later postcombat offers increase the weight of
eligible Blessings from that Path.

Public evidence says the appearance rate is “greatly increased” but publishes
no numeric multiplier. Runtime v1 therefore freezes `x2` for every Wax as one
replaceable project-policy approximation. Missing Path options are skipped
rather than creating an invalid empty conditional policy. This is catalog
composition, not a content-specific resolver branch or native handler.

## Warping Compound Eye

When a postcombat offer contains a one-star Blessing, acquiring that option
immediately stores its enhanced level. The ordinary acquisition count is one;
Warping Compound Eye contributes its owned inventory count, producing exactly
two for one-star options and leaving two- and three-star options unchanged.
The expression is checked Activity arithmetic inside the same reward
transaction, so path counters, Dimension Reduction Dice settlement, formation
gates and state hashing remain atomic.

## Fruit of the Alien Tree

After a won battle, the runtime inspects the validated participant projection.
If one or more carried participants have zero HP and a non-alive life state,
the generic `RestoreParticipant` operation revives every such participant at
100% maximum HP. The same boundary transaction records the Curio event and
removes the one-charge Fruit, including its state and charge entries. If no
participant is downed, the effect does not trigger and the Fruit is retained.

This effect is owned by the Activity settlement adapter. Combat emits only
participant results and has no knowledge of Curio inventory or cross-battle
carry state.

## Revision and executable evidence

`standard-universe-entry-v10` and `standard-universe-topology-v10` identify the
expanded nine-Path offer table, enhanced one-star acquisition expression and
post-battle revival settlement. Tests execute all six Path-correct acquisition
boundaries through the production runtime facade, inspect the compiled
one-star/fixed acquisition expressions, and submit a won production handoff
with one defeated participant to prove full restoration and atomic Curio
destruction. All eight native-handler reviews close as `IrSufficient`.
