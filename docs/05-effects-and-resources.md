# Effects and Resources

## Effect instances

An effect instance should contain:

- stable instance and definition IDs;
- source, applier, and target IDs;
- classification: buff, debuff, control, DoT, mark, field/zone, or neutral state;
- tags and dispel category;
- stack count and stack limit;
- duration policy, remaining duration, and tick phase;
- refresh/replace/independent-stack policy;
- snapshot or dynamic stat bindings;
- modifiers, triggers, and granted abilities;
- application sequence for deterministic conflict resolution.

Do not encode every status as an enum variant with bespoke engine code. Universal behaviors should be composable data; genuinely unique mechanics may use a registered native handler behind a stable interface.

## Application chance

For a debuff with a stated base chance:

```text
real_chance =
    base_chance
  * (1 + attacker_effect_hit_rate)
  * (1 - target_effect_res)
  * (1 - target_specific_debuff_res)
```

This is documented by the [Effect Hit Rate reference](https://honkai-star-rail.fandom.com/wiki/Effect_Hit_Rate) and in-game terminology transcriptions. Clamp the final comparison probability to `[0, 1]`; retain the pre-clamp value for diagnostics.

`target_specific_debuff_res` is resistance to a category such as Crowd Control or a named status. It is a separate multiplicative factor from general Effect RES.
Effect definitions name the applicable typed resistance stat, such as
`FreezeResistance`. A rule may explicitly ignore only that specific channel
while still applying general Effect RES; this is distinct from fixed chance.
Both resistance channels are signed ratios at this formula boundary. A
negative value means an authored resistance reduction and therefore increases
the corresponding `(1 - resistance)` multiplier. The authoritative supported
domain is `[-1, 1]`; the final probability, rather than either resistance
input, is clamped to `[0, 1]`.

Fixed chance is different: it ignores Effect Hit Rate and resistances unless an authored rule explicitly modifies it. Guaranteed/unconditional application should bypass RNG rather than use an arbitrarily large base chance.

Use one RNG draw per independently worded application. A multi-hit ability that states one chance for the whole attack should not roll on every hit.

Negative-effect duration is also a derived application boundary. Its authored
base multiplier is `1.0`; active target modifiers may change it before the new
effect instance is created:

```text
resolved_duration = round(authored_duration * debuff_duration_multiplier)
```

This applies to dispellable debuffs and cleanseable control effects, including
both static and expression-backed effect definitions. Permanent and positive
effects bypass this multiplier. The resolved positive integer duration is
stored in authoritative effect state and therefore participates in hashes and
replays.

## Stack and refresh policies

Support these policies explicitly:

- `Replace`: newest instance replaces the old one;
- `Refresh`: keep magnitude/stacks, reset duration;
- `RefreshAndAddStacks`;
- `StrongestWins`: only strongest modifier is effective while instances remain independently tracked;
- `IndependentBySource`;
- `IndependentInstances`;
- `UniqueGlobal` or `UniquePerSource`.

Magnitude comparison needs a definition-specific comparator. Comparing a single numeric field is insufficient for effects with several modifiers.

Effect stack mutation is an authoritative operation, not a modifier-side
shortcut:

- `ApplyEffect` evaluates a positive integer stack expression and then applies
  the definition's stack/refresh policy and stack limit;
- `AdjustEffectStacks` applies one signed integer delta to every matching
  instance on each selected target, clamps each instance to its authored
  `[0, stack_limit]` interval, and removes an instance that reaches zero;
- a nonzero adjustment emits `EffectRefreshed` with exact before/after stacks,
  or `EffectRemoved` at zero, so subsequent triggers observe the committed
  mutation;
- `QueryEffectStacks` reads the integer sum of all matching instances on one
  subject from the immutable evaluation snapshot;
- `StackCount` is the post-event count while `StackDelta` is the signed
  before/after difference. An initial application has `StackDelta ==
  StackCount`.

Modifier attachments backed by effect stacks are refreshed in the same
transaction before the stack-change event is dispatched. A rule that reacts
to its own stack event must exclude its source definition or use another
explicit terminating condition; the reaction budget remains the final fault
boundary.

## DoT and control

Ordinary DoT normally resolves at the affected unit's turn start and does not CRIT. Break DoT uses the Break formula and duration rules in [Toughness and Break](04-toughness-and-break.md).

Control must prevent only the actions named by its category. For example, follow-up actions are documented as unavailable while their source is under Crowd Control, but passive modifiers and all trigger types should not automatically be disabled.

Treat cleanse and dispel as queries over effect tags and dispel rules. Some states are non-dispellable.

## Team Skill Points

Default team values:

```text
starting_skill_points = 3
maximum_skill_points = 5
```

Most Basic ATKs generate 1 and most Skills consume 1, but these are ability costs/results rather than universal hard-coded side effects. Enhanced basics, special Skills, and personal substitute resources are common exceptions. The defaults are documented on the [Skill Point reference](https://honkai-star-rail.fandom.com/wiki/Skill_Point).

Skill Points belong to the team, not a character. A command is legal only when all costs can be paid under its substitution priority. Emit both `ResourceSpent` and `ResourceGained`; clamp ordinary gain at the current cap and record overflow separately for effects that care about it.

## Energy and Ultimates

Energy is normally a per-unit resource with an authored maximum. Most Ultimates become legal at maximum and consume their defined cost, but partial-cost and non-Energy Ultimates exist and must be data-driven.

Common ability generation defaults are:

| Source | Base Energy |
|---|---:|
| Most Basic ATKs | 20 |
| Most Skills | 30 |
| Defeating an enemy | 10 |
| Being hit | Common authored values: 5, 10, 15, 20, or 25 |

These are defaults only; see the current [Energy reference](https://honkai-star-rail.fandom.com/wiki/Energy#Generating_Energy).

Affected Energy gain uses:

```text
real_energy_gain = base_energy_gain * energy_regeneration_rate
```

Represent `energy_regeneration_rate` with a base value of `1.0`. Authored
action-boundary Energy gains always query this stat with the committed action
tags, so tag-filtered modifiers apply before the `ActionResolved` fact.
Rule-IR resource operations opt in with `scales_with_regeneration`; validation
accepts that flag only for `Gain(Energy)`. The selected operation rounding
policy is applied once after multiplying the fixed-point base gain by the
rate. Fixed Energy effects set the flag to `false`. Fractional Energy remains
authoritative internally.

## Aggro and enemy targeting

For an enemy action that selects a target by aggro:

```text
unit_aggro = base_aggro * (1 + total_aggro_modifier)
target_probability = unit_aggro / sum(eligible_unit_aggro)
```

Community-derived path defaults are:

| Path/archetype | Base aggro |
|---|---:|
| Hunt | 3 |
| Erudition | 3 |
| Harmony | 4 |
| Nihility | 4 |
| Abundance | 4 |
| Remembrance | 4 |
| Elation | 4 |
| Destruction | 5 |
| Preservation | 6 |

The formula and values are explicitly labeled community-derived on the [Aggro reference](https://honkai-star-rail.fandom.com/wiki/Aggro), so keep them configurable rather than fundamental constants.

Targeting precedence:

1. valid forced target/Taunt;
2. valid lock-on or scripted target;
3. ability-specific deterministic priority;
4. weighted aggro draw;
5. uniform draw for Bounce attacks documented to ignore aggro.

Select only from alive, present, targetable units. Normalize after all modifiers; if total weight is non-positive, fall back to a documented uniform choice.

## HP consumption and damage

HP consumption is not automatically damage. It may bypass shields, not trigger `Damaged`, and obey a minimum-HP rule such as reducing to 1. Model it as a separate `HpChangeKind::Consumption` with authored floor and trigger tags.

Likewise, damage redirection, distribution, and recorded damage should be pipeline transformations, not negative healing.

## Shield effects

Each shield remains an effect instance with its own remaining capacity and expiry. On incoming shieldable damage:

1. identify all active eligible shield instances;
2. effective HP protection is determined by the largest applicable remaining shield under the ordinary non-stacking rule;
3. reduce every applicable shield instance by the incoming amount;
4. remove broken instances and fire their removal triggers;
5. apply only overflow beyond the largest effective shield to HP.

An authored stacking shield can replace this policy for its own group.

## Snapshot policy

Public documentation does not define one universal snapshot rule for all character DoTs, fields, shields, and delayed damage. Every effect definition should therefore declare which inputs are:

- captured on application;
- queried on each tick/trigger;
- captured from the source but queried from the target;
- recalculated when stacks change.

Start with dynamic queries unless a verified kit test requires snapshots. The declaration is more important than the initial default.
