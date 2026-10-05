# Footstep of Gods native HP-loss policy

The explicit constructor compiles one real HP-loss contribution through shared
Rule IR, battle commands, slots and resource operations. It is not normal Curio
equipment admission. The Skill-use damage-stack clause, production Sora operand
authoring, Forge and complete-run equipment replay remain pending. The current
Weighted Curio equipment count stays **13/17** and no terminal reference or
mechanic-program credit follows.

## Released evidence

Pinned released revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[TurnBasedGameData](https://gitlab.com/Dimbreath/turnbasedgamedata), game version
4.4, was inspected on 2026-10-05. `RogueTournHex` HexID 1011, Tourn3, joins
MazeBuff 633411 and original `Warrior`/`Memory` Path eligibility. MazeBuff level
one joins `StageAbility_633411`, description hash `10599000866283908992`, and
canonical operands `0.5`, `0.08`, `10`. Independent summaries of the two text
maps: accumulate loss equal to half maximum HP for one Skill Point; each Skill
use adds an 8% damage bonus, up to ten stacks. 毁灭／记忆角色累计降低生命值获得战技点，
释放战技叠加伤害增益。 These are factual operands, not native timing parity.

| Pinned blob | SHA-256 |
|---|---|
| `ExcelOutput/RogueTournHex.json` | `51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455` |
| `ExcelOutput/MazeBuff.json` | `2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac` |
| `TextMap/TextMapEN.json` | `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789` |
| `TextMap/TextMapCHS.json` | `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147` |
| `Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json` | `5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47` |

The merged program's `Modifier_StageAbility_633411_Sub` has
`OnListenHPChange` and `OnAfterSkillUse`. The HP callback accumulates negative
owner changes through named LoseHP/TotalLoseHP/AvatarMaxHP/BoostBP values and
requests team Boost Point gain. Its dynamic postfix arithmetic and hidden
callback timing have not been proven equivalent to this native policy. Raw
programs stay in the source cache, not the repository's authored runtime surface.

## Independently replaceable execution policy

`HpLossPointPolicy` is a `VersionedProjectPolicy` construction boundary.
The caller supplies a validated `(0, 1]` fixed Scalar fraction and immutable
provenance/policy identity; there is no production default or workbook loader.
The native tests bind `0.5`, not a new shipping hardcoded Curio operand.

- Qualify the immutable original entry form through the core build catalog's
  Destruction/Remembrance Path. Other Paths return unchanged without adding
  definitions. Owner anchoring uses `First`, a Present/Transformed union and
  formation equality; owned linked units are excluded even with inherited
  rules, sources and forms. Current transformed forms do not requalify Paths.
- Observe negative effective `DamageApplied` and `HpChanged` facts AfterEvent,
  priority zero, once per event. Ordinary/True/DoT/Break damage shares the first
  point; explicit consumption uses the second. Shield-only damage, healing and
  maximum-HP clamps add no loss. Lethal loss counts while the original remains
  Present/Transformed. This does not invent a universal HP-change event.
- Each original has an independent Battle Scalar slot initialized to zero.
  Store absolute HP residue, not a percentage. On each loss, compute the live
  resource capacity times the fraction with six-place Floor; divide the sum of
  residue and actual loss by that threshold and Floor to an integral point
  count. Subtract the crossed thresholds and commit the residue before gain.
  All expressions use the same immutable trigger snapshot, not preceding
  uncommitted slot writes. Healing and waves retain residue; each loss is
  processed independently, including multiple losses inside one action.
- Consume thresholds regardless of the Skill Point cap. Shared resource
  operations report requested/effective/discarded amounts. Overflow points are
  not banked for a later cap opening. Zero points retains the shared zero balance
  event. Checked arithmetic or a request outside `u16` faults with rollback,
  not saturation. No RNG is consumed by the contribution.
- Preserve the existing cause actor/ancestry while the owner selector determines
  the receiving team. Do not relabel an enemy damager as the victim. Source
  identity binds the caller's policy identity, fraction, assembly, entry form's
  build digest and formation. Cross-Curio gain attribution is not source parity.
- Win/loss clears every bound remainder without an alive/presence gate. Fresh
  construction starts at zero and there is no Activity carry or live mutation.

Unavailable facts: callback order, fractional thresholds, capacity-clamp
inclusion, healing/resurrection/transformation interactions, gain attribution,
and overflow banking. Confidence is low for those fields. Alternatives include
entry-max versus live-max thresholds, percentage rather than absolute residue,
declaration versus settlement timing, including maximum clamps and owner-sourced
gain actors. The selected policy uses existing immutable event/query facts and
bounded operations without source-ID resolver branches. Replace each field with
a released executable binding or reproducible released observation independently.
The constructor does not claim the whole source program is decoded.

## Verification boundary

The native command corpus covers cumulative actual loss, healing intervals,
multiple events in one action, original Path exclusion, inherited unrelated
roster isolation, independent formation-one ownership, shields, consumption
floors, overflow disposal, live and fractional thresholds, lethal loss,
terminal reset, invalid fractions, policy identity, rollback, rejected commands,
fresh event payloads and canonical hashes. Linked actor creation, real
transformation, Break/DoT packet producers and wave transitions need separate
consumer fixtures before normal equipment admission; their selector/slot design
is not proof of those scenarios. No fixture substitutes for the missing
Skill-use damage clause or formal workbook authoring.

```text
cargo test -p starclock-mode-universe weighted_curio_footstep
```
