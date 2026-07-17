# Damage and Sustain Formulas

All percentages in formulas are decimal values. For example, 20% is `0.20`.

## Derived stats

The common derived-stat pattern is:

```text
HP  = base_hp  * (1 + hp_percent)  + flat_hp
ATK = base_atk * (1 + atk_percent) + flat_atk
DEF = base_def * (1 + def_percent) + flat_def
SPD = base_spd * (1 + spd_percent) + flat_spd
```

For player characters, equipment base stats that explicitly count as base stats must be added before percentage modifiers. Conditional and final modifiers belong to later query layers, not these four equations.

## General ordinary damage

For ordinary direct damage, ordinary character DoT, and ordinary additional damage:

```text
damage =
    base_damage
  * original_damage_multiplier
  * crit_multiplier
  * damage_boost_multiplier
  * weaken_multiplier
  * defense_multiplier
  * resistance_multiplier
  * vulnerability_multiplier
  * mitigation_multiplier
  * broken_multiplier
```

This is the current formula documented by the [Star Rail Wiki Damage page](https://honkai-star-rail.fandom.com/wiki/Damage#General_Damage_Formula) and is consistent with the independent worked examples on [Prydwen](https://www.prydwen.gg/star-rail/guides/damage-formula).

### Base damage

The common form is:

```text
base_damage = scaling_stat * ability_multiplier + additive_damage
ability_multiplier = base_ability_multiplier + multiplier_increase
```

An authored damage expression should support sums of multiple scaling terms, because some abilities scale from ATK plus HP, DEF, HP lost, or another tally.

### CRIT

```text
crit_multiplier = if can_crit && crit_roll_succeeds {
    1 + crit_damage
} else {
    1
}
```

Clamp the roll probability to `[0, 1]`. Multi-hit attacks roll CRIT separately per hit. Ordinary DoT does not CRIT unless an authored exception says otherwise.

### DMG Boost

Relevant bonuses within this block are additive:

```text
damage_boost_multiplier =
    1
  + matching_element_damage_boost
  + matching_ability_or_damage_type_boost
  + all_damage_boost
```

Only tags matched by the damage instance are included. Do not multiply each DMG Boost source separately.

### Weaken

```text
weaken_multiplier = 1 - total_weaken
```

This mostly applies to damage dealt by a weakened attacker. Keep it separate from target mitigation.

### Defense

The generic formula from actual target DEF is:

```text
defense_multiplier =
    1 - target_def
        / (target_def + 200 + 10 * attacker_level)
```

Standard enemy base DEF is:

```text
enemy_base_def = 200 + 10 * enemy_level
```

Combining enemy base DEF with DEF bonus, reduction, and attacker ignore:

```text
effective_def_factor = max(
    0,
    1 + def_bonus - def_reduction - def_ignore
)

defense_multiplier =
    (attacker_level + 20)
    / ((enemy_level + 20) * effective_def_factor
       + attacker_level + 20)
```

DEF reduction and DEF ignore add inside the same effective-DEF block. Keep source attribution even though the formula combines them, because conditions may apply to only one source.

### Elemental resistance and penetration

```text
effective_res = target_element_res - attacker_element_res_pen
resistance_multiplier = 1 - effective_res
```

The documented effective-RES bounds are `[-1.0, 0.9]`, producing a multiplier between `2.0` and `0.1`; keep bounds configurable. Common enemies often use 0% RES to a weakness and 20% neutral RES, but these are defaults, not universal rules, and mode/boss values can be much higher. See the current [Damage RES reference](https://honkai-star-rail.fandom.com/wiki/Damage_RES).

Weakness and resistance are separate authored concepts. Having an elemental weakness permits Toughness damage and often correlates with 0% RES, but the engine must not derive one from the other.

### Vulnerability

Relevant target vulnerabilities within this block are additive:

```text
vulnerability_multiplier =
    1
  + matching_element_vulnerability
  + matching_damage_type_vulnerability
  + all_damage_vulnerability
```

Vulnerability is distinct from the attacker's DMG Boost and therefore multiplies it.

### Mitigation

Independent mitigation effects multiply:

```text
mitigation_multiplier = product(1 - mitigation_i)
```

Do not combine mitigation with Weaken or elemental RES.

### Broken multiplier

Against an enemy that currently has an active, non-depleted Toughness bar:

```text
broken_multiplier = 0.9
```

Against a Weakness Broken enemy:

```text
broken_multiplier = 1.0
```

This 10% reduction applies to incoming damage while Toughness remains, including the Break damage created by the hit that depletes it. See [Toughness](https://honkai-star-rail.fandom.com/wiki/Toughness).

## Worked ordinary-damage vector

Use this synthetic vector as an early golden test:

- base damage: 1,000;
- attacker and enemy level: 80;
- CRIT occurs with 50% CRIT DMG, so CRIT multiplier is 1.5;
- matching total DMG Boost: 20%, so 1.2;
- standard same-level enemy with no DEF modification: DEF multiplier 0.5;
- enemy RES: 20%, no penetration, so 0.8;
- no vulnerability, weaken, or extra mitigation;
- enemy is not broken, so 0.9.

```text
1000 * 1.5 * 1.2 * 0.5 * 0.8 * 1.0 * 0.9 = 648
```

When the same enemy is broken, the result is `720`.

## Damage types that bypass the general pipeline

Break and Super Break use formulas in [Toughness and Break](04-toughness-and-break.md). True damage is an authored amount that bypasses ordinary multipliers unless its definition explicitly says otherwise.

Do not implement a damage type by toggling a few ordinary flags. Give each formula family a named calculator so additions remain auditable.

## Healing

```text
base_healing = scaling_stat * healing_ratio + additive_healing

healing_multiplier =
    1
  + outgoing_healing_boost
  + incoming_healing_boost
  - incoming_healing_reduction

healing = base_healing * healing_multiplier
```

This formula is documented by the [Outgoing Healing Boost reference](https://honkai-star-rail.fandom.com/wiki/Outgoing_Healing_Boost). Clamp applied healing to missing HP, but log both calculated healing and effective healing so overheal-dependent mechanics remain possible.

## Shields

```text
shield_value =
    (scaling_stat * shield_ratio + additive_shield)
    * (1 + shield_bonus)
```

Incoming damage is applied to shields before HP; overflow reduces HP. The base game has unusual non-additive concurrent-shield behavior: the effective visible protection is the largest remaining shield, while every shield instance absorbs the incoming amount simultaneously and may expire in the background. This is documented on the [Shield reference](https://honkai-star-rail.fandom.com/wiki/Shield#Shield_Stacking).

Therefore preserve separate `ShieldInstance`s. Do not collapse them into one integer. Character-specific shields may define refresh, additive stacking, or caps and should supply an explicit stacking policy.

## Numeric and rounding policy

The public sources reliably describe formula factors but not one universal internal precision/rounding rule for every mechanic. For the first milestone:

- calculate with the pinned decimal fixed-point representation from [Cross-platform determinism and numeric policy](09-determinism-and-numerics.md);
- do not round intermediate multipliers;
- finalize ordinary damage, healing, and shield creation once with the documented project rounding policy unless a verified mechanic requires another boundary;
- log the raw fixed-point intermediate and finalized integer in diagnostic builds;
- keep UI formatting outside the core;
- add captured golden cases before claiming exact last-digit parity.

Do not rely on Rust's implicit casts, floating-point conversion, or arithmetic operators that hide overflow and rounding behavior.
