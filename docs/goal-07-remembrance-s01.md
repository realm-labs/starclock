# Goal 07 Remembrance Partition S01

`G07-P2-M03-S01` completes seventeen content records, sixteen
mechanic-rule records, two production semantic fixtures and ten
native-handler reviews. It covers both released levels of Perfect Experience:
Fuli, Innocence, Reticence, Melancholia and Dizziness, plus the
definition-only Insensitivity record and the Remembrance Path record. No
native handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns Path, Blessing, level and exact parameter rows;
- `UniverseBindings.xlsx` binds those records to their mechanic-rule sources;
- `UniverseEvidence.xlsx` owns the Freeze fixtures, review rows and
  provenance.

`tools/goal07/author-path-partition.py` rejects formulas and spreadsheet error
cells, proves exact partition ownership, and compares assigned workbook rows
with the committed Sora 0.3.0 debug and production exports. Runtime
materialization consumes the validated `.sora` bundle.

## Shared status semantics

Freeze is a one-target-turn control effect. At the controlled unit's turn
start it suppresses the normal action and places that timeline actor at 50%
Action Gauge before normal timeline selection resumes.

Dissociation is a one-target-turn dispellable debuff. Removing its instance,
whether by natural expiry or an authored removal operation, emits one typed
`EffectRemoved` fact and deals:

```text
Dissociation removal damage = 0.30 * target MaxHP
```

The damage is non-critical Ice Additional damage. Perfect Experience: Fuli
L2 multiplies that base removal damage by `1.20`.

Effect application uses the shared checked formula:

```text
pre-clamp chance =
    base chance
  * (1 + applier Effect Hit Rate)
  * (1 - target Effect RES)
  * (1 - target-specific resistance)
```

The result is clamped once to `[0, 1]` and sampled from the labeled effect
chance stream. Base chances above 100%, such as Reticence L1/L2, remain ratios
until this final clamp.

## Exact Blessing mechanics

### Perfect Experience: Fuli (`612130`)

When an allied Attack deals ordinary damage to a Frozen enemy, it has a 100%
base chance to apply Dissociation for one target turn. The trigger runs at
most once for that target in the authored action.

L2 increases Dissociation removal damage by 20%.

### Innocence (`612131`)

After an ally causes Weakness Break, it has a 100% base chance to apply
Dissociation to that exact enemy for one target turn. The enhanced source flag
records that the application ignores Freeze-specific resistance; ordinary
Effect RES remains part of the shared chance calculation.

### Reticence (`612132`)

Each enemy owns an independent battle-scoped counter. One authored Attack
against that enemy increments it once, regardless of hit count:

```text
L1: after 6 attacks, 120% base chance to Freeze for 1 turn
L2: after 5 attacks, 150% base chance to Freeze for 1 turn
```

The threshold attempt resets the counter. Generated Additional damage, DoT,
Break and Super Break do not count as another attack.

### Melancholia (`612140`)

When an allied Attack hits an enemy already carrying Dissociation, the old
instance is removed and its removal damage is increased to:

```text
L1: 150% of normal Dissociation removal damage
L2: 200% of normal Dissociation removal damage
```

The implementation emits the normal removal damage plus the exact 50%/100%
increment. This preserves one common removal lifecycle for natural expiry and
authored removal. If the same Attack also satisfies Fuli, a newly applied
Dissociation instance is distinct from the removed old instance.

### Dizziness (`612141`)

Applying Dissociation also applies a two-target-turn dispellable vulnerability
effect:

```text
L1: damage received +36%
L2: damage received +54%
```

The vulnerability covers ordinary, DoT, Break, Super Break, Additional,
Joint and Elation damage. A two-turn duration means it remains through one
target turn after the one-turn Dissociation instance is removed.

## Generic core additions

This partition adds no Standard Universe source-ID branch. Shared combat
capabilities now include:

- exact effect-definition filtering on effect lifecycle events;
- the `IsFrozen` Rule IR predicate over generic control and Break Freeze;
- per-target Effect Hit Rate and Effect RES resolution for Rule IR effects;
- resistible base chances above 100% before final probability clamping;
- generic control-driven normal-turn suppression with the 50% Freeze gauge;
- explicit `EffectRemoved` definition identity in event and replay encoding;
- first-player, every-player and every-enemy battle-rule attachment policies.

Effect duration advancement moved to its own resolver responsibility module so
the main operation resolver remains below the 1,200-line engineering limit.

## Production verification

The production integration fixtures prove:

- Reticence L2 applies Freeze after five qualifying attacks;
- Fuli applies Dissociation only to a Frozen target;
- Dizziness attaches with Dissociation;
- enhanced natural Dissociation removal uses exactly 36% of target MaxHP
  before target vulnerability; the Dizziness fixture resolves to 55.44%;
- Melancholia L2 contributes exactly 200% of enhanced removal damage and
  removes exactly one old Dissociation instance;
- generated Additional damage cannot recursively count as an authored Attack.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M03-S01.json`.
