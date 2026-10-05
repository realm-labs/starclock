# Parallel Universe Walkie-Talkie authoring boundary

Production `WeightedCurioOverflows` privately lowers one current Walkie-Talkie
definition. Canonical ratios `10`, `1`, `0.8` mean ten times base DMG, one times
overflow and an 80% ATK increase. Eligibility is Erudition/Hunt, with no element
restriction. Normal equipment admits both contributions as independently
replaceable `VersionedProjectPolicy` behavior, not exact hidden-formula parity
or terminal execution credit. The current Weighted Curio equipment-definition
count is 15/17, including the separately policy-bound Footstep and Deflagration contributions.

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

## Native equipment policy

Production status is `NativeProjectPolicyHitEnded`. Source 133 explicitly has
`ProjectPolicy` quality and binds the runtime policy note's SHA-256, not an
upstream blob. Its replacement condition separately records unavailable callback
ordering, ATK timing/stacking, historical mark snapshots and deathrattle
correspondence, alternatives, low confidence and replacement evidence.

Accepted caller-selected equipment goes through the existing Activity boundary.
Normal contribution snapshots admit the definition; immutable battle assembly
combines the original entry-form Path proof, base-ATK buff and authored
level/Protocol death-conversion policy. No live Activity query, mode state
machine or shared content-ID branch is added. Unloading the equipment constructs
a new battle without either contribution; it does not mutate a live battle.

The both-family equipment test starts unmodified production materializations
and retains their mode bindings in controlled native command probes. It checks
Hunt/Erudition grants, Harmony/Preservation exclusion, 100-to-180 ATK queries,
250 damage killing a 100-HP enemy, selected level-1 base conversion of
`10 * 80 + 150 = 950`, labeled singleton RNG, fresh event payloads/hashes and
unequipped zero-conversion/zero-draw behavior. Assembly preserves Activity bytes
and its debug/RNG view. Forge offers, separate delayed-deathrattle/player-rescue
interactions and complete-run release remain outside this admission proof.

## Native construction contract

### Executable death-conversion bridge

`weighted_curio_overflow::bind_death_conversion_policy` is an explicit catalog
construction API. It takes a production-lowered definition, one caller-proven
eligible original roster member and an immutable base-DMG expression/identity.
The normal Activity battle assembly calls it together with the separate ATK
constructor after original entry-form Path admission. The bridge itself does not
implement the ATK modifier.
The separate `OverflowBaseDamagePolicy::from_authored` constructor now compiles
the reviewed HPRatio/Protocol policy described below; `new` still accepts an
explicit caller-provided expression, which is not factual evidence.

The bridge uses a replaceable `ProjectPolicy` for callback correspondence:

- Only original actor/applier `SourceClass::Ability` attacks collect. One private
  rule instance, private target mark and NeutralState readiness effect per roster formation prevent another
  original or an inherited form from sharing its accumulator or target pool.
  The exact cause actor must be a Unit, not a timeline actor resolved through
  its owner. Original roster admission follows Present or Transformed presence
  through a stable selector union; it does not compare the current unit form
  with the entry form. Reserved, Departed, Untargetable and Linked presence do
  not qualify. Owned linked units remain excluded even if they copy the same
  form, source, rule bundle and roster formation.
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
them. Player-only lethal rescue and delayed deathrattle still need
consumer-specific fixtures. Nested same-owner queued attacks now have native
consumer coverage described below. Phase resets and enemy damage
guards now have native consumer coverage described below. Linked exclusion is covered by
actual summon, memosprite, shared-actor and countdown commands, not merely the
copied-form fixture. The linked-unit vectors use both formation 4 and the
original's formation 0, proving that the owner-link exclusion is not redundant
with the roster-slot filter.

Eight native tests load the actual Sora definition and use a clearly labeled
fixture-only base `selected_enemy_level * explicit_factor`, not a production
formula. They verify multiple deaths before one conversion, a higher-HP
unattacked enemy remaining untouched, all-tie/empty/unready pools, singleton and
tie draw counts, shields and ShieldOverflowOnce, converted kill credit without
self-recursion, multi-hit reset, inherited-form exclusion, rejected decisions,
fresh canonical events/hashes, policy-bound participant state and the current seed 1..8 vector
`[5, 4, 5, 4, 4, 4, 4, 4]`. Run
`cargo test -p starclock-mode-universe weighted_curio_overflow`.

Three lifecycle tests additionally exercise a real original transformation,
fresh canonical events/hashes, restoration and a subsequent conversion ending
the battle; genuine inherited summons, memosprites and queued shared actors;
and a unitless countdown with its original owner still Transformed. Each linked
attack executes and kills enemies but produces no conversion, extra draw or
residual bridge effects. Resolving the countdown as its owner would wrongly
permit a 1,320 conversion; the raw actor-kind filter excludes that fallback
without changing shared legacy actor-selector resolution. These tests still
use the explicit fixture base, not a production HPRatio policy.

Four survival-boundary tests additionally construct real authored enemy phases,
persistent HP floors and one-use team defeat guards. A phase reset retains
positive event overflow (`150/170`) but emits no confirmed defeat, conversion
or random draw. The final phase can then really die on a subsequent attack;
fresh event/hash reconstruction and stale-command rejection prove the earlier
phase overflow does not carry into that attack. A mixed phase reset and actual
death in one hit converts only `170`, not `150 + 170`, giving `2,070` rather
than `2,220` under the fixture base. A 25% HP floor leaves both protected enemies
alive with zero overflow/readiness; a one-use team guard preserves the first
enemy at 1 HP, while the second enemy's actual death still converts its own
`170`. All cases retain the native post-guard damage facts and clear bridge
marks/readiness without removing unrelated persistent floors. These tests do
not establish hidden callback parity, player-rescue correspondence or equipment
admission.

Two queued-action tests now cover actor-local state across real same-owner
FollowUp envelopes. One queues after the first HitEnded, once per battle,
at AfterHit or AfterAction through native Rule IR. The between-hit case really
starts and resolves its child before the outer action resolves; the deferred
case starts the child after the outer action resolves. Both orders confirm
separate 100 and 150 excess values, giving 1,100 and 1,150 under the explicitly
fixture-only `selected_enemy_level * 2` base, rather than a combined 1,250.
Conversion retains the correct inner/outer ActionId and original owner in each
order. Both draw twice and finish with no marks/readiness. Fresh event payloads
and hashes match, and a stale command changes neither hash nor draws.

The second test queues a single-target child during a multi-target outer action.
The child kills its only target with 100 excess, but cannot convert into the
living enemy marked only by the suspended outer action. It draws zero times;
the resumed outer hit reopens observation and records no unrelated death or
conversion. This verifies the declared reset policy, not a source action stack,
delayed deathrattle callback, normal equipment or complete-run admission.

These explicit-base bridge tests are not complete-run release evidence. Normal
equipment has its separate both-family construction/command proof above.
No obligation/program/family/gap/policy terminal disposition changes.

### Original-roster ATK policy

`weighted_curio_overflow::attack_increase::bind_attack_increase_policy` now
constructs the separate 80% base-ATK contribution. Eligibility comes from the
immutable entry form's actual build-catalog Path: Hunt/Erudition qualify, while
all other Paths return the unchanged participant. Unknown forms, non-player
participants, formations outside 0..=3 and altered exact ratios reject before
construction. The source identity binds the exact character-definition digest,
authored ratio/key and caller's assembly identity.

One BattleStarted/AfterEvent trigger applies a non-dispellable permanent Buff
to the original unit only. Its ATK modifier uses Dynamic/PercentOfBase and
UniquePerSource, not an inheritable static participant modifier or a
damage-producer filter. The original's grant persists through transformations
and waves. Owned linked units are excluded even when their form, sources,
bundles and formation copy the original. Borrowing the original's actual ATK
remains governed by the shared stat-query contract; this is not an independent
grant to a borrower. BattleWon/Lost explicitly remove the effect using an
all-presence original-owner selector. No new resolver branch or live Activity
read exists. Battle-start correspondence, original-roster scoping and generic
ATK stacking are replaceable native policies, not proven hidden callback parity.

Actual commands query 100 base ATK as 180 for Hunt/Erudition and 100 for all
seven other Paths. They verify rejected-command hash/RNG stability, fresh event
payloads/hashes, transformation and restoration, and real summon/memosprite/
shared-actor attacks at formations 0 and 4 that still read their own ATK as 100.
Victory and concede remove the effect. These tests exercise the explicit
construction API; the separate normal equipment proof is described above. They
do not establish Forge or complete-run admission.

### Base damage policy

Production now authors 95 exact Group 1 HPRatio rows, with canonical decimal
strings and level-specific stable keys, in `WeightedCurioOverflowLevels`.
Source 131 pins `ExcelOutput/HardLevelGroup.json` at the same released revision,
SHA-256 `d185c09b5388f4eeb368199276ee8a815b406fd9902744027b72a5011b962978`,
accessed 2026-10-05. Levels 1/40/80/95 retain `0.8/9.524581/148.01102/294.42172`
exactly; no quantization or Excel/Python/JavaScript float transport occurs.

`EnemyGroupOneHpRatioProtocolHpMultiplierFloor` is a separately authored
`VersionedProjectPolicy`: multiply the factual fixed operand `100` by Group 1's
HPRatio at the selected enemy's own level, then by `1 +` the immutable battle
Protocol maximum-HP increase. Checked multiplication floors to six fractional
places at each boundary; final native TrueDamage floors to integral damage.
Choosing Group 1, interpreting the hidden query as Protocol HP scaling, operator
order and rounding remain low-confidence policy. None is decoded postfix or
observed parity. Alternatives and independent replacement conditions live beside
the policy. Source 132 has explicit `ProjectPolicy` quality: its URL/revision
identify released context, its locator identifies this local contract, and its
SHA-256 binds the authored policy note, not an upstream Git blob.

The loader validates exact-once 1..95 identity, Curio/group/source joins,
positive canonical millionths and distinct factual/policy provenance. The source
verifier independently matches every decimal against the pinned Git blob.
`OverflowBaseDamagePolicy::from_authored` now lowers the complete curve into
native Rule IR: a balanced selection tree performs at most seven comparisons
against `CurrentTarget`'s own checked level. The two checked Scalar multiplications
explicitly floor to six fractional places. Construction preflights every level
and rejects incomplete/nonpositive curves, invalid policy metadata, negative
Protocol scaling or overflow. The canonical identity binds all 95 ratios,
fixed/group/policy fields, notes/replacement condition and the full immutable
Protocol snapshot. No live Activity query or new shared resolver branch exists.

Native command tests execute all 95 levels with both-family no-Protocol
snapshots and real Protocol 1/2 snapshots, comparing an independent integer
oracle including final integral TrueDamage. The selected target's level varies
independently of the fixed original-owner and defeated-target levels.
Representative levels 1/40/80/95
reconstruct fresh events, canonical payloads and hashes. Changes to an unselected
level, policy metadata or Protocol alter the initial battle identity; malformed
definitions and arithmetic overflow reject construction. Signed Protocol-factor
boundary vectors cover zero, one millionth, negative input and checked overflow.

This is executable authored-policy lowering, not observed formula parity.
Normal battle assembly uses it with the ATK and callback policies; existing
callback/survival fixtures continue using their explicit test base. Equipment
admission contributes to the current 15/17 equipment-definition count, not
terminal dispositions.

### Remaining parity and release boundaries

The base-DMG expression reads a hard-level `HPRatio`, an opaque difficulty query
and unresolved postfix operands. No constant, owner ATK or zero is substituted
as exact base DMG. Exact postfix semantics and callback correspondence need
released typed evidence or a separately reviewed replaceable
`VersionedProjectPolicy` preserving the known level/Protocol constraints.

The production consumer uses explicit replaceable policies for hit/attack timing,
accumulator ownership, death correspondence, eligibility and marked-target lifetime.
Separate player-rescue and delayed-deathrattle fixtures remain required. A death
confirmed only after ActionResolved has no active collector and does not replay
earlier excess. Native HitEnded conversion cannot be called exact source parity
merely because its command probes pass. Existing
[DamageOverflow](rule-event-observation-runtime-boundary.md) is only a shared
settlement prerequisite, not the entire source mechanism.

Native `QueryUnitLevel` now exposes each resolved target's own checked level
through immutable Rule IR reads and current-state selector expressions. Actual
commands prove 1/81/95-level damage and a committed summon with a level distinct
from its owner. This removes the level-read capability gap, but does not decode
the source HPRatio/postfix program. See the
[unit-level query contract](rule-event-observation-runtime-boundary.md#unit-level-query).

## Verification

`cargo test -p starclock-data weighted_curio_overflow` loads the actual Sora
bundle and tests exact ratios, stable reconstruction, native policy status,
missing/duplicate rows, eligibility and operand drift, numeric transport,
false-parity status, all-level row/precision rejection and forged evidence at
each of eight required sources.
`verify-weighted-curio-overflow-authoring.mjs --check-source` verifies exact Git
blob hashes, current joins and named callback/target structure independently.
The normal decision-workbook verifier regenerates schema/readers/exports and a
fresh openpyxl workbook. None of these data checks proves battle execution.

Forge offers, accepted equipment replay, complete-run release and original
obligation/program/family/gap/policy terminal dispositions remain unchanged.
