# Divergent Universe Road of Prayers

## Released operands

Production `WeightedCurioPrayers` records Road of Prayers / 祈祷之路.
Sources 89–92 pin released Version 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-10-01, with exact file digests and row locators.
Hex 1004 / Tourn3 / Display 1022 joins MazeBuff 633404 / Lv 1;
Mage / Erudition and Warlock / Nihility, without an element filter.
Canonical parameters `0.6`, `0.15`, `0.25` specify increased maximum HP,
owner-turn current-HP consumption, then a maximum-HP shield. No duration is
provided. The before-birth `StageAbility_633404` binding and displayed terms
do not establish hidden capacity, stacking, floor or lifecycle behavior.

## Executable policy

`VersionedProjectPolicyEntryHpAndTurnStartConsumeShield`:

- Fresh immutable battle assembly adds floor(60% of resolved entry HP capacity)
  to eligible mapped player participants' actual maximum HP. It adds no HP-stat
  modifier and preserves locked builds and unrelated fields. Linked actors and
  summons do not inherit the binding automatically.
- Explicit first-entry HP remains explicit; otherwise shared battle creation
  starts at the new capacity. Later entries use declared Activity carry,
  [rebound to destination capacities](activity-battle-entry-carry.md), without
  extra healing. Unequip only affects a later assembled battle.
- An alive/present owner `TurnStarted` reaction runs `AfterEvent`, priority zero,
  once per owner turn. It consumes floor(15% of current HP), preserving one HP,
  removes this source's previous shield/effect, then applies one non-dispellable
  permanent-clock effect and shield. Battle teardown removes the effect.
- Shield capacity is floor(25% of the live HP resource maximum), before shared
  shield modifiers. The generic `QueryMaximumHp` reads actual current capacity,
  not effective HP stat or immutable initial maximum. Expressions read the
  triggering root query snapshot before queued operations; ordered shared
  operations emit the consumption and replacement events.
- Attacks, multi-hit actions and Ultimate interrupts do not independently
  trigger this owner-turn effect. Replacement does not stack shield capacities.

Entry-capacity basis rather than base-character-HP addition, entry filling,
phase, one-HP floor, snapshot timing, refresh and duration are low-confidence
project policy, not observed gameplay parity. Alternatives include observed
base-stat stacking, retained entry HP, different turn phases, lethal consumption,
application-time queries and finite target-turn lifetimes. The selected policy
preserves all three released terms using existing bounded shared operations.
Replace each uncertain field when released programs or reproducible current
traces establish it; the workbook records provenance and replacement conditions.

## Verification and remaining scope

Native production-backed tests cover both families and both eligible Paths,
real maximum-HP capacity, current-HP consumption and one-HP boundaries,
ordered shield replacement, once-turn behavior, deterministic reconstruction,
rejected commands, unequip and verified battle-result maxima. A controlled
shared-operation fixture changes live capacity and independently modifies HP
stat to distinguish all three HP quantities. Data tests reject seasonal joins,
changed parameters, noncanonical decimals and forged provenance.

Original Forge admission, equipment-command replay, other Weighted Curios,
Grand Miracles and complete 13/17/20-position runs remain pending. This
component adds no original obligation/program/family/gap/policy terminal credit.
See [current repository state](state.md) for totals.
