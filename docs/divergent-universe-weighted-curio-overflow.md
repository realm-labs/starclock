# Parallel Universe Walkie-Talkie authoring boundary

Production `WeightedCurioOverflows` privately lowers one current Walkie-Talkie
definition. Canonical ratios `10`, `1`, `0.8` mean ten times base DMG, one times
overflow and an 80% ATK increase. Eligibility is Erudition/Hunt, with no element
restriction. **Native admission remains pending**: a loaded definition is not
an equipped battle effect, an exact hidden formula or terminal execution credit.
The implemented Weighted Curio count remains 12/17.

## Released joins and program evidence

Sources 126–130 pin released Version 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-10-03. Hex 1016 / Tourn3 / Display 1007 joins MazeBuff 633416 / Lv 1;
description hash `12104045893670599658` binds all three parameter positions in
both released text maps. The production Sources sheet contains exact file
hashes, row locators and short original notes.

The executable source is inside
`Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json`,
SHA-256 `5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47`,
at `Name=StageAbility_633416`. Absence of an ID-named filename is **not** proof
that a merged program is absent. The authoring verifier inspects the ability
name and named task structure rather than inferring semantics from opaque bytes.

Named structure establishes character-create Path filtering, a Replace
AttackAddedRatio modifier, attack-start resets, hit-target marks, zero-current-HP
observation and `Result_OverflowHPDamage`. Separate death and deathrattle
callbacks choose the highest-current-HP marked target, select randomly among
ties and perform a TrueDamage/ByBaseDamage conversion. Conversion permits
last-kill behavior; accumulator and marks are then cleared. Attack-end clears
the attack-active flag. These are source facts, not proof that Starclock's
event order or existing damage fields reproduce them.

The current Arcadian Chronicles variant on the
[independent Curio page](https://honkai-star-rail.fandom.com/wiki/Parallel_Universe_Walkie-Talkie)
corroborates the ATK and overflow terms, including base-DMG dependence on enemy
level and Threshold Protocol. Historical variants are excluded. It does not
provide the hidden numeric formula or establish callback timing.

## Remaining native contract

### Executable death-conversion bridge

`weighted_curio_overflow::bind_death_conversion_policy` is an explicit catalog
construction API. It takes a production-lowered definition, one caller-proven
eligible original roster member and an immutable base-DMG expression/identity.
The normal Activity battle assembly path does **not** call it or admit the
pending effect. Existing caller-selected loadout storage may retain the identity;
the battle snapshot rejects it. It does not implement the ATK modifier or select a production
HPRatio/difficulty formula. Caller-provided expressions are not factual evidence.

The bridge uses a replaceable `ProjectPolicy` for callback correspondence:

- Only original actor/applier `SourceClass::Ability` attacks collect. One private
  rule instance, private target mark and NeutralState readiness effect per roster formation prevent another
  original or an inherited form from sharing its accumulator or target pool.
- ActionStarted clears accumulated overflow, death confirmation and old marks.
  HitStarted marks the committed opposing living targets and enables observation.
  Marks persist across hits until conversion or ActionResolved.
- DamageApplied adds the immutable exact `DamageOverflow` for a currently
  defeated marked target. A separate eligible UnitDefeated event with positive
  accumulated overflow confirms death and makes living attacked targets ready.
  Readiness is a NeutralState effect, not an extra gameplay mark or debuff.
  Current lifecycle state is an explicit policy read, not a historical snapshot.
- HitEnded performs one conversion after all settlements. Only living,
  present, attacked **and ready** enemies enter the current-HP maximum pool.
  All equal maxima remain eligible; registered `damage-target` uniform RNG picks
  one. A singleton draws once; an empty/unready pool draws zero times.
- The entire program reads one immutable snapshot. Ordered SetSlot emissions
  reset the accumulator/death flag before the captured old value becomes
  `10 * caller_base + 1 * total_overflow` TrueDamage. Mark and readiness are then removed,
  including when no living target remains. Converted kills keep native owner,
  actor, applier and action ancestry but their Mode source cannot recollect.
- ActionResolved, WaveEnded, BattleWon and BattleLost clear marks, readiness and slots.
  A nested same-owner attack start resets actor-local state; its end clears it;
  an outer resumed HitStarted enables observation again. This is not a new
  action stack or proven source reentrant/deathrattle semantics.

Readiness is mechanically necessary: shared dispatch eagerly resolves declared
program selectors before trigger conditions. Referencing a random selector in
every program or relying only on a false condition would consume unrelated
draws. Each program declares only its actual selectors/filter dependencies;
the conversion selector also requires readiness before choosing.

The selected policy uses existing shared events, operations, scopes, checked
arithmetic and budgets. Alternatives were immediate conversion at the first
death (which misses later settlements in the same operation), end-of-attack
only conversion (which delays every conversion), or a separate callback/state
machine (which violates shared ownership). Confidence in **source parity is
low**; confidence in the specified bridge behavior comes from native commands.
Replace callback timing, reentrancy, marked-target lifetime and attribution
individually when released typed semantics or reproducible observations establish
them. Phase transitions, lethal rescue, genuine linked actors, delayed
deathrattle and nested queued attacks still need consumer-specific fixtures;
they are not credited by ordinary hit tests or the inherited-form fixture.

Eight native tests load the actual Sora definition and use a clearly labeled
fixture-only base `selected_enemy_level * explicit_factor`, not a production
formula. They verify multiple deaths before one conversion, a higher-HP
unattacked enemy remaining untouched, all-tie/empty/unready pools, singleton and
tie draw counts, shields and ShieldOverflowOnce, converted kill credit without
self-recursion, multi-hit reset, inherited-form exclusion, rejected decisions,
fresh canonical events/hashes, policy-bound participant state and the current seed 1..8 vector
`[5, 4, 5, 4, 4, 4, 4, 4]`. Run
`cargo test -p starclock-mode-universe weighted_curio_overflow`.

These are executable bridge tests, **not** Ordinary/Cyclical equipment or
complete-run release evidence. Production promotion still requires the authored
base policy, ATK modifier, eligibility/teardown coverage and actual both-family
battle construction and accepted-command fixtures. The count remains 12/17;
no obligation/program/family/gap/policy terminal disposition changes.

### Production admission still pending

The base-DMG expression reads a hard-level `HPRatio`, an opaque difficulty query
and unresolved postfix operands. No constant, owner ATK or zero is substituted
as exact base DMG. Exact postfix semantics and callback correspondence need
released typed evidence or a separately reviewed replaceable
`VersionedProjectPolicy` preserving the known level/Protocol constraints.

The production consumer still needs reviewed hit/attack observation timing, accumulator ownership,
death versus deathrattle mapping, phase/rescue/nonlethal exclusion, linked-actor
eligibility, marked-target lifetime, labeled tied-target RNG, last-kill recursion
limits, battle teardown and actual Ordinary/Cyclical command fixtures. An
ActionResolved-only approximation or per-hit conversion cannot be called exact
merely because a simple attack passes. Existing
[DamageOverflow](rule-event-observation-runtime-boundary.md) is only a shared
settlement prerequisite, not the entire source mechanism.

Native `QueryUnitLevel` now exposes each resolved target's own checked level
through immutable Rule IR reads and current-state selector expressions. Actual
commands prove 1/81/95-level damage and a committed summon with a level distinct
from its owner. This removes the level-read capability gap, but does not decode
the source HPRatio/postfix program, select a base-DMG policy or implement the
death conversion. See the
[unit-level query contract](rule-event-observation-runtime-boundary.md#unit-level-query).

## Verification

`cargo test -p starclock-data weighted_curio_overflow` loads the actual Sora
bundle and tests exact ratios, stable reconstruction, pending status,
missing/duplicate rows, eligibility and operand drift, numeric transport,
false-parity status and forged evidence at each of five required sources.
`verify-weighted-curio-overflow-authoring.mjs --check-source` verifies exact Git
blob hashes, current joins and named callback/target structure independently.
The normal decision-workbook verifier regenerates schema/readers/exports and a
fresh openpyxl workbook. None of these data checks proves battle execution.

Forge offers, accepted equipment replay, complete-run release and original
obligation/program/family/gap/policy terminal dispositions remain unchanged.
