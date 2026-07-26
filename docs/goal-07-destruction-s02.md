# Goal 07 Destruction Partition S02

`G07-P2-M07-S02` completes sixteen assigned content records and sixteen
mechanic-rule records. It covers the released levels of five Destruction
Blessings plus the base and first-level rows of a sixth Blessing. Every
executable level is lowered to generic Rule IR, effects and modifiers; no
native handler is admitted.

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

The structured decimal `0.007999999` used by Indicative Depth of Field and
Instability Strip is quantized to the authoritative six-decimal numeric
domain as `0.008000`. The completion receipt records this transparent,
replaceable approximation for both released levels.

## Fighting Spirit mitigation

Indicative Depth of Field (`612542`) extends the shared Fighting Spirit model
introduced by S01. The synchronized engine-effect stack drives one ordinary
Mitigation-stage modifier for each damage purpose:

```text
damage reduction = effective Fighting Spirit stacks × 0.8%
L1 Fighting Spirit cap = 35
L2 Fighting Spirit cap = 45
```

The modifiers consume the effect `source_stack_slot`; they do not query
Blessing identities or duplicate Fighting Spirit state. The cap upgrade is
therefore visible to every S01 producer while preserving one canonical stack
source.

## Missing-HP and low-HP modifiers

Instability Strip (`612543`) synchronizes an ordinary effect after battle
entry and every owner HP change. Its stack count is the complete missing-HP
percentage, rounded down:

```text
missing percentage points = floor((MaxHP - current HP) / MaxHP × 100)
L1 ATK = +0.8% × missing percentage points
L2 ATK = +0.8% × missing percentage points
L2 DEF = +0.5% × missing percentage points
```

Reflection (`612544`) uses a separate 0/1/2-tier effect state machine. This is
deliberate: modifier evaluation consumes synchronized effect stacks and never
performs a live HP query inside the stat resolver.

```text
L1 below 50% HP: All-Type DMG +40%
L1 below 35% HP: no additional bonus
L2 below 50% HP: All-Type DMG +50%
L2 below 35% HP: an additional +20%, total +70%
```

The damage modifier is authored for ordinary, DoT, Break, Super Break,
Additional, Joint and Elation damage purposes.

## Bounded healing

Disciplinary Flicker (`612545`) reacts after incoming damage and after
self-authored HP loss while current HP is below 35%. It uses a battle slot
reset at every action start to enforce the released per-action cap:

```text
L1 healing per trigger = 12% MaxHP; cap per action = 36% MaxHP
L2 healing per trigger = 20% MaxHP; cap per action = 50% MaxHP
effective healing = min(per-trigger healing, remaining action cap)
```

Incoming damage and its associated HP-change event cannot double-trigger the
Blessing: incoming damage uses `DamageApplied`, while the HP-change route
requires the owner to be both actor and target and requires a negative delta.
The already-resolved Blessing amount bypasses a second healing-formula pass.

## Ultimate shield

Construct: Firmness (`612546`) uses the shared timed-shield primitive after an
owner Ultimate resolves:

```text
L1 shield = missing HP × 25%
L2 shield = missing HP × 25% + MaxHP × 7%
duration = 2 owner turns
```

Application, replacement, owner-turn advancement and expiration remain
ordinary shield/effect lifecycle operations.

## Blessing-count attack

Universal Heat Death Characteristic (`612550`) receives the validated count
of selected Destruction Blessings from the contribution compiler and lowers
it to one percent-of-base ATK modifier:

```text
L1 ATK = min(selected Destruction Blessings, 6) × 5%
```

The enhanced level belongs to the next frozen partition; this partition
claims only the base record and released level-one row assigned by the
manifest.

## Production verification

Production tests prove:

- all six selected effective-level bindings materialize as native-handler-free
  Rule IR;
- the enhanced Fighting Spirit cap is exactly 45 and all seven mitigation
  modifiers consume the shared stack slot;
- missing-HP ATK and DEF factors are exactly `0.008000` and `0.005000`;
- low-HP damage is represented by seven effect-backed damage-purpose
  modifiers;
- bounded healing owns an action-reset scalar slot and emits an unmodified
  typed Heal operation;
- the Ultimate trigger emits the generic timed Shield operation;
- all S02 rules execute from 30% HP in a production battle without a fault.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M07-S02.json`.
