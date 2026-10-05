# Footstep of Gods authored execution policies

Separate explicit constructors compile the HP-loss/point and after-Skill damage
contributions through shared Rule IR, battle commands, slots, effects and
resource/formula operations. Normal immutable equipment assembly now reads
their operands and independently replaceable policy identity from Sora.
The current Weighted Curio equipment-definition count is **14/17**, not a
complete-mechanic count. Real transformations, linked/unitless timeline
actors, periodic damage, dedicated damage channels and wave transitions now
have a bounded equipment consumer corpus in both run families.
Forge and complete-run equipment replay remain pending. No terminal reference
or mechanic-program credit follows.

## Production authoring and equipment boundary

`WeightedCurioFootsteps` authors canonical decimal strings `0.5` and `0.08`,
integer cap `10`, released Path membership and parameter locators. Sources
134–138 bind five pinned released blobs; Sources 139–140 are explicitly
`ProjectPolicy` and bind the corresponding HP/Skill policy-note bytes rather
than an upstream blob. Each clause has its own replacement condition. The
loader rejects missing/duplicate definitions, eligibility/operand drift,
false-parity policy labels and forged provenance. Runtime loads only the
generated binary Sora bundle, never the workbook or debug JSON.

Both Ordinary and Cyclical accepted loadouts lower the same authored clauses
through the existing constructors into immutable battle bundles. Configuration,
policy notes, source identities, build identity and original formation bind the
assembled contribution identity. No live Activity mutation or shared resolver
content branch is introduced. Equipment command probes retain those assembled
bindings without rebinding a policy: Destruction/Remembrance originals return
one point from two 30-HP losses separated by healing at 100 maximum HP, retain
10 residue and acquire three Skill stacks, yielding 124 rather than 100 Basic
damage. Harmony/Hunt originals receive neither contribution. Tests also cover
stale loadout/battle rejection, independent owners, teardown, unequip, fresh
canonical events/hashes, no RNG and unmodified production start/concede.
These bounded handoffs are not full-run or missing lifecycle-consumer proof.

Both run families additionally execute genuine reversible transformations on
the original unit: eligible Destruction/Remembrance originals become Hunt,
and the Hunt original becomes Destruction. Neither current-form change
requalifies the immutable entry Path. HP residue and Skill stacks survive
transformation/restoration and still affect actual resource gains and damage.

Actual Summon, Memosprite and SharedActor units inherit the original form,
bundles, sources and modifier bindings. A queued, explicitly tagged Assist
Skill consumes their own HP and deals damage; it cannot earn the original's
points or stacks. Even force-applying the genuine ten-stack effect leaves
linked damage at 100 rather than 180. Both colliding and distinct formation
indices are tested. A transformed original's genuinely unitless countdown
likewise deals 100 rather than borrowing the owner's bonus. These command
fixtures cover stale rejection, terminal cleanup, no RNG and fresh canonical
events/hashes; they are not claims about a released character's transformation
or Assist program. Forge and full-run acceptance remain pending.

Real target-turn DoT ticks retain an original applier distinct from the ticking
target. Applying a DoT before a Skill yields 100 then 108 for eligible originals,
showing live stack reads; Hunt remains at 100. Two incoming 20-HP ticks or actual
25-HP initial Break plus 15-HP Super Break combine with retained 30-HP residue
to grant exactly one point and leave 20 residue only for eligible victims.
The noneligible damager's attribution stays intact. Real Direct/Additional hits
increase from 100 to 124 at three stacks, while dedicated Elation remains 100
and initial Break/Super Break remain 25/15. These are policy reach checks, not
released calculator parity.

A real two-wave encounter emits wave-end/start facts and retains 30 HP residue
and two Skill layers. Loss in the second wave returns one point, retains 10
residue and raises the original to three layers; damage is still 124. Winning
the last wave clears both clauses. The corpus retains production rules and
source/modifier bindings, rejected-command inertness, no RNG and fresh canonical
events/hashes. These controlled encounter consumers do not substitute for Forge
or a complete Activity equipment replay.

The composition retains production Clara's bounded representative Counter
rule. Its [presence admission policy](representative-character-v1b-production.md#bounded-counter-admission)
does not fault or spend charges while its Present-only owner is transformed.
A separate transform-first fixture starts with two real charges, retains both
through damage while transformed, and resumes admission after restoration.
The unbound internal Counter is explicitly cancelled, never credited as an
executed Counter or complete Clara kit.

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
The Skill callback predicates `ByCurrentSkillType = Skill`, then adds
`Modifier_StageAbility_633411_Effect` to the modifier owner. The effect callback
updates `_Layer` and stacks `AllDamageTypeAddedRatio`. The engine's dynamic
postfix layer/property behavior and hidden callback ordering are not a decoded
native binding; the text's per-stack ratio/cap remain independently factual.

## Independently replaceable execution policy

`HpLossPointPolicy` is a `VersionedProjectPolicy` construction boundary.
The caller supplies a validated `(0, 1]` fixed Scalar fraction and immutable
provenance/policy identity. Production supplies both from its validated Sora
definition; explicit callers can still bind other validated policy inputs.
The native tests bind `0.5`, not a shipping hardcoded Curio default.

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
fresh event payloads and canonical hashes. Production equipment fixtures above
add actual linked actor creation, real transformation, target-turn periodic
DoTs, initial Break/Super Break loss and wave transitions. Their proof comes
from accepted commands and actual packets, not selector/slot declarations.
The bounded production handoffs do not substitute for Forge or full-run
equipment construction.

## After-Skill damage policy

`SkillDamagePolicy` separately binds a positive additive fixed Scalar ratio,
nonzero `u16` cap and immutable caller identity. The capped bonus product must
fit checked Scalar before catalog mutation. Tests supply the released `0.08`
and `10`; production reads those values from validated Sora, without a shipping
default or content-ID resolver branch. Source identity includes both operands, caller identity, assembly,
immutable entry-form build digest and formation.

- Use the same immutable Destruction/Remembrance original-Path proof and
  Present/Transformed formation anchoring as the HP policy. Other Paths append
  no contribution. Inherited rules on unrelated originals cannot acquire the
  effect. The policies have distinct typed source/rule/effect identities and
  can compose on the same participant without a second state machine.
- Observe `ActionResolved`, `AfterAction`, priority zero, with an explicit
  Skill/action-bearing original actor filter and `OnceScope::Action`. A
  complete Skill adds one effect stack, even without damage. Multiple targets,
  hits or HP facts cannot multiply it; other action kinds do not add stacks.
  Its already-resolved damage cannot receive the new layer retroactively.
- Use a permanent non-dispellable Buff with `RefreshAndAddStacks`, capped by
  the explicit operand, and `PersistByScope` teardown. Each original has an
  independent holder and effect definition. Dynamic modifier-local stack
  slots multiply the ratio with six-place Floor and the shared final damage
  floor remains authoritative. Explicit removal deletes its modifier captures;
  later Skills create a fresh one-stack instance.
- Add the ratio only at `DamageBoost` for Ordinary, DoT and Additional
  formula purposes. `DamageProducer::OriginalUnit` rejects linked/unitless
  formula-owner fallback. No ATK base is changed, and no True, Break, Super Break
  or dedicated Elation damage alias is introduced. This channel choice is
  policy, not proof that the source property has identical calculator reach.
- Waves retain the permanent effect. Win/loss uses an unconditional direct-owner
  cleanup selector, including defeated/absent holders; fresh construction has
  no stack. Checked errors use normal command rollback and no RNG is consumed.

Unavailable fields are exact callback order, same-Skill eligibility, bonus
calculator reach, linked/statistic fallback, and removal/transformation/wave
retention. They are low-confidence independently replaceable execution fields.
Alternatives include declaration-time stacking, same-action snapshots, different
damage-purpose sets, shared/linked recipients and reset-on-transformation/wave.
This policy selects the shared complete-action envelope, explicit original
ownership and ordinary additive formula blocks. Replace a field when released
executable evidence or reproducible observations prove its exact behavior.

The Skill command corpus exercises three-hit versus one-envelope accounting,
the cap, nondamaging Skills, composition with two HP losses in one action, all
original Paths, inherited and formation-one isolation, actual common-channel
damage changes, True exclusion, explicit removal/reapplication, terminal
cleanup, invalid operands, policy hashes, stale commands, checked formula
rollback retaining prior stacks/HP and fresh canonical
events/hashes. A Direct/DoT/Additional class vector checks the ordinary
calculator's named channels; the DoT-class vector is not a periodic effect
producer fixture. Separate production-equipment consumers above execute real
target-turn DoTs, initial Break/Super Break, dedicated Elation, linked/timeline
actors, transformations and wave transitions. Forge and complete-run equipment
replay remain independent pending acceptance boundaries.

```text
cargo test -p starclock-mode-universe weighted_curio_footstep
cargo test -p starclock-data weighted_curio_footstep
.cache/tools/node-v24.15.0-win-x64/node.exe tools/divergent-universe-runtime/verify-weighted-curio-footstep-authoring.mjs --check-source
```
