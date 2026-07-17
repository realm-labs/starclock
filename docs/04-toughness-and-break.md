# Toughness and Weakness Break

## Toughness state

Toughness is normally an enemy-only resource. An attack reduces it only when:

- the attack's element matches an active weakness, or an explicit rule allows off-weakness reduction;
- Toughness is not locked/protected;
- the enemy is not already in the ordinary Weakness Broken state;
- the hit has positive authored Toughness reduction.

Store Toughness in the current raw scale used by the reference formulas. The current wiki describes most base values as multiples of 10 and notes that 10 raw Toughness is often called 1 Toughness Unit. Never mix raw points and display units in one type.

## Toughness reduction

```text
toughness_reduction =
    (base_reduction + additive_reduction)
  * (1 + toughness_reduction_increase)
  * (1 + weakness_break_efficiency + toughness_vulnerability)
  * ability_toughness_multiplier
```

Current community data documents a 300% cap for Weakness Break Efficiency. Treat the cap as configurable. See the [Toughness reduction formula](https://honkai-star-rail.fandom.com/wiki/Toughness#Toughness_Value_and_Reduction).

Apply at most the target's remaining Toughness to the resource, but retain both attempted and effective reduction in the event. Super Break and tally effects may care about one or the other; their authored rule must specify which.

## Entering Weakness Break

When the final Toughness bar reaches zero:

1. identify the element of the depleting hit;
2. calculate and apply one Break damage instance;
3. add the universal 25% action delay (2,500 AG);
4. roll the element-specific break debuff at 150% base chance;
5. enter Weakness Broken state;
6. enqueue `WeaknessBroken` triggers.

The 150% base chance still passes through Effect Hit Rate, Effect RES, and relevant debuff RES unless a specific rule states otherwise. These core results are summarized on the current [Toughness reference](https://honkai-star-rail.fandom.com/wiki/Toughness).

While the enemy remains unbroken, the hit that breaks it still receives the `0.9` broken multiplier. Later damage against the broken enemy uses `1.0`.

At the enemy's recovery turn, it exits Weakness Broken and restores Toughness to maximum. Ice Freeze can cause a skipped action before the later recovery flow; represent recovery and control as separate effects rather than one boolean callback.

## Break base damage

```text
max_toughness_multiplier = 0.5 + target_max_toughness / 40

break_base_damage =
    element_break_coefficient
  * attacker_level_multiplier
  * max_toughness_multiplier
```

Element coefficients:

| Element | Break coefficient | Applied effect |
|---|---:|---|
| Physical | 2.0 | Bleed |
| Fire | 2.0 | Burn |
| Ice | 1.0 | Freeze |
| Lightning | 1.0 | Shock |
| Wind | 1.5 | Wind Shear |
| Quantum | 0.5 | Entanglement |
| Imaginary | 0.5 | Imprisonment |

The level multiplier is lookup data, not the character's literal level. See [Reference data](reference-data.md).

## General Break damage formula

For the initial Break hit and the special DoT/additional damage caused by a break:

```text
break_damage =
    break_base_damage
  * ability_multiplier
  * (1 + break_effect)
  * break_damage_increase_multiplier
  * defense_multiplier
  * resistance_multiplier
  * vulnerability_multiplier
  * mitigation_multiplier
  * broken_multiplier
```

Break damage cannot normally CRIT and does not use ordinary DMG Boost or Weaken. It does use DEF, the matching elemental RES/PEN, matching vulnerabilities, mitigation, and the broken multiplier. See the [Break damage formula](https://honkai-star-rail.fandom.com/wiki/Toughness#Damage_Formula).

## Element-specific break effects

The default break debuffs last as follows:

| Effect | Base damage before common Break multipliers | Duration | Extra behavior |
|---|---|---:|---|
| Bleed | Normal: `0.16 * target_max_hp`; Elite/Boss: `0.07 * target_max_hp`, capped at `2 * level_multiplier * max_toughness_multiplier` | 2 turns | Physical Break DoT. |
| Burn | `1.0 * level_multiplier` | 2 turns | Fire Break DoT. |
| Freeze | `1.0 * level_multiplier` | 1 turn | On thaw: Ice additional damage, skip current action, then 50% advance to the following action. |
| Shock | `2.0 * level_multiplier` | 2 turns | Lightning Break DoT. |
| Wind Shear | `1.0 * stacks * level_multiplier` | 2 turns | 1 stack on normal enemies, 3 on Elite/Boss; maximum 5. |
| Entanglement | `0.6 * stacks * level_multiplier * max_toughness_multiplier` | 1 turn | Adds one stack when hit, maximum 5; additional delay below. |
| Imprisonment | No damage | 1 turn | Additional delay and 10% Speed reduction. |

All damaging rows pass the listed base damage through the general Break formula above. The table is transcribed from the current [type-specific Break effects](https://honkai-star-rail.fandom.com/wiki/Toughness#Type-Specific_Effects).

Additional action delays:

```text
entanglement_delay = 0.20 * (1 + break_effect)
imprisonment_delay = 0.30 * (1 + break_effect)
```

These are in addition to the universal 25% Break delay. Imprisonment's 10% Speed reduction is not scaled by Break Effect.

## Super Break extension

Super Break is not an automatic consequence of ordinary Weakness Break. An ability or field effect must enable it. Keep its calculator in the core, but enable it through authored effects.

```text
super_break_damage =
    (effective_toughness_reduction / 10)
  * attacker_level_multiplier
  * ability_multiplier
  * (1 + break_effect)
  * (1 + break_damage_increase)
  * (1 + super_break_damage_increase)
  * defense_multiplier
  * resistance_multiplier
  * vulnerability_multiplier
  * mitigation_multiplier
  * broken_multiplier
```

Super Break is Break damage, cannot normally CRIT, and does not use ordinary DMG Boost. The current formula is documented in the [Super Break section](https://honkai-star-rail.fandom.com/wiki/Toughness#Super_Break_DMG).

## Layered Toughness extensions

The authoritative model uses an ordered collection of `ToughnessLayer` records rather than flags on one bar. A layer declares its maximum/current value, eligibility predicate, damage routing order, recovery policy, whether depletion applies a Break effect, whether it changes the unit's global broken state, and which event source receives Break credit.

It must represent:

- multiple Toughness bars, where non-final layers may trigger Break damage without the full broken state;
- Exo-Toughness, a secondary bar that can be reduced after the original bar and can retrigger Break;
- weakness-agnostic or partial off-element reduction;
- locked/protected bars;
- break effects selected independently of the attack's element.

Ordinary enemies compile to one primary layer. Exo-Toughness compiles to a secondary layer whose activation/routing and retrigger behavior are authored by its provider. Multiple boss bars remain distinct layers so depletion events, phase gates, recovery, and per-layer tallies are deterministic. If the exact live-game routing or multiplier of a specific provider is not verified, its data row must remain `Researching` rather than inherit the single-bar policy silently.
