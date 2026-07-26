# Goal 07 Destruction Partition S01

`G07-P2-M07-S01` completes seventeen assigned content records, sixteen
mechanic-rule records and the Destruction review fixture. It covers both
released levels of five Blessings, the Destruction path record and the base
catalog record for the next Blessing partition. All runtime behavior is
expressed with effects, modifiers, selectors and generic Rule IR; no native
handler is admitted.

## Authoritative authoring boundary

The openpyxl-authored `Universe.xlsx`, `UniverseBindings.xlsx` and
`UniverseEvidence.xlsx` workbooks remain the editable source. The focused
partition gate rejects formula/error cells and compares every assigned row
with the committed Sora 0.3.0 binary and debug exports.

Released structured values remain the numeric source of record. Public
mechanism descriptions were cross-checked against the
[Simulated Universe Paths reference](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Paths)
and the
[Star Rail Station Blessing catalog](https://starrailstation.com/en/simuniverse/current/blessings),
accessed 2026-07-26.

## Fighting Spirit model

Fighting Spirit is not represented as a Destruction-specific branch in the
resolver. Three ordinary effects keep its concerns separate:

- real Fighting Spirit stores stacks earned from being attacked or consuming
  HP;
- virtual Fighting Spirit stores the HP-derived minimum from Non-Inverse
  Antimatter Equation;
- a synchronized engine effect stores `max(real, virtual)` and exposes that
  value through the generic modifier `source_stack_slot`.

The engine effect applies the released per-stack ATK and DEF modifiers.
Synchronizing through effect lifecycle events keeps modifier evaluation
independent of content identities and prevents real and virtual stacks from
being added together incorrectly.

Non-Inverse Antimatter Equation (`612530`) updates virtual stacks at battle
entry and after HP changes:

```text
L1 below 50% HP: 16 virtual stacks
L2 below 50% HP: 20 virtual stacks
L2 additionally: +2 stacks per complete 10% HP below 50%
maximum before the cap upgrade: 35
per effective stack: ATK +3%, DEF +3%
```

Universal Heat Death Characteristic (`612531`) grants four real stacks once
per incoming action. Self-authored HP loss is observed separately so an
ability that consumes HP also grants the same four stacks without double
counting ordinary incoming damage. Four real stacks expire at owner turn end.
Its enhanced level grants one stack to adjacent allies.

## Damage distribution

Regression Inequality of Annihilation (`612532`) is implemented as two ordered
generic steps:

```text
living party size = N
damage retained by original target = incoming damage × (1 - reduction) / N
each other living ally receives the same retained amount as True Damage

L1 reduction = 0%
L2 reduction = 15%
```

The current living-party count is stored in the ordinary effect stack slot
and refreshed after defeat or presence changes. Distributed damage carries
the Blessing source and uses True Damage, so it neither re-enters the
distribution trigger nor receives mitigation a second time.

## Retaliation and HP consumption

Incremental Doomsday (`612540`) reacts once per incoming action while the
owner has effective Fighting Spirit:

```text
L1 retaliation = base ATK × 4% × stacks
L2 retaliation = (base ATK × 4% + missing HP × 2%) × stacks
retaliation cannot defeat the attacker
```

Catastrophic Resonance (`612541`) snapshots and consumes 10% of current HP at
the start of an Attack action, never reducing the owner below one HP. Each
distinct enemy hit by that action receives event-element Additional damage:

```text
L1 damage = consumed HP × 60%
L2 damage = consumed HP × (60% + 1% × effective stacks)
```

The base catalog row for Indicative Depth of Field (`612542`) is validated by
this partition because it closes the frozen row range. Its two executable
levels, per-stack mitigation and enhanced cap are assigned to
`G07-P2-M07-S02`; S01 does not claim or pre-implement them.

## Production verification

Production tests prove:

- all six selected level bindings materialize as native-handler-free Rule IR;
- the shared real, virtual and synchronized Fighting Spirit effects are
  present with the released S01 cap of 35;
- effective stacks drive percent-of-base ATK and DEF;
- 30% HP produces exactly 24 enhanced virtual stacks;
- an Attack consumes exactly 10% current HP in a real battle;
- distribution uses typed True Damage, retaliation inherits the event element
  and cannot defeat, and HP consumption uses the generic sustain operation;
- an enemy-first production action executes distribution without recursion or
  a battle fault.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M07-S01.json`.
