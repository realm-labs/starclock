# Goal 07 Remembrance Partition S04

`G07-P2-M03-S04` completes ten content records, ten mechanic-rule records and
eight native-handler reviews. It executes both released levels of Experience:
Thrill of Escalation and Experience: Responsive Excitement, Path Resonance:
Remembrance, and all three Remembrance Resonance Formations. No native handler
is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns both Blessings, their exact level parameters, the base
  Resonance and all Formation rows;
- `UniverseBindings.xlsx` owns the ten executable mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance, audits and native-handler reviews.

`tools/goal07/author-path-partition.py` verifies every assigned Excel row,
rejects formulas/error cells and compares the normalized rows with committed
Sora 0.3.0 debug and production exports. Runtime materialization consumes only
the validated `.sora` bundle.

## Exact Blessing mechanics

### Experience: Thrill of Escalation (`612156`)

When a character successfully applies Freeze to an enemy, that same character
regenerates Energy:

```text
L1: 8 Energy
L2: 12 Energy
limit: once per complete action
```

The trigger accepts both ordinary Freeze effects and Ice Weakness Break
Freeze. An action-context filter makes the action once key valid for either
event family. Freezing several enemies in one action consumes no additional
Energy trigger.

### Experience: Responsive Excitement (`612157`)

When a character successfully applies Freeze to an enemy, that same character
receives a replacement Shield:

```text
L1: 16% of the applier's Max HP
L2: 24% of the applier's Max HP
duration: 3 applier turns
```

The dedicated Shield store owns capacity and absorption. A bounded Rule IR
counter owns lifetime, so refreshing the Shield restarts all three turns
without introducing a Blessing branch into combat core.

## Path Resonance and Formations

### Path Resonance: Remembrance (`612120`)

The first player owns a legal interrupt ability that spends 100 Path Resonance
Energy, targets every living enemy, deals non-CRIT Ice Path Additional damage,
then attempts to Freeze every target at 120% base chance for one target turn.

The public source exposes a level-battle-event `ByMaxHP` calculation but does
not expose Starclock's later path-level base-damage curve. The current approved
numeric projection uses:

```text
base_damage = 0.60 * resonance_owner_max_hp
```

Ability Tree Path Resonance damage bonuses multiply this numeric projection.
This is a registered numeric approximation only; target selection, element,
damage ordering, effect chance, resource cost and Freeze lifetime are
normative.

### Resonance Formation: Total Recall (`612121`)

Before the same Resonance deals damage and applies Freeze, every enemy receives
a 150% base-chance one-turn debuff that reduces Freeze RES by 100%. The final
specific-resistance stat is bounded to the supported `[-100%, 100%]` formula
domain before the Resonance Freeze roll.

### Resonance Formation: Rich Experience (`612122`)

Before the same Resonance applies Freeze, every enemy receives **Eonian River**
at 150% base chance for one target turn. While Eonian River is active, the
duration of each newly applied dispellable debuff or cleanseable control effect
is multiplied by two. Therefore, the one-turn Freeze from that Resonance is
authored and tested as a two-turn Freeze when Eonian River succeeds.

### Resonance Formation: First Love Once More (`612123`)

```text
battle entry: gain 40 Path Resonance Energy
each enemy that becomes Frozen: gain 5 Path Resonance Energy
```

Unlike Thrill of Escalation, the Freeze gain has no once-per-action limit and
therefore resolves once for each newly Frozen enemy. Gain is capped by the
ordinary keyed team-resource maximum.

## Generic core additions

This partition adds no Standard Universe or content-ID branch to combat core.
Shared capabilities now include:

- an effect-definition event fact naming its typed specific-resistance stat;
- an optional action-context event filter, which safely permits action once
  scopes on Effect and Toughness event families;
- a `DebuffDurationMultiplier` derived stat with authored base `1.0`;
- per-target duration resolution for both static and expression-backed effect
  definitions;
- multiple ordered programs and effect/modifier definitions on one executable
  Path Resonance;
- complete selector closure for selectors referenced by trigger filters.

## Production verification

Production integration tests prove:

- all ten assigned records and rules materialize from formal Excel/Sora data;
- Remembrance Resonance deals exact `0.60 * 100,000 = 60,000` raw fixture
  damage before mitigation;
- Rich Experience changes the same Resonance Freeze from exactly one to two
  target turns;
- enhanced Thrill restores exactly 12 Energy once per action;
- enhanced Responsive Excitement creates an exact 24,000 Shield from 100,000
  Max HP;
- First Love produces exact 40-point entry and 5-point per-Freeze resource
  changes;
- all eight exceptional candidates close as `IrSufficient`.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M03-S04.json`.
