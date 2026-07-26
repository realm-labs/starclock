# Goal 07 Destruction Partition S04

`G07-P2-M07-S04` completes the final ten assigned Destruction records: both
levels of Polarization Receptor and Eternally Collapsing Object, the base Path
Resonance and all three Resonance Formations. Every mechanic lowers to generic
Rule IR, effects, modifiers, resource policy and queued-action primitives; no
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

## Missing-HP defensive statistics

Polarization Receptor (`612556`) and Eternally Collapsing Object (`612557`)
reuse the generic missing-HP stack rule introduced by Destruction S02. The
rule derives one stack for each complete one percent of missing HP and
dynamically applies:

```text
DEF bonus per stack:
  L1 = 0.4%
  L2 = 0.6%

Effect RES bonus per stack:
  L1 = 0.3%
  L2 = 0.45%
```

The released floating values `0.0039999997`, `0.0059999996` and
`0.0029999998` are explicitly quantized to `0.004000`, `0.006000` and
`0.003000` under the six-decimal numeric policy.

## Path Resonance: Destruction

The manual Resonance consumes 100 points from the team-scoped Destruction
Resonance resource and deals non-critical Fire Additional DMG to all enemies:

```text
damage = sum(current MaxHP - current HP for all present allies) × 250%
```

The missing-HP total is queried at resolution time. The ordinary Resonance
action and the automatic Event Horizon variant share the same typed ability
program, but only the ordinary action owns the 100-point payment policy.

## Cataclysmic Variable

Before Resonance damage, Cataclysmic Variable (`612521`) processes each
present ally in deterministic selector order:

```text
HP floor = MaxHP × 40%
effective HP consumed = max(current HP - HP floor, 0)
shield = effective HP consumed
Resonance multiplier = 250% × (100% + 20%) = 300%
shield duration = 2 target turns
```

HP consumption cannot reduce a unit below the configured floor. A prior
Cataclysmic shield is removed before the replacement is applied, and ordinary
effect expiry removes its associated shield amount.

## Extreme Helium Flash

Extreme Helium Flash (`612522`) applies Entropic Retribution to all enemies
before Resonance damage:

```text
base application chance = 150%
duration = 2 target turns
DEF = -20%
turn-start Fire Additional DMG =
  current total missing HP of all present allies × 125%
```

Application uses the standard resistible-effect RNG path. The DEF modifier
is a dynamic percent-of-base modifier with unique-per-source stacking. Each
afflicted enemy turn queries the party's current missing HP rather than a
snapshot captured when the debuff was applied.

## Event Horizon

Event Horizon (`612523`) observes damage from an opposing attack. If that
attack leaves the affected ally below 35% MaxHP, it queues one automatic
Destruction Resonance at `AfterAction`:

```text
energy payment = suppressed
maximum automatic uses = 2 per battle
maximum triggers = 1 per source action
```

The action is explicitly `Forced`, so it resolves without offering a player
decision. The auxiliary Resonance ability is registered on the participant
binding just like its manual counterpart; the resolver therefore validates
and executes it through the normal ability pipeline instead of using a
Destruction-specific callback.

## Production verification

Production tests prove:

- all ten assigned records materialize through generic Rule IR without native
  handlers;
- enhanced missing-HP rules expose exactly `+0.6% DEF` and `+0.45% Effect
  RES` per complete missing-HP percent;
- the Resonance program contains deterministic HP consumption, shield
  replacement and resistible Entropic application;
- a production battle consumes allies down to the 40% floor, creates equal
  shields and emits Fire Additional DMG;
- Entropic Retribution reads current party missing HP and deals its damage at
  the afflicted enemy's turn start;
- Event Horizon queues and resolves one free forced Resonance after an
  opposing attack leaves an ally below the threshold.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M07-S04.json`.
