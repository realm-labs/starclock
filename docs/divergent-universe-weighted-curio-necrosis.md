# Mock Crimson Moon

## Current production boundary

`divergent-universe.weighted-curio.1003` is a current Tourn3 Weighted Curio,
not a Grand Miracle. Accepted equipment contributes a complete, source-owned
Necrosis program through production Excel/Sora lowering and shared Rule IR.
This does not implement its Forge offer, equipment-command replay, the other
eight remaining equipment effects or an original-game complete run. Reference obligations
and their pending family dispositions are unchanged.

## Released facts

On 2026-10-01, all 80 reference pack file digests were checked against the current
pack index `59fe8211da025d6526f08f9d0a8e1a9955ae60219ba8b82fa6cab9f1f2b544b6`.
The fixed released source revision is
`fd978d6ef09f941fba644c731ab54abd6f7c3568`.

- `ExcelOutput/RogueTournHex.json`: HexID 1003, Tourn3, Priest (Abundance),
  DisplayID 1016, MazeBuffID 633403; SHA-256
  `51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455`.
- `ExcelOutput/MazeBuff.json`: ID 633403, Lv 1, ordered operands `1.5`, `6`,
  `3`, `2`, binding `StageAbilityBeforeCharacterBorn` / `StageAbility_633403`;
  SHA-256 `2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac`.
- `TextMap/TextMapEN.json`, description hash `14457895563947008111`;
  SHA-256 `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789`.
- `TextMap/TextMapCHS.json`, the same description hash;
  SHA-256 `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147`.

The independent current [BWIKI entry, revision 81596](https://wiki.biligame.com/sr/index.php?title=%E6%8B%9F%E8%B5%A4%E6%9C%88&oldid=81596)
and [English entry](https://honkai-star-rail.fandom.com/wiki/Mock_Crimson_Moon),
accessed 2026-10-01, agree on 150% base chance, six-times-ATK Fire DoT,
three turns and two-times damage from other Burns. Older four-times-ATK text
is not used. No released executable `StageAbility_633403` was found in the
pinned source tree; these descriptions do not establish hidden trigger timing.
Source rows 103–106 retain exact provenance; short summaries are independently
authored rather than copied descriptions.

## Replaceable execution policy

The named accuracy is
`VersionedProjectPolicyAttackResolvedNecrosisAndDotDamageBurnDetonation`.
Every choice below is low-confidence hidden behavior, not observed parity.

| Field | Selected deterministic behavior | Alternatives / replacement evidence |
|---|---|---|
| Attack observation | ActionResolved, AfterAction, priority zero, once per action; only original Abundance party owners | Per-hit scheduling or linked/transformed eligibility; replace with released stage program or current trace |
| Targets | Distinct alive/present opposing EventTargets, formation order, maximum 16; empty is no-op | Hit-by-hit targets or other retarget rules; replace with target trace |
| Chance | One resistible 1.5-base attempt per target; live Effect Hit Rate and Effect RES, no authored specific-resistance stat; labeled purpose 41001 | Different ordering or special immunity; replace with observed chance/immunity data |
| Snapshot | Capture six times effective owner's ATK on successful application, nearest-ties-even; dynamic target/formula context thereafter | Dynamic ATK or full target snapshot; replace with buff/debuff trace |
| Replacement | One stack; successful application replaces previous Necrosis on that target across casters and captures the new source | Source-independent coexistence or retained first source; replace with two-caster trace |
| Lifetime | Dispellable debuff, target TurnStart damage, TargetTurnEnd three-turn duration; shared owner/battle teardown | Different dispel or expiry clocks; replace with cleanse/turn/death trace |
| Burn clause | DamageApplied for Necrosis, AfterEvent, priority zero, once per event, including externally detonated Necrosis | Application-only, turn notification or natural-tick-only; replace with traces distinguishing these cases |
| Burn membership | Classified ordinary Burns plus periodic base Break Burn, excluding every Necrosis definition instance | Different Break inclusion or special Burn exclusions; replace with released selector or current trace |

Chance zero and certainty consume no draw under the shared chance contract;
intermediate outcomes consume one project-owned draw. Rejected commands preserve
state, hashes and counters. Guards and damage bounds use shared settlement.
All ATK/formula arithmetic uses checked project numerics; detonation scales the
complete unfloored formula by two and floors once.

The Burn reaction retains each Burn's original applier and source. It neither
ticks nor refreshes that Burn's duration/stacks. Both stores form one global
effect-instance-ordered pool. Shock, unclassified Fire damage, control/expiry
damage and Necrosis itself are excluded. No application-only damage is invented,
and no Necrosis-to-Necrosis recursive chain is possible. Other rule reactions
remain bounded by the shared resolver rather than a mode-owned state machine.

Equipment mutation goes through accepted Activity operations. Contribution and
battle digests bind the current decision bundle. Unequip affects future assembly
only; an already-created battle owns its immutable definitions and state.

## Native verification

[Real-bundle loader tests](../crates/starclock-data/src/divergent_universe_weighted_curio_necrosis_tests.rs)
check all four operands, exact joins, canonical decimal strings, required policy,
missing/duplicate rows and forged provenance at each source.
[Accepted-command tests](../crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_necrosis.rs)
cover both run families, Abundance versus ineligible owners, multi-hit/multi-target
cardinality, live chance inputs and seeded success/failure, three ticks and expiry,
mixed ordinary/Break Burn detonation, independent sources, exclusions, cross-caster
recapture, external detonation, fresh canonical events/state, rejection,
unequip and production battle-result handoffs. Controlled actions retain the
production-lowered rules; their artificial opponents are not encounter-parity
or complete-run evidence.
