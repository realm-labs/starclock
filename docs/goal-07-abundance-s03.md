# Goal 07 Abundance Partition S03

`G07-P2-M05-S03` completes sixteen content records, sixteen mechanic-rule
records and eleven native-handler reviews. It executes level 2 of Dharma Rain
and both levels of Dew Delight, Extended Life, Mudra, Peril Parry and Back to
Life. No native handler is admitted.

## Authoritative authoring boundary

The production source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns Blessing definitions, levels and exact parameters;
- `UniverseBindings.xlsx` owns all sixteen assigned mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance and native-handler reviews.

`tools/goal07/author-path-partition.py` loads these workbooks with openpyxl,
rejects formula/error cells and verifies exact semantic parity with the Sora
0.3.0 binary and debug exports. Runtime materialization reads only the
validated `.sora` bundle.

## Permanent healing and MaxHP modifiers

Dew Delight (`612351`) installs an incoming-healing modifier on every
character:

```text
L1: +12% Incoming Healing
L2: +18% Incoming Healing
```

The modifier is target-directional at the shared Healing formula stage. It
therefore applies exactly once whether the healer and target are different
units or the same unit.

Dharma Rain (`612350`) level 2 raises each character's effective MaxHP by 7%
for every selected Abundance Blessing, capped at nine Blessings. The
contribution compiler supplies the validated selected count and Rule IR
receives one ordinary percent-of-base HP modifier capped at 63%.

## Event healing

Extended Life (`612352`) restores HP when each character enters battle:

```text
L1: 24% of the character's MaxHP
L2: 36% of the character's MaxHP
```

The operation executes at `BattleStarted` after lower-ID permanent modifiers
have been installed. It uses ordinary healing calculation, so Incoming
Healing and other applicable healing modifiers participate once.

Mudra (`612353`) observes `WeaknessBroken` with the attached rule owner as the
breaking actor:

```text
L1: restore 16% of the breaking character's MaxHP
L2: restore 24% of the breaking character's MaxHP
```

Each distinct enemy break event may trigger the restoration. It does not heal
unrelated allies merely because an enemy was broken.

## Healing reactions

Peril Parry (`612354`) observes positive effective healing received by its
owner:

```text
L1: +24% DEF for 1 owner turn
L2: +36% DEF for 1 owner turn
```

The state is an ordinary dispellable, one-stack effect. Repeated healing
refreshes the duration instead of adding another modifier instance.

Back to Life (`612355`) observes positive healing provided by a character
ability:

```text
L1: healer restores 12% of their MaxHP
L2: healer restores 18% of their MaxHP
```

The trigger uses `OnceScope::Action`, so multi-target and repeated healing
inside one action produce one self-heal. Ability-source filtering prevents the
resulting mode heal, Extended Life, Mudra and other path healing from
recursively triggering Back to Life. The self-heal uses ordinary healing
calculation and may itself refresh Peril Parry.

## Production verification

The production catalog and battle materializer prove:

- all six assigned families materialize from Excel/Sora as generic Rule IR;
- all eleven exceptional candidates close as `IrSufficient`;
- enhanced Dharma Rain reaches exactly 63% with nine selected Abundance
  Blessings;
- enhanced Extended Life restores 36% MaxHP and receives the 18% Dew Delight
  multiplier exactly once;
- enhanced Mudra heals only the breaking actor for 24% MaxHP;
- enhanced Peril Parry installs one refreshable 36% DEF effect for one turn;
- an action with two positive ability heals triggers one enhanced Back to Life
  heal, while both original heals still receive Dew Delight;
- the same action's forced Weakness Break emits the independent Mudra heal.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M05-S03.json`.
