# Goal 07 Preservation Partition S01

`G07-P2-M02-S01` completes the first frozen Preservation partition: 17
records, 16 rule records, five semantic fixtures and ten native-handler
reviews. No native handler is admitted.

## Authoritative authoring boundary

The owned rows remain in the formal workbooks:

- `Universe.xlsx`: Path, Path-to-Blessing, Blessing, BlessingLevel and exact
  BlessingParameter rows;
- `UniverseBindings.xlsx`: the 16 `UniverseMechanicRule` rows;
- `UniverseEvidence.xlsx`: content audit, source records and the damage,
  defense, DoT, shield and Preservation fixtures.

`tools/goal07/author-path-partition.py` reads these files with openpyxl,
rejects formulas and spreadsheet errors, validates all assigned links and
provenance, and compares the selected rows with Sora 0.3.0 debug output.
Runtime code consumes only the generated `.sora` bundle and validated domain
definitions.

The normalized source records are
`content-reference/standard-universe-v1/blessings.json`,
`blessing-levels.json`, `mechanic-rules.json` and `paths.json`. The exact
effect wording was also cross-checked against the public Bilibili Star Rail
Wiki and Honkai: Star Rail Wiki entries. Local provenance rows retain the
upstream revision, locator and evidence hash.

## Executable formulas

All values below are fixed-point ratios. Let:

- `S` be the rule owner's effective current shield;
- `S_before` be the effective shield immediately before the observed hit;
- `S_team` be the effective shield sum for all other player units;
- `DEF` be the rule owner's current derived Defense;
- `Q` be the resulting Quake damage before ordinary target-side mitigation.

### Divine Construct: Resonance Transfer (`612030`)

On each distinct enemy target hit by an attack:

```text
Q_base(L1) = S
Q_base(L2) = S + 0.20 * S_team
```

The rule uses `TargetWithinAction`, so a multi-hit action damages a target at
most once while a blast or AoE action can trigger once for each distinct
target. The owner, actor, primary target and source remain distinct in the
cause chain.

### Divine Construct: Metastatic Field (`612031`)

When a shielded character is attacked:

```text
Q_base(L1) = 3.40 * current_shield
Q_base(L2) = 4.20 * S_before
```

The enhanced level reads the pre-hit shield snapshot. This Quake cannot reduce
the attacker below 1 HP. The generic damage operation therefore carries an
explicit `can_defeat` policy instead of adding a Blessing-specific branch.

### Divine Construct: Macrosegregation (`612032`)

At battle start, every player receives a special shield:

```text
special(L1) = 0.01 * MaxHP
special(L2) = 0.10 * MaxHP
```

When another source grants shield to that unit, the special shield grows by
`1.00` or `1.30` times the newly granted amount. Every two owner turns the
special shield is removed and recreated from its base value. A dedicated
shield store records the originating effect, allowing deterministic
effect-scoped removal without treating shields as ordinary buffs.

### Interstellar Construct: Quadrangular Pyramid (`612040`)

The Blessing modifies all compatible Quake:

```text
Q_boost(L1) = +10%
Q_boost(L2) = +15%
splash(L1) = 25% of applied Q to one adjacent enemy
splash(L2) = 30% of applied Q to every other enemy
```

The primary Quake boost is composed into the shared formula before damage is
emitted. Splash reads the applied Quake fact and iterates enemies in stable
formation order.

### Interstellar Construct: Shear Structure (`612041`)

Compatible Quake applies one-turn Physical Bleed:

```text
base chance = 65% / guaranteed
Bleed = min(0.05 * target MaxHP, 0.80 * applied Q)
duration = 1 target turn
```

The unenhanced level uses the standard resistance pipeline and deterministic
RNG stream. The enhanced level performs no draw.

### Interstellar Construct: Solid Solution (`612042`)

This record increases compatible Quake by current Defense; it does not
increase shields:

```text
DEF contribution(L1) = 0.80 * DEF
DEF contribution(L2) = 1.20 * DEF
Q = (Q_base + DEF contribution) * (1 + Q_boost)
```

The contribution is a shared compile-time formula primitive. It is folded
into attack and retaliatory Quake before Quadrangular Pyramid's boost, so
splash and Bleed observe the same authoritative Quake result. Its two level
records remain assigned to `G07-P2-M02-S02`, but the S01 definition rule is
already exercised against the released level parameters.

## Generic core additions

This partition adds only reusable combat concepts:

- current and pre-event effective-shield queries;
- signed shield-change event facts;
- effect-scoped shield removal;
- per-target-within-action once scope;
- explicit nonlethal rule damage;
- rule-owner identity independent of the observed event owner;
- Rule IR shield execution through the normal shield formula and store.

The shield origin changes authoritative state semantics. New battle states use
`SCBS` codec version 5 and `sha256-v6`; battle-event payload version 4 adds
effect-scoped shield-removal facts. Historical event payload versions 1–3
remain encodable.

## Production verification

The production integration fixture executes:

- four Macrosegregation shields at battle start;
- enhanced attack Quake with teammate shields, `120% DEF`, a `15%` boost and
  `30%` multi-enemy splash;
- enhanced Quake Bleed application;
- Macrosegregation's two-turn removal/reissue cycle;
- Metastatic Field attachment and exact pre-hit/nonlethal formula coverage;
- Solid Solution's exact current-Defense contribution.

The core test independently proves that each nonlethal damage emission clamps
at 1 HP. The typed Preservation oracle retains exact parameters for both
levels. The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M02-S01.json`.
