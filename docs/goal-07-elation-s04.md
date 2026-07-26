# Goal 07 Elation Partition S04

`G07-P2-M08-S04` completes Platinum Age, Clockwork Apple, Path Resonance:
Elation and all three released Resonance Formations.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. They are authored with openpyxl, exported through Sora
0.3.0 and checked against the focused partition golden. Runtime lowering
consumes validated domain records and released binding keys, never workbook
rows.

Parameters, modifier lifetimes and callback order were verified against the
pinned `RogueMazeBuff.json`, `Level_RogueBuff_Ability_3.json`,
`Level_RogueBuff_Ability_4.json` and
`Avatar_RogueBattleevent126_Ability.json` records. Released English and
Chinese descriptions were cross-checked in `TextMapEN.json` and
`TextMapCHS.json`, accessed 2026-07-26.

The upstream BattleEvent actor derives its damage scale through private
event-level stats not exposed as a public content row. Starclock maps that
scale to the highest current allied ATK and records the substitution as a
reviewable approximation. Hit count, random element selection, formation
thresholds, energy behavior and modifier values remain exact.

## Platinum Age and Clockwork Apple

After an eligible character completes one follow-up attack:

```text
Platinum Age L1/L2: +40% DEF for 1/2 owner turns
Clockwork Apple L1/L2: +16% SPD for 1/2 owner turns
```

Each source uses a one-stack, replace-by-caster timed effect. Follow-up and
counter tags are independent routes. Champion's Dinner adds character
Ultimates as a third route without changing action identity.

The neutral Path Resonance BattleEvent is explicitly excluded from these
character callback routes. It still deals Elation follow-up damage, but it
does not pretend that the player character hosting the interrupt command
launched the attack.

## Path Resonance: Elation

At 100 Resonance Energy, the manual interrupt emits an all-enemy sequence:

```text
base hit count = uniform integer in [3, 5]
element per hit = independent uniform choice from the seven combat elements
damage per hit = highest current allied ATK × 25%
critical hits = disabled
damage class = Elation
```

Target iteration, hit-count draws and element draws use stable candidate
ordering and dedicated deterministic draw purposes. The repeated damage
program runs once before the ordinary all-target hit envelope, preventing
the program from being multiplied by the number of selected enemies.

The normal Standard Universe Resonance damage-ratio projection multiplies
the 25% coefficient at the same checked fixed-point boundary.

## Doomsday Carnival

Every committed Elation Resonance damage event has a 150% base chance to add
one stack of Sensory Pursuit for one target turn.

```text
follow-up vulnerability per stack = 8%
stack policy = refresh duration and add stacks
```

The debuff uses target-side Vulnerability modifiers. Ordinary, additional and
joint follow-up damage require the follow-up ability tag. Elation damage uses
its dedicated formula purpose so subsequent Resonance hits receive the same
released Sensory Pursuit interaction.

## Dance of Growth

Dance of Growth changes the shared resource contract:

```text
maximum Resonance Energy = 200
manual use threshold = 100
manual use consumes all currently available Energy
extra hit count = floor(excess Energy / 20)
```

At 200 Energy, the fixed 100-point action cost is followed by an explicit
100-point Rule IR spend, and the random range becomes 8–10 hits. At 100
Energy, the range remains 3–5. Invalid or insufficient spends retain the
normal transactional rollback guarantee.

## Instant Win

Instant Win is evaluated against the effective Resonance maximum:

```text
battle-entry gain = 40% maximum = 40 or 80
character follow-up gain = 5% maximum = 5 or 10
```

The entry value is materialized into the battle resource specification. The
action-complete gain is a checked team-resource mutation once per eligible
character action. Resonance-origin actions are excluded from the trigger, so
using the Resonance does not refund its own cost.

## Production verification

The focused tests prove:

- all six S04 assignments materialize without native handlers;
- enhanced Platinum Age and Clockwork Apple preserve exact values, two-turn
  lifetime and all three character routes;
- the complete Resonance contains the exact 3–5 and 8–10 random-hit programs;
- Dance of Growth exposes a 200-point resource and Instant Win starts it at
  exactly 80;
- Instant Win restores exactly 10 points per eligible character follow-up at
  the 200-point maximum;
- a charged production two-enemy battle spends all 200 Energy and emits one
  deterministic 8–10-hit all-enemy Resonance sequence without faults.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M08-S04.json`.
