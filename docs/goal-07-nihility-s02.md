# Goal 07 Nihility Partition S02

`G07-P2-M04-S02` completes sixteen content records, sixteen mechanic-rule
records and eleven native-handler reviews. It executes the second assigned
Nihility group, including Weakness Break Efficiency, Break propagation,
random DoT application, selected DoT detonation and the first Ignosticism
level. No native handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns the exact Blessing levels and parameters;
- `UniverseBindings.xlsx` owns the sixteen mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance and all native reviews.

The assigned rows were already present in the authoritative workbooks.
`tools/goal07/author-path-partition.py` verifies them with openpyxl, rejects
formula/error cells and compares them with the committed Sora 0.3.0 production
and debug exports. Runtime materialization reads only the validated `.sora`
bundle.

## Executed Blessings

### Call of the Wilderness levels (`612242`)

The two level rows assigned to this partition retain the exact deterministic
six-decimal conversion established by S01:

```text
L1: -0.3% enemy ATK per Suspicion stack
L2: -0.4% enemy ATK and Effect RES per Suspicion stack
cap: -30% for each affected stat
```

### Night Beyond Pyre (`612243`)

All allies gain Weakness Break Efficiency through the ordinary staged
modifier pipeline:

```text
L1: +30%
L2: +45%
```

The production Kafka Ultimate fixture observes an exact `13 / 10` attempted
Toughness-reduction ratio at L1. The modifier is queried from the acting unit
for each ordinary reduction and is not applied a second time to authored
forced Break.

### Hell is Other People (`612244`)

When an ally causes Weakness Break, the same element force-breaks:

```text
L1: adjacent living enemies only
L2: all other living enemies
```

Seven fixed-element triggers preserve the observed Break element without
content-ID branching. Cause ancestry recovers that element at the
`WeaknessBroken` boundary, and source exclusion prevents propagation from
recursing. The adjacent selector excludes the original target, so it does not
emit a synthetic zero-reduction event.

### Twilight of Existence (`612245`)

After a broken enemy is attacked, one of the following DoTs is selected with
equal probability in stable order:

```text
Bleed, Burn, Wind Shear, Shock
```

The selected effect then uses a separate labeled 75% resistible application
draw and lasts two target turns. Burn, Wind Shear and Shock snapshot 15% of
the applier's ATK. Bleed snapshots:

```text
min(6% of target Max HP, 2 * applier level Break base damage)
```

The level-derived Break base uses the exact public level 1–80 lookup table.
Enhanced Twilight removes the attacker's newest removable negative effect
after the application attempt.

### All Things are Possible (`612246`)

After an enemy is attacked, exactly one currently attached DoT is chosen in
stable instance order and detonated:

```text
L1: 100% of current DoT damage
L2: 150% of current DoT damage
```

Zero candidates consume no draw; one candidate is selected without a draw;
multiple candidates use the labeled `behavior-choice` stream.

### Ignosticism level 1 (`612250`)

Each selected Nihility Blessing increases all allies' DoT damage by 6%, up to
six counted Blessings:

```text
DoT increase = 6% * min(selected Nihility Blessings, 6)
maximum = 36%
```

Level 2 belongs to the following partition and is not claimed here.

## Generic core additions

This partition adds no Nihility or content-ID branch to combat core. Shared
domain behavior now includes:

- deterministic random selection among sorted effect definitions followed by
  an independent effect-chance draw;
- deterministic selection of one current DoT for detonation;
- oldest-first and newest-first bounded effect removal;
- modifier-aware Weakness Break Efficiency on authoritative reductions;
- an adjacent-only selector anchored to the observed primary target;
- exact level 1–80 Break-base lookup exposed as a generic derived stat;
- bounded cause-ancestry recovery of a Break element.

Numeric and RNG implementations remain private. The new Rule IR operations
lower into the same transaction journal, so rejected or faulted commands keep
state and draw counters atomic.

## Production verification

The production Kafka form executes its released Ultimate through the normal
legal-command boundary against a two-enemy encounter. The fixture proves:

- all sixteen assigned records and rules materialize from formal Excel/Sora
  data;
- Night Beyond Pyre changes a real Toughness attempt by exactly 30%;
- Hell propagates the triggering Lightning Break to only the adjacent
  Fire-weak enemy;
- Twilight performs ordered random selection, resistible application and the
  exact Bleed cap;
- All Things detonates exactly one current DoT;
- enhanced Twilight uses newest-first negative-effect removal;
- all eleven exceptional candidates close as `IrSufficient`.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M04-S02.json`.
