# Goal 07 modifier-stage and snapshot runtime

`G07-P1-B3` closes the shared modifier capability needed by Standard Universe
content. The implementation remains combat-owned and content-neutral:
production Excel rows lower to immutable definitions, while battles contain
only typed modifier instances and captured domain values.

## Executable stages

`StatResolver::query` evaluates base-stat stages in this order:

1. `BaseAdd`;
2. `PercentOfBase`;
3. `Flat`;
4. `FinalAdd`;
5. `FinalMultiply`.

Bounds are intersected and applied at their named stage. A bound authored on an
earlier contribution may therefore cap a later stage; it is not discarded or
applied once per instance.

`StatResolver::query_formula` evaluates a named `FormulaStage` independently of
base stats. Damage, healing and shield operations construct one immutable
formula-input view per operation and query it for every target. Ordinary
damage interprets additive stage contributions as follows:

- `Crit`, `DamageBoost`, `Defense`, `Resistance`, `Vulnerability` and
  `Broken` add to their corresponding factor;
- `Weaken` subtracts from the outgoing factor;
- `Mitigation` multiplies by `1 - contribution`.

The query API also accepts probability, action-order, Break and specialized
damage purposes. Their operation owners consume that API as those operations
are closed in later Phase 1 batches. Formula modifiers never masquerade as
base-stat modifiers.

## Stacking

Applicable instances are ordered by priority, source, sequence and instance
identity before grouping. The nine aggregation policies are executable.
`StrongestByComparator` evaluates the stacking group's authored comparator for
each candidate; it never infers strength from the absolute modifier value.
The non-selected candidates remain alive and become eligible when the winner
expires. Equal comparator results use the canonical instance order.

Every group applies one intersected floor/cap after aggregation. Invalid
comparator presence, group references, stage/cap combinations and filter order
fail catalog construction.

## Snapshots

Snapshots contain fixed-point domain values, never references to mutable unit
state.

- `Dynamic` reads current state.
- `OnApplication` captures before insertion.
- `OnActionStart`, `OnPhaseStart` and `OnHitStart` refresh immediately before
  the corresponding authored `Started` event.
- `SourceSnapshotTargetDynamic` captures source-side `QueryStat` leaves.
- `SourceDynamicTargetSnapshot` captures target-side leaves.
- `ExplicitFields` captures every explicitly referenced `QueryStat` leaf.
- `RecomputeOnStackChange` uses the application capture until the explicit
  stack mutation boundary supplied by the state-slot/effect runtime.

Initial participant modifiers use the same capture algorithm after the complete
battle state has been assembled. The instance being captured is excluded from
its own capture resolver. A missing required capture faults; it never falls
back to dynamic evaluation.

## Determinism and cache boundary

Formula inputs clone only the base-stat map and ordered active modifier view
once per operation. This view is an ephemeral query cache: it is excluded from
canonical state and hashes and may be discarded without changing events.
Every accepted mutation still passes through the battle transaction journal.
Runtime query cycles produce deterministic faults; catalog-visible invalid
definitions are rejected before a battle can be constructed.

Production evidence includes 1,556 modifier definitions, 151 typed filters and
268 formula-stage modifiers. The openpyxl-authored comparator probe is exported
through Sora and executed by the production data reader; it is evidence of the
formal authoring path and is not content coverage credit.
