# Goal 07 Elation Partition S02

`G07-P2-M08-S02` completes both released levels of The Hourglass
Kindergarten, The Painted Albatross, 12 Monkeys and Angry Men, Aiden
Gravitational Rainbow and Twenty-First Military Rule. It also completes the
base record and level 1 of Exemplary Conduct; its enhanced level is assigned
to the next frozen partition.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. The workbooks are authored with openpyxl, exported
through Sora 0.3.0, and checked by a focused partition golden. Runtime code
consumes validated domain definitions rather than workbook rows.

Released numeric values and callback structure were verified against
`Level_RogueBuff_Ability_4.json` from the pinned TurnBasedGameData cache.
Public wording was cross-checked against the
[Simulated Universe Paths reference](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Paths)
and the
[Elation Blessing catalog](https://gamewith.net/honkai-starrail/article/show/39685),
accessed 2026-07-26. The structured callback is decisive when shortened
display wording omits target selection, hit boundaries or formula timing.

## Follow-up eligibility

The rules in this partition observe native follow-up and counter tags. When
Champion's Dinner is selected, they also accept Ultimate tags. Champion's
Dinner does not rewrite the action kind: Ultimate interrupt timing, resource
payment, cause attribution and replay identity remain unchanged.

## The Hourglass Kindergarten

Every distinct element of Aftertaste damage applies an independently
refreshable ATK reduction to the damaged enemy:

```text
L1: -4% ATK per distinct Aftertaste element
L2: -6% ATK per distinct Aftertaste element
duration: through the affected target's next action end
```

Seven effect definitions retain Physical, Fire, Ice, Lightning, Wind,
Quantum and Imaginary identity. Repeated damage of one element refreshes its
effect; different elements add through the ordinary percent-of-base ATK
stage. The rule does not count raw hit instances.

## The Painted Albatross

After a follow-up attack resolves, every enemy in its committed hit-target
list contributes one complete Additional-DMG pass over that same list:

```text
L1: each pass = 24% of the attacker's ATK to every hit enemy
L2: each pass = 36% of the attacker's ATK to every hit enemy
number of passes = number of distinct committed hit enemies
```

For an attack that hits `N` enemies, the rule emits `N × N` sequential
Additional-DMG events. This preserves per-instance triggers and cause chains
instead of collapsing the result into one multiplied number. Damage inherits
the originating action's element and uses the ordinary Additional-DMG formula
once.

## 12 Monkeys and Angry Men

The released callback increments the ramp at `HitStarted`, before the current
hit is calculated:

```text
L1 hit k bonus: k × 4%
L2 hit k bonus: k × 6%
k starts at 1 for each follow-up attack
```

An action-end effect carries the current hit count. Its stack-backed
DamageBoost modifiers apply only to follow-up, counter and
Champion-enabled Ultimate damage. The effect expires at the complete action
boundary, so nested reactions and later actions cannot inherit the ramp.

## Aiden Gravitational Rainbow

After an eligible attack resolves, the released callback traverses every
enemy in the attack's committed target list in a stable randomized order:

```text
L1: delay every attacked enemy by 12%
L2: delay every attacked enemy by 12%, then roll Imprisonment independently
    for each at 10% base chance
Imprisonment: 1 target turn, -10% SPD, and an additional 20% action delay
```

The without-replacement target traversal starts from canonical event order
and uses a labeled RNG purpose. For each visited target, the 12% delay is
applied before that target's enhanced control roll. The additional delay
occurs only after the resistible control effect is successfully applied, so
Effect RES and replay draw accounting remain in the shared effect pipeline.

The released structured callback wraps `AttackTargetList` in `Retarget` with
`ByRandom = true`. The same construct wraps known all-team Abundance healing
and cleanse callbacks, establishing randomized iteration rather than
single-target choice. Starclock expresses that shared behavior through the
generic `RngUniform` selector with a complete, non-repeating target budget and
a `ForEach` body.

## Twenty-First Military Rule

After one complete eligible attack:

```text
L1: 65% fixed chance to recover 1 Skill Point
L2: 100% fixed chance to recover 1 Skill Point
maximum trigger frequency: once per action
```

The fixed roll is represented by a transient non-dispellable Rule IR effect.
Only successful application emits the checked team Skill Point mutation.
The effect expires at the same complete-action boundary and no native
handler is required.

## Exemplary Conduct level 1

The contribution compiler counts selected Elation Blessings in the immutable
battle snapshot:

```text
bonus = min(selected Elation Blessings, 6) × 9%
maximum level-1 bonus = 54%
```

The result is installed as ordinary tag-filtered DamageBoost modifiers for
follow-up attacks, counters and Champion-enabled Ultimates. The count is
resolved during materialization; battle resolution does not query run state
or workbook data.

## Production verification

Production tests prove:

- every selected S02 level materializes without a native handler;
- Hourglass owns seven distinct target-action effects with exact negative ATK
  values;
- 12 Monkeys increments before damage, reads effect stacks and resets at
  action end;
- Painted Albatross retains target-count repetition instead of folding it
  into one damage event;
- enhanced Aiden uses a stable committed-target draw, resistible control,
  exact SPD reduction and success-gated extra delay;
- Military Rule level 1 uses one 65% fixed roll per action and level 2
  restores exactly one team Skill Point;
- a production Kafka Ultimate against two enemies produces four Painted
  Albatross Additional-DMG events and executes the other Champion-enabled
  rules without changing the root Ultimate action.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M08-S02.json`.
