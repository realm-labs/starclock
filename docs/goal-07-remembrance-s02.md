# Goal 07 Remembrance Partition S02

`G07-P2-M03-S02` completes sixteen content records, sixteen mechanic-rule
records, two production semantic fixtures and eleven native-handler reviews.
It executes both released levels of Insensitivity, Sentimentality,
Indelibility, Shudder and Maverick, plus level 1 of Unspeakable Shame. No
native handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns each Blessing, level and exact parameter row;
- `UniverseBindings.xlsx` binds the level records to released stage-ability
  sources;
- `UniverseEvidence.xlsx` owns the Effect RES and Ultimate fixtures, review
  rows and provenance.

`tools/goal07/author-path-partition.py` rejects formulas and spreadsheet error
cells, proves exact partition ownership, and compares the assigned workbook
rows with committed Sora 0.3.0 debug and production exports. Runtime
materialization consumes only the validated `.sora` bundle.

## Exact Blessing mechanics

### Ultimate Experience: Insensitivity (`612142`)

After Dissociation is removed from an enemy, the same enemy receives one
resistible Freeze application for one target turn:

```text
L1: 50% base chance
L2: 75% base chance
```

The trigger observes the typed `EffectRemoved` event. It does not infer
Dissociation from copied text, source IDs or damage values.

### Ultimate Experience: Sentimentality (`612143`)

When allied Ice damage is applied, it deals non-critical Ice Additional
damage derived from the applied damage of the observed event:

```text
L1: 20% to enemies adjacent to the original target
L2: 24% to every other enemy
```

The original target is excluded. The generated event preserves an explicit
source definition, and a generic event-source comparison prevents the
Blessing from recursively triggering itself.

### Ultimate Experience: Indelibility (`612144`)

Every allied damage application independently attempts to Freeze its target
for one target turn:

```text
L1: 2.0% base chance
L2: 2.5% base chance; enemy Freeze RES -20%
```

The enhanced reduction is a signed target-specific resistance modifier. It
uses the same checked effect-chance formula as every other resistible effect.

### Ultimate Experience: Shudder (`612145`)

After an ally resolves an Ultimate, one labeled deterministic draw selects an
enemy and applies Ice Weakness with a 70% base chance. The Weakness lasts for
two turns of the selected enemy:

```text
L1: choose one random enemy
L2: choose one random enemy that does not already have Ice Weakness
```

An empty enhanced candidate pool is a no-op. Temporary Weakness ownership,
duration and removal are generic Toughness state; the second target turn emits
one typed `WeaknessRemoved` event and restores the canonical weakness set.

### Ultimate Experience: Maverick (`612146`)

At battle start, all enemies receive a 150% base-chance Freeze application for
one target turn. L2 also binds `-30%` SPD, at the percent-of-base formula
stage, to the Freeze effect instance. Removing or expiring that instance also
removes its SPD modifier.

### Experience: Unspeakable Shame (`612150`, level 1)

Each owned Remembrance Blessing reduces every enemy's Freeze RES by 6%, up to
six counted Blessings:

```text
freeze_res_reduction = -0.06 * min(owned_remembrance_blessings, 6)
```

The validated contribution snapshot supplies the count. The battle resolver
does not inspect run inventory or a Blessing ID.

## Generic core additions

This partition adds no Standard Universe source-ID branch. Shared combat
capabilities now include:

- target-turn lifetimes for Rule IR `AddWeakness`;
- canonical temporary-Weakness refresh, expiry and replay state;
- `LacksWeakness(element)` as a typed selector predicate;
- correct `PrimaryPlusAdjacent` selection from the complete formation pool;
- signed general and target-specific resistance inputs in `[-1, 1]`;
- event-source identity comparisons through existing typed event properties;
- explicit selector dependencies for trigger filters and nested programs.

The numeric backend, temporary-Weakness store and generated Sora records
remain private. Public consumers continue to use domain commands, events and
battle views.

## Production verification

The production integration fixtures prove:

- all six assigned rule sources materialize and execute;
- Maverick freezes every enemy and binds its enhanced SPD modifier;
- Indelibility and Unspeakable Shame bind target-specific resistance
  reductions;
- enhanced Sentimentality emits exactly one 24% event for every other enemy
  and does not recurse;
- enhanced Shudder never selects an enemy that already has Ice Weakness;
- Shudder's Weakness expires on exactly the second selected-enemy turn;
- a `-56%` specific-resistance vector produces a `1.56` pre-clamp chance
  multiplier.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M03-S02.json`.
