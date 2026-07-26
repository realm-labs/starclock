# Goal 07 Hunt Partition S04

`G07-P2-M06-S04` completes ten assigned content records and ten mechanic-rule
records: both released levels of Arboreal Volley and Arrow Shades Bow, the Hunt
Path Resonance, and its three Resonance Formations. All behavior is expressed
by generic battle definitions, Rule IR, selectors, effects and shared resolver
policies; no native handler is admitted.

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

## Turn-start Blessings

Arboreal Volley (`612456`) observes the owning character's `TurnStarted` fact
and uses ordinary personal-Energy mutation:

```text
L1: restore 4 Energy
L2: restore 6 Energy
```

Arrow Shades Bow (`612457`) maintains one owner-local election bit for the
most recent allied actor. At the next allied `TurnStarted`, that elected rule
owner applies an ATK effect whose value is captured from the previous actor:

```text
L1: ATK += 10% of the last acting ally's current ATK
L2: ATK += 15% of the last acting ally's current ATK
```

The effect is replaced at the beneficiary's next turn start. The snapshot
prevents later changes to the previous actor's ATK from retroactively changing
the granted value and avoids a path-global “last ally” singleton.

## Hunt Path Resonance

The Resonance is an ordinary interrupt ability paid from the keyed
`standard-universe.path-resonance-energy` team resource. Its damage program
selects the living allied character with the highest effective ATK, using
stable unit identity as the tie-break, then deals Wind Additional damage to
all enemies:

```text
damage = highest current ally ATK × 550%
cost   = 100 Resonance Energy
```

The ability owner is therefore only the command carrier; party order no longer
incorrectly determines the damage base.

## Resonance Formations

Star Hunter (`612421`) selects the same highest-ATK ally, immediately grants
that character an extra turn and applies Light-Hunting Celestial Arrow for the
next ability:

```text
CRIT DMG += current CRIT Rate × 50%
first enemy defeated by that ability: grant one additional extra turn
effect expires after the ability or after granting the defeat reward
```

Bow and Arrow (`612422`) uses a reusable hit policy: each target below 50% HP
is guaranteed to receive a CRIT, while other targets retain independent normal
CRIT sampling. A before-/after-hit effect adds 50% CRIT DMG only for this
Resonance action. Each enemy defeated by the Resonance restores 50% of the
current authored Resonance capacity.

Perfect Aim (`612423`) changes the keyed resource capacity without changing
the 100-point activation cost:

```text
maximum Resonance Energy = 200
each allied TurnStarted  = restore 3% × 200 = 6 Energy
Bow and Arrow defeat     = restore 50% × 200 = 100 Energy
```

## Production verification

Production tests prove:

- both effective Blessing bindings and all three Formation bindings
  materialize as native-handler-free Rule IR;
- Arboreal Volley restores exactly 6 Energy at level 2;
- Arrow Shades Bow uses an application snapshot and the exact 15% multiplier;
- Resonance damage reads the stat-descending highest-ATK selector and retains
  the exact 550% multiplier;
- Bow and Arrow installs the exact below-50% conditional CRIT policy and
  restores 100 Energy when Perfect Aim is active;
- Perfect Aim materializes a 200-point resource and restores exactly 6 Energy
  per allied turn;
- the complete Resonance and all three Formations execute in a production
  battle without a fault and consume one 100-point charge.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M06-S04.json`.
