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

The base-DMG expression reads a hard-level `HPRatio`, an opaque difficulty query
and unresolved postfix operands. No constant, owner ATK or zero is substituted
as exact base DMG. Exact postfix semantics and callback correspondence need
released typed evidence or a separately reviewed replaceable
`VersionedProjectPolicy` preserving the known level/Protocol constraints.

The consumer still needs hit/attack observation timing, accumulator ownership,
death versus deathrattle mapping, phase/rescue/nonlethal exclusion, linked-actor
eligibility, marked-target lifetime, labeled tied-target RNG, last-kill recursion
limits, battle teardown and actual Ordinary/Cyclical command fixtures. An
ActionResolved-only approximation or per-hit conversion cannot be called exact
merely because a simple attack passes. Existing
[DamageOverflow](rule-event-observation-runtime-boundary.md) is only a shared
settlement prerequisite, not the entire source mechanism.

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
