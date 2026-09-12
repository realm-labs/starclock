# Divergent Universe Curio battle stats and lifetime

The decision workbook's `CurioBattleStats` table binds Tawot Cards state 9068
to an all-allies 35% SPD bonus and state 9073 to a 50% outgoing final-damage
bonus, each with a five-battle discard limit. These are separate
from the three/five-domain lifetimes of states 9070/9079. These are different
states of handbook identity 9068, not interchangeable trigger clocks.
State 9069's [battle-healing reaction](divergent-universe-curio-battle-reactions.md)
shares the verified-result lifetime stage, but has a separate authored effect
and a separate explicitly admitted [paid service producer](divergent-universe-tawot-service.md).

## Released inputs and policy

Sources 43–45 in `DivergentUniverseDecisions.xlsx` pin released 4.4 data from
`Dimbreath/turnbasedgamedata` at
`fd978d6ef09f941fba644c731ab54abd6f7c3568`, accessed 2026-09-12:

| Input | Exact locator |
| --- | --- |
| Mode state | `RogueTournMiracle.json`: 9068, `Tourn3`, handbook 9068, effect 2068 |
| Parameters | `RogueMiracleEffect.json`: 2068, `ParamList[0]=0.35`, `[1]=5` |
| Released English text | `TextMapEN.json`: hash `16574523273451283809` |

The rows retain full input digests. Magnitude and battle limit are exact;
`VersionedProjectPolicyBasePercentVerifiedBattleLifetime` explicitly owns
the following lower-confidence execution choices:

- An active holding with positive allowance contributes at immutable battle
  assembly, using `PercentOfBase` rather than total-speed multiplication.
- Each player combatant receives an attributed non-dispellable modifier;
  linked **unit** subjects inherit the binding. Timeline-only actors do not.
  Enemy specs, original build fields, locked build digests and existing
  bindings are preserved.
- The shared [action-order resolver](action-timeline-boundary.md) applies
  the modifier to actual selection, elapsed time and gauge advancement.
  No mode-specific action scheduler or base-speed rewrite is used.
- A verified `Won` or `Lost` result consumes one active battle allowance.
  `Faulted`, rejected or duplicate results do not consume it again. The
  immutable battle never mutates the live Activity inventory.
- Destruction pauses both contribution and consumption; repair resumes the
  remaining allowance. Replacement/reacquisition initializes a fresh limit.
  The last counted result removes state, charge and activation entries rather
  than marking the Curio destroyed. A trusted zero allowance contributes
  nothing and is removed at the next counted result.
- Domain selection and internal graph nodes never consume this battle counter.

Replace scope, stacking, dispellability, activation and terminal-result counting
independently when released executable evidence or reproducible observations
establish them. Alternatives include roster-only applicability, total-speed
scaling and victory-only counting; none is presented as observed parity.

## Settlement and evidence boundaries

Fresh-view generated settlement stages run in order:

`24060 base fragments → 23700 Green Miracle → 24050 Sage's Leaf Robe →
24110 Curio battle lifetimes → 23702 ordinary Blessing offer`.

They share verified carry and one atomic commit with automatic graph pumping.
A later offer failure rolls back prior discard, battle carry, pending result,
inventory and RNG. Loss can discard even after the graph becomes terminal.

Focused tests cover real assembly, all initial actor gauges and elapsed time,
active/destroyed/repaired/replaced holdings, exact five-count operation vectors,
verified win/loss/fault, duplicate results and late-stage rollback. A public
initial-event acquisition followed through later domains and fresh transcript
replay covers both Ordinary and Cyclical. That baseline contains only three
battles: it does **not** prove five-battle exhaustion through a complete public
run. Counterfactual fixtures are labeled separately from public producers.

The lexicographically first-state reward policy currently selects 9068 for
handbook 9068. This provides a public producer for this component but does not
establish original random-state selection parity, implement all twelve Tawot
Cards states or close any original mechanic program/source obligation.

## Final-damage component

Sources 52–54 pin the same released revision and access date, with full digests:

| Input | Exact locator |
| --- | --- |
| Mode state | `RogueTournMiracle.json`: 9073, `Tourn3`, handbook 9068, effect 2073 |
| Parameters | `RogueMiracleEffect.json`: 2073, `ParamList[0]=0.5`, `[1]=5` |
| Released English text | `TextMapEN.json`: hash `7393919144864761857` |

The 50% magnitude and five-battle limit are exact. The independent
`VersionedProjectPolicyOutgoingFinalMultiplierVerifiedBattleLifetime` selects
an active positive-allowance holding at immutable battle assembly and attaches
six source-attributed, non-dispellable `1.5` modifiers, one per purpose:
Direct, DoT, Additional, Elation, Break and Super Break. Player units, including
linked unit subjects, receive the binding; timeline-only actors do not. This
does not establish production summon construction. Original build fields,
locked build identity, enemy specs and existing bindings remain unchanged.

Each purpose uses a Product group at the shared
[`DamageFinalMultiply` boundary](final-damage-boundary.md). Across groups,
factors multiply in canonical order. Checked nearest-ties-even fixed-point
multiplication precedes flooring applied damage, shields and HP subtraction;
`1.9 × 1.5` yields raw `2.85` and calculated `2`. Continuing Break effects
retain their original applier. True damage and other source-modifier-bypassing
operations remain unamplified, and a positive absolute override still wins.
These scope, stacking and bypass decisions remain lower-confidence policy;
replace each independently when released executable evidence or reproducible
observations establish it. No change to ATK, additive DamageBoost, healing or
speed is implied.

9073 shares the active-only verified win/loss lifetime described above. Tests
cover production assembly for all six purposes, actual ordinary HP/shield
events, Break effect ownership through the shared resolver, deterministic
repeated probes, destroy/repair, zero allowance, replacement, reacquisition,
five-result operation vectors, terminal-result projections, duplicate rejection
and whole-settlement rollback after a later reward failure.

Both run families also buy this exact state through the public paid service,
execute three actual proxy battles and verify an encoded transcript against a
fresh production fixture. During the run the allowance decreases once per
battle; accepted run finalization clears the remaining inventory. This is not
five-battle public exhaustion, original Forge placement parity, or completion
of all Tawot states/source programs.
