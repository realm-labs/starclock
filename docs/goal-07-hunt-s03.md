# Goal 07 Hunt Partition S03

`G07-P2-M06-S03` completes sixteen assigned content records and sixteen
mechanic-rule records: the enhanced level of Vermeil Bow and White Arrow, plus
both released levels of Sit Life, Hit Death; Stay High, Strike Low; Thundering
Chariot; Constellation Surge; and Astral Menace. All behavior is generic Rule
IR and no native handler is admitted.

## Authoritative authoring boundary

The openpyxl-authored `Universe.xlsx`, `UniverseBindings.xlsx` and
`UniverseEvidence.xlsx` workbooks remain the editable source. The focused
partition check rejects formula/error cells and compares every assigned row
with the committed Sora 0.3.0 binary and debug exports.

Released structured values remain the numeric source of record. Public
mechanism descriptions were cross-checked against the
[Simulated Universe Paths reference](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Paths)
and the
[Star Rail Station Blessing catalog](https://starrailstation.com/en/simuniverse/current/blessings),
accessed 2026-07-26.

## Permanent offensive stats

Sit Life, Hit Death (`612451`) and Stay High, Strike Low (`612452`) install
ordinary battle-lifetime modifiers:

```text
Sit Life, Hit Death:   +11% / +16% CRIT Rate
Stay High, Strike Low: +20% / +30% CRIT DMG
```

They share the normal stat pipeline and do not introduce Hunt-specific stat
storage. Vermeil Bow and White Arrow (`612450`) uses the contribution compiler
from S02; its enhanced row produces +4% percent-of-base SPD per selected Hunt
Blessing, capped at nine Blessings.

## Timeline mechanics

Thundering Chariot (`612453`) observes a Weakness Break credited to the rule
owner and delays the broken target through the generic timeline operation:

```text
L1: 20% action delay
L2: 30% action delay
```

Astral Menace (`612455`) runs at the owner's `TurnEnded` boundary, after the
ordinary action gauge reset:

```text
L1: 8% action advance
L2: 12% action advance
```

The operation changes the normal timeline gauge and does not grant an extra
turn. It therefore composes with turn-duration effects and the same stable
timeline ordering used by S01 defeat and Break advances.

## Entry speed lifetime

Constellation Surge (`612454`) applies one non-dispellable SPD effect at battle
start:

```text
L1: +30% SPD
L2: +45% SPD
```

The first `DamageApplied` event targeting that character removes the effect.
The bonus is represented by the same effect/modifier attachment pipeline used
by ordinary combat buffs; no Boolean mode flag participates in speed queries.

## Production verification

Production tests prove:

- all six selected effective bindings materialize as native-handler-free Rule
  IR;
- six selected Hunt Blessings produce exactly 24% SPD from enhanced Vermeil
  Bow and White Arrow;
- enhanced permanent critical modifiers retain 16% CRIT Rate and 30% CRIT
  DMG;
- enhanced Thundering Chariot emits exactly 30% delay on Weakness Break;
- enhanced Astral Menace emits exactly 12% advance at `TurnEnded`;
- enhanced Constellation Surge installs 45% percent-of-base SPD and removes
  the owning effect on the first damage event.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M06-S03.json`.
