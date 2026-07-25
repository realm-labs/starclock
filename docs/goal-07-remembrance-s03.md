# Goal 07 Remembrance Partition S03

`G07-P2-M03-S03` completes sixteen content records, sixteen mechanic-rule
records, one production semantic fixture and eleven native-handler reviews.
It executes level 2 of Unspeakable Shame and both released levels of Torment
of Alienation, Lost Memory, Stone Cold Hatred, Pain & Suffering and Primordial
Hardship. No native handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns each Blessing, level and exact parameter row;
- `UniverseBindings.xlsx` binds the level records to released stage-ability
  sources;
- `UniverseEvidence.xlsx` owns the HP-threshold fixture, review rows and
  provenance.

`tools/goal07/author-path-partition.py` rejects formulas and spreadsheet error
cells, proves exact partition ownership, and compares the assigned workbook
rows with committed Sora 0.3.0 debug and production exports. Runtime
materialization consumes only the validated `.sora` bundle.

## Exact Blessing mechanics

### Experience: Unspeakable Shame (`612150`, level 2)

Each owned Remembrance Blessing reduces every enemy's Freeze RES by 8%, up to
nine counted Blessings:

```text
freeze_res_reduction = -0.08 * min(owned_remembrance_blessings, 9)
```

The count comes from the validated contribution snapshot and is resolved
before battle assembly.

### Experience: Torment of Alienation (`612151`)

All allies receive Effect Hit Rate:

```text
L1: +16%
L2: +24%
```

This is an ordinary party stat modifier at the effect-chance query boundary.
It affects every resistible effect whose source is an ally and does not alter
guaranteed applications.

### Experience: Lost Memory (`612152`)

The first allied Attack that lowers an enemy from at least 50% HP to below 50%
HP in a battle attempts to Freeze that same enemy for one target turn:

```text
L1: 70% base chance
L2: 100% base chance
```

The once key is owned by the enemy's rule instance, so each enemy can trigger
once per battle. The comparison uses `hp_after` from the authoritative damage
event and the target's derived maximum HP. Damage that finds the target
already below the threshold does not retroactively trigger it.

### Experience: Stone Cold Hatred (`612153`)

Skill and Ultimate damage against a currently Frozen enemy receives:

```text
L1: +36% DMG Boost
L2: +54% DMG Boost
```

The modifier lives on the Frozen target but contributes to the attack's DMG
Boost stage only when the observed ability carries the matching Skill or
Ultimate tag.

### Experience: Pain & Suffering (`612154`)

When an enemy becomes Frozen, attacks against that enemy gain 100% CRIT Rate
for a finite number of complete attacking actions:

```text
L1: next 1 attacking action
L2: next 2 attacking actions
```

One charge is consumed after the complete action if at least one ordinary
attack hit that enemy. Multi-hit and bounce hits in the same action consume
only one charge. Charges are cleared when the enemy is no longer Frozen.
Each target compares the action's shared deterministic CRIT draw against its
own final CRIT threshold, so one multi-target hit can legitimately crit some
targets and not others without consuming extra RNG draws.

### Experience: Primordial Hardship (`612155`)

Frozen enemies receive additive incoming-damage vulnerability:

```text
L1: +16% damage taken
L2: +24% damage taken
```

The target-side modifier applies at the Vulnerability stage to ordinary, DoT,
Break, Super Break, Additional, Joint and Elation damage. It remains separate
from source-side DMG Boost.

## Generic core additions

This partition adds no Standard Universe or Blessing ID branch to combat
core. Shared capabilities now include:

- typed `hp_before` and `hp_after` event properties;
- typed effect-category and Toughness-event-kind filters;
- action-kind, ability-tag and source-class propagation into modifier queries;
- target-owned conditional DMG Boost and vulnerability contributions;
- target-owned additions to final CRIT probability;
- one shared raw CRIT draw per shared-CRIT hit, evaluated against
  target-specific thresholds;
- a collision-safe rule-state journal fact when multiple event once keys are
  cleared together.

Freeze-linked effects observe generic Control-effect and Ice base-effect
lifecycle events. They do not inspect the ID of the Freeze source.

## Production verification

The production integration fixtures prove:

- all six assigned rule sources materialize from the selected enhanced rows;
- enhanced Lost Memory freezes on the exact first hit crossing below half HP;
- enhanced Stone Cold Hatred produces exactly a `1.54` Skill-damage ratio;
- enhanced Primordial Hardship produces exactly a `1.24` incoming-damage
  ratio;
- enhanced Pain & Suffering grants and consumes exactly two per-target
  critical exposures at complete-action boundaries;
- existing Dissociation vectors now include the already-authored Dizziness
  vulnerability at the correct target formula stage;
- every exceptional candidate closes as `IrSufficient`.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M03-S03.json`.
