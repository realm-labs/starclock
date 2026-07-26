# Goal 07 Destruction Partition S03

`G07-P2-M07-S03` completes sixteen assigned content records and sixteen
mechanic-rule records. It closes the enhanced level of Primordial Black Hole
and covers both released levels of Reflection, Orbital Redshift, Instability
Strip, Metric Reservation and Sentinel Satellite. Every executable mechanic
uses generic Rule IR, effects, modifiers and sustain/resource primitives; no
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

## Blessing-count attack and maximum HP

The S02 generic Blessing-count ATK rule also consumes the enhanced Primordial
Black Hole (`612550`) row assigned here:

```text
L2 ATK = min(selected Destruction Blessings, 9) × 7%
```

Orbital Redshift (`612552`) is one ordinary percent-of-base MaxHP modifier:

```text
L1 MaxHP +16%
L2 MaxHP +24%
```

Both values are compiled from the selected effective level. No runtime code
branches on the Blessing identity after materialization.

## Defeat prevention

Reflection (`612551`) uses the generic one-shot team defeat guard. A field
effect is applied to the active party at battle start. When lethal damage
would reach an ally, the guard:

1. clamps that damage so the actual target remains at one HP;
2. atomically consumes every matching team guard instance;
3. emits `TEAM_DEFEAT_GUARDED_SIGNAL` with the actual protected target and
   consumed effect definition;
4. lets the Blessing heal only that target.

```text
L1 heal after prevention = 1% MaxHP
L2 heal after prevention = 30% MaxHP
team trigger limit = 1 per battle
```

The typed signal is a general combat-core fact, not a Destruction-specific
resolver branch. It also removes the ambiguity of using an arbitrary
effect-removal target when a team-wide guard is consumed.

## Energy after damage or HP consumption

Instability Strip (`612553`) restores Energy when the owner is hit or consumes
HP:

```text
L1 Energy = 4
L2 Energy = 6
maximum = once per action
```

One action-scoped integer slot is reset at `ActionStart`. Incoming
`DamageApplied` and self-authored negative `HpChanged` facts share that slot,
so the two observation routes cannot grant Energy twice for one action.

## Entry and low-HP shields

Metric Reservation (`612554`) applies a two-owner-turn shield at battle
entry:

```text
L1 shield = missing HP × 36%
L2 shield = missing HP × 54%
```

Sentinel Satellite (`612555`) reacts after incoming damage leaves the owner
below 50% HP:

```text
L1 shield = MaxHP × 20%
L2 shield = MaxHP × 30%
duration = 2 owner turns
maximum = once per character per battle
```

Both mechanics use the shared timed-shield primitive. Sentinel Satellite adds
one bounded battle slot set by the same ordered apply program that creates the
shield, making the once-per-character policy rollback-safe.

## Production verification

Production tests prove:

- all six selected effective-level bindings materialize without native
  handlers;
- Reflection owns the generic `TeamDefeatOnce` guard;
- six selected Destruction Blessings produce exactly +42% enhanced
  percent-of-base ATK;
- enhanced Orbital Redshift is exactly +24% MaxHP;
- enhanced Instability Strip emits exactly six Energy through a typed
  personal-resource operation;
- both shield families emit ordinary timed Shield operations;
- a production duel preserves the actual lethal target, restores at least
  30% MaxHP after the one-HP clamp and emits the typed guard signal;
- a production attack triggers enhanced hit Energy and the once-per-battle
  low-HP shield without a fault.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M07-S03.json`.
