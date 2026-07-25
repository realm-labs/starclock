# Goal 07 effect, state and lifecycle runtime

`G07-P1-B4` closes the shared stateful-mechanic boundary used by Standard
Universe content. Effects, state slots, shields and resources remain generic
combat-domain values; no Path, Blessing, Curio or character ID is interpreted
inside the resolver.

## State ownership and reset

Rule instances own typed state slots. Every slot has one value kind, initial
value, optional inclusive bounds, logical scope and an ordered set of reset
points. All nine reset points are authoritative:

1. `BattleStart`;
2. `WaveStart`;
3. `TurnStart`;
4. `ActionStart`;
5. `HitStart`;
6. `ActionEnd`;
7. `TurnEnd`;
8. `WaveEnd`;
9. `BattleEnd`.

The runtime resets only instances belonging to the supplied owner at
owner-scoped boundaries. A reset is journaled and participates in the
canonical state hash. Bounds are checked before mutation; an invalid update
faults and rolls back the command.

Charges are not a second subsystem. Character charges, once counters and
small mechanic meters use bounded state slots or named character/team
resources according to ownership and persistence. This keeps acquisition,
spending, overflow and replay encoding on the common transaction path.

## Effect application and stacking

An effect instance records definition, original source/applier, target,
duration clock, tick phase, stack policy, comparison fields, teardown policy
and attached rules/modifiers. The eight stack policies retain distinct
identity behavior. Replacement/removal always tears down attached modifiers
and rule instances in the same transaction.

`RefreshAndAddStacks` updates the stored stack count before later operations in
the same hit. A modifier may bind one private slot to its source effect's
stack count. The slot is initialized on attachment and refreshed on stack
change; `RecomputeOnStackChange` captures the new value before a following DoT
detonation or formula query. Catalog validation requires such a modifier to
belong to exactly one effect and prevents it from entering a participant's
static build.

Expression-backed duration and base magnitude are materialized per application
target. Inputs that must remain live after application are represented by
attached `Dynamic` modifiers/rules; partial and boundary snapshots use the
modifier snapshot contract. The effect snapshot policy remains in state and
replay evidence so content can declare and test the intended split explicitly.

## DoT and tick order

`DotDefinition` always forces `DamageClass::Dot`; callers cannot accidentally
route it through ordinary-damage modifiers. Natural ticks and detonations:

- multiply the authored per-stack base by the current effect stacks;
- query the DoT formula stages through the current modifier view;
- preserve the original effect applier and source definition;
- apply an authored detonation fraction only after the complete DoT formula;
- floor once at the damage boundary and emit the source effect identity.

At a target turn boundary the DoT resolves before its matching duration clock
advances. `ActionStart`, `ActionEnd`, `TurnStart` and `TurnEnd` ticks execute in
the current transaction. `WaveEnd` and `BattleEnd` duration clocks execute
after the corresponding ended event and before terminal reset/carry.
`AfterEvent` has no implicit “all events” meaning: catalog construction
requires an attached Rule IR trigger, which supplies the exact event/phase and
prevents recursive unbounded ticking.

## Owner teardown and carry

`RemoveWithOwner` removes effects applied by a defeated or departed owner,
including every attached rule/modifier. The remaining policies deliberately
retain the instance and its original attribution:

- `TransferToTeam` survives the unit boundary. Its enclosing contribution or
  team owner remains outside the unit effect, while combat retains the
  original applier attribution;
- `FreezeSnapshot` survives with captured values;
- `PersistByScope` is removed by its duration/activity scope;
- `ExplicitRule` is removed only by an authored operation.

They do not silently retarget the applier to another living unit. Wave carry
may additionally clear or reset player effects according to the encounter
policy.

Shield capacity remains in the shield store because simultaneous shields have
their own non-stacking absorption and decay rule. An effect may own the
lifecycle/modifier side of a shield, but incoming damage never treats a
generic scalar state slot as shield HP.

## Formal authoring evidence

Production contains four effect definitions, three state slots, two reset rows,
one effect/modifier binding and 46 named character-resource definitions. The
Goal 07 reset probe is authored in `StateSlotReset.xlsx` with Python
`openpyxl`, exported by pinned Sora 0.3.0 and asserted after domain lowering.
JSON remains diagnostic output only.
