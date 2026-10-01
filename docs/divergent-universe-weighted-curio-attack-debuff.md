# Divergent Universe Automated Experience

## Released operands

The production `WeightedCurioAttackDebuffs` sheet authors Automated Experience /
自动化体验. Sources 81–84 pin released Version 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-10-01. Exact file digests and row locators live in the Sources sheet.

- `RogueTournHex.json`: Hex 1014 / Tourn3, Display 1021, MazeBuff 633414,
  Warrior / Destruction and Warlock / Nihility, no element restriction.
- Generic `MazeBuff.json`, ID 633414 / Lv 1: canonical parameters `0.2`, `0.3`,
  `1`; binding `StageAbility_633414` before character birth.
- EN/CHS TextMap hash `4217722633224787979`: after a qualifying character's
  attack, attacked enemies advance 20% and deal 30% less damage for one turn.
  Before-birth binding does not establish attack trigger timing.

A bounded pinned filename search found no matching executable file; this is not
proof of program absence. Public cross-checks corroborate the displayed effect,
not hidden phase, modifier stage, stacking or dispel. Preview/leak and later
version results were not admitted. The normalized reference's unresolved
RogueMazeBuff locator does not invalidate the exact generic MazeBuff join.

## Executable policy

`VersionedProjectPolicyAttackResolvedAdvanceAndTargetTurnFinalReduction`:

- Bind only mapped player characters with immutable Destruction/Nihility Path.
  Observe owner `ActionResolved` / `AfterAction`, `Attack` tag and action identity;
  once per action, not damage hit. Action kind alone is not admission: tagged
  follow-ups/counters share this boundary; explicitly non-attack actions do not.
- Resolve committed opposing targets that remain alive/present, in stable
  formation/ID order. Each gets shared `AdvanceAction(0.2)`, including implicit
  full-team attack targets without a controller primary. Dead targets are skipped.
- Replace only this definition's old effect. Apply one guaranteed,
  non-dispellable Debuff for one `TargetTurnEnd`; an already-running target turn
  counts. The shared effect store owns modifier creation, refresh and removal.
  Cross-caster replacement does not accumulate multiplicative layers.
- Six source-owned final product factors `1 - 0.3 = 0.7` affect OrdinaryDamage,
  Dot, AdditionalDamage, ElationDamage, Break and SuperBreak. They are not ATK
  penalties or incoming damage reduction. Unrelated groups coexist. Dynamic
  fixed-point shared arithmetic uses nearest-ties-even then final damage floor.
  True damage, explicit damage overrides and deliberately unboosted operations
  retain their shared bypass behavior; no new bypass is added here.
- Run equipment contributes fresh immutable battle bindings. Locked builds,
  enemy specs and prior bindings stay intact. No resistance draw, RNG, recursive
  attack, implicit linked/new summon inheritance or live Activity mutation.
  Unequip changes later snapshots, not already constructed battles.

Hidden phase, committed/declaration targets, dead/absent filtering, formula
stage/category coverage, dispel, immediate turn tick and cross-caster refresh are
low-confidence **project policy, not observed parity**. Alternatives include
declaration-time triggers, source Weaken aggregation, break exemption,
dispellable debuffs and independent caster layers. The chosen policy preserves
both displayed effects and uses bounded existing shared operations. Replace
fields independently when released programs or reproducible current traces
establish them, preserving both Paths, attack/enemy direction and exact operands.

## Verification and remaining scope

Production-backed native fixtures cover both run families and both eligible
Paths, single/all targets, multi-hit once cardinality, refresh, explicit
non-attack and allied-action rejection, dead targets, actual enemy damage before
and after expiry, immutable Activity/build boundaries, fresh reconstruction,
unequip and a real production controller/BattleResult handoff. Data fixtures
reject malformed joins, Paths, parameters, decimals, duration and provenance.
Static inventories describe current code and test targets, not test-pass receipts.

See [current repository state](state.md) for workbook and effect totals;
unsupported equipment remains fail-closed. Original Forge offer/slot admission, equipment
command replay, Grand Miracle effects and complete 13/17/20-position runs remain
pending. This component adds no original obligation/program/family/gap/policy
terminal credit and does not prove the complete release goal.
