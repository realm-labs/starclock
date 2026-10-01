# Divergent Universe The Story Presently

## Released operands

Production `WeightedCurioSupportAttacks` authors The Story Presently /
故事的现在时. Sources 85–88 pin released Version 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-10-01, with exact file hashes and field locators.

- Hex 1017 / Tourn3 / Display 1020 joins MazeBuff 633417, no element filter;
  Shaman / Harmony, Priest / Abundance and Knight / Preservation.
- Generic MazeBuff 633417 / Lv 1 parameters are canonical `0.15`, `0.3`, `1`,
  binding `StageAbility_633417` before character birth.
- EN/CHS hash `18107300873516931386`: each eligible teammate increases those
  characters' CRIT Rate by 15% and CRIT DMG by 30%; their attacks add damage
  equal to the attacker's Max HP, DEF and ATK sum.

The public [current-season cross-check](https://honkai-star-rail.fandom.com/wiki/The_Story_Presently)
separates Arcadian Chronicles from Human Comedy and Protean Hero variants.
It corroborates the displayed values, not hidden behavior. Older 25%/75% and
Remembrance variants are not admitted. No released executable program or
reproducible timing trace was established by the bounded search; a before-birth
binding does not establish when the attack reaction resolves.

## Executable policy

`VersionedProjectPolicyRosterCountCritAndAttackResolvedAdditional`:

- Count the immutable mapped player character roster, including down/absent
  characters, excluding separately created linked actors/summons. Attach only
  to eligible Paths, with no implicit summon inheritance.
- Two battle-lifetime flat Stat modifiers add count times the exact operands,
  in independent unique-per-source groups. Existing bonuses coexist. No extra
  cap is invented; shared crit sampling applies its normal probability bounds.
- Observe owner `ActionResolved` / `AfterAction`, `Attack` tag and action
  identity, once per action rather than hit. Use committed opposing targets
  still alive/present in stable formation/ID order. Allied, non-attack and dead
  targets do not receive additional damage. Multi-target attacks apply once to
  each survivor; an implicit full-team target needs no primary controller input.
- Dynamically query effective HP stat, DEF and ATK of the owner, not current HP
  or target stats. Multiply their sum by `1` with shared nearest-ties-even
  fixed-point arithmetic; shared damage applies its integral floor.
- Emit AdditionalDamage using the actor's canonical Basic element and shared
  damage modifiers. It cannot crit, can defeat, and does not reduce Toughness.
  No new action/recursive attack is generated. Shared operations own mutation.
- Run equipment contributes fresh immutable battle bindings. Locked builds,
  unrelated bindings, enemy specs and live Activity remain unchanged. Unequip
  affects later snapshots only; the battle owns teardown and replay state.

Roster life/presence counting, phase, target cardinality, stat snapshot,
element, critical eligibility and Toughness are low-confidence project policy,
not observed parity. Alternatives include live eligible-count updates, per-hit
or primary-only damage, application snapshots, observed attack elements and
crit-capable additional damage. The selected policy preserves all exact terms
and bounded existing shared execution. Replace each field independently when
released programs or reproducible current traces establish it.

## Verification and remaining scope

Production-backed native fixtures exercise both families, all three eligible
Paths, zero through four-member counts, actual queried crit stats and crit damage,
single/all-target multi-hit Basic/Skill attacks, Ultimate reactions, noncrit additional
damage, fractional rounding and composition with the real ordinary Curio final
damage modifier, defeat, negative admission, fresh deterministic reconstruction,
immutable build/Activity boundaries, unequip and real production battle handoff.
Data fixtures reject seasonal variants, bad joins/parameters/decimals and
unrelated provenance. Current static inventories are not test-pass receipts.

Four of 17 effects execute; thirteen remain fail-closed. The workbook now has
38 tables / 385 rows. Original Forge admission, equipment-command replay,
Grand Miracle effects and complete 13/17/20-position runs remain pending.
This component adds no original obligation/program/family/gap/policy terminal
credit and does not establish complete release readiness.
