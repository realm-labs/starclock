# Self-Amusement: target weight and nonlethal Additional damage

## Executable boundary

The production `WeightedCurioRetaliations` worksheet lowers the current
`divergent-universe.weighted-curio.1013` / 自娱自乐 definition. It joins released
Tourn3 Hex 1013, Display 1026 and generic MazeBuff 633413 level one. The exact
canonical operands are `4` and `0.3`; Physical element eligibility is explicit,
with no Path restriction. The released clauses increase Physical characters'
chance of being attacked and deal Additional damage based on the attacked
character's own 400%-ATK basis, without defeating the attacker.

Accepted equipment contributes immutable player bindings and overlays existing
enemy ability definitions during battle assembly. All mutations then use shared
combat commands, selectors, modifier stages, Rule IR, formulas, events and RNG.
There is no mode-specific battle machine or additional attack action. Unequip
removes these bindings from future assemblies; a live battle never queries or
mutates the Activity loadout. Locked build identities, resources, enemy specs,
ability order and manual/queued/forced target commitments are preserved.

## Released evidence

All six worksheet provenance rows bind released Version 4.4 of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
revision `fd978d6ef09f941fba644c731ab54abd6f7c3568`, accessed 2026-10-01.
Current file digests and row locators are enforced by the data validator:

| Path | Locator | SHA-256 |
| --- | --- | --- |
| `ExcelOutput/RogueTournHex.json` | HexID 1013, TournMode Tourn3, MazeBuffID 633413, Physical | `51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455` |
| `ExcelOutput/MazeBuff.json` | ID 633413, Lv 1, ParamList 4 / 0.3, StageAbility_633413 | `2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac` |
| `TextMap/TextMapEN.json` | Hash 6457248194266440334 | `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789` |
| `TextMap/TextMapCHS.json` | Same hash, independent Chinese cross-check | `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147` |
| `ExcelOutput/AvatarConfig.json` | Representative AvatarID-to-Path joins below | `c14584519e2feda70e9932501f7b79d67242d700585fe44119bbce9da9678e98` |
| `ExcelOutput/AvatarPromotionConfig.json` | Same AvatarIDs, Promotion 0–6, BaseAggro.Value | `4453f206d6b79658128f22ce4d923e2e608f92b48175ba5c913ed2be322d24c5` |

The current
[Self-Amusement public cross-check](https://honkai-star-rail.fandom.com/wiki/Self-Amusement)
distinguishes this Additional/nonlethal variant from the old Counter variant.
Do not substitute Protean Hero's Intoxicate or its 250%-ATK operand. No extracted
StageAbility program, unreleased source or bulk description is imported.

## Field-level targeting policy

`VersionedProjectPolicyPhysicalAggroAndOwnerAdditional` is not observed parity.
Qualification reads immutable canonical Basic scaling-damage elements from the
mapped player definitions, rejecting absent or conflicting elements. No content
ID branch, linked entity or summon inheritance is introduced. The exact `0.3`
operand is interpreted as a +30% `PercentOfBase` Aggro modifier.

While equipped only, the explicit authored baseline enters one Aggro-purpose
`FinalMultiply` after the normalized factor. The shared selector queries that
effective weight once; it never infers or multiplies a second Path baseline.
The representative joins are exact; applying them Path-wide, including unknown
character exceptions, is policy:

| Path | Representative AvatarID / source Path | Baseline |
| --- | --- | --- |
| Destruction | 1008 / Warrior | 125 |
| Hunt | 1002 / Rogue | 75 |
| Erudition | 1003 / Mage | 75 |
| Harmony | 1009 / Shaman | 100 |
| Nihility | 1004 / Warlock | 100 |
| Preservation | 1001 / Knight | 150 |
| Abundance | 1105 / Priest | 100 |
| Remembrance | 1402 / Memory | 100 |
| Elation | 1501 / Elation | 100 |

Stable ordered materialized enemy abilities acquire an automatic primary only
when their normal Basic/Skill is Opposing Single/Blast and currently unbound.
Explicit automatic bindings, self/All actions, ability choice/order and later-hit
invalidation remain unchanged. The shared current-state selector admits
alive/present candidates, formation order, exactly one primary and a fault for
an empty pool. It uses integer `aggro-target` battle sampling, once before action
declaration/payment. This does not globally change other modes or unequipped DU.
Unreconstructed enemy target locks and special target programs remain pending.

## Field-level reaction policy

An Ability-sourced Ordinary `DamageApplied` event with Attack tag and an action
admits an alive/present Physical owner as victim and an alive/present opposing
original actor as attacker. `AfterEvent`, priority zero, once per owner per
Action includes fully shielded and zero-HP-loss hits. Down/departed victims,
DoT, Break, Additional, reflected and non-Attack damage do not admit the rule.

`DamageFromOwner` reads effective victim ATK from the trigger snapshot, multiplies
by the exact `4` with floor rounding and uses the shared Physical Additional
formula. It cannot critically hit, reduce Toughness or defeat the attacker.
Owner/Actor/Applier credit belongs to the victim; original command/action/hit
ancestry remains. No extra action/turn is declared and the emitted damage cannot
recursively admit this rule.

Missing facts include the mapping of `0.3`, Path exceptions, target-lock
admission, hit/action cardinality, reaction phase, shield/zero-hit admission,
snapshot, damage element, critical eligibility and linked inheritance. Confidence
is low for those hidden fields. Alternatives include a multiplicative bonus,
per-hit or after-action reactions, effective-loss-only admission, attacker-element
or critical damage and retaining unsupported fixed targeting. The selected
policy executes both clauses using existing bounded primitives. Replace each
field independently when released executable data or reproducible current traces
establish it; the worksheet preserves rationale, alternatives and replacement
conditions separately from `ExactStructured` provenance.

## Verification and remaining scope

Data tests validate the real Sora bundle and reject wrong joins, decimals,
baselines, duplicate/missing definitions and forged source locators/digests.
Native battle tests cover Physical/non-Physical binding, actual materialized
enemy primaries, distinct victim ATK, three-hit once-per-action cardinality,
nonlethal one-HP floor, shield/zero hits, invalid damage classes, dead/linked
owners, fresh hashes and a 64-seed exact integer-weight comparison. Both run
families have real nested-battle handoffs and fresh reconstruction, rejected
commands and lawful unequip. A controlled attack fixture isolates the mechanic;
the real battle still uses the explicitly labeled shared-minion proxy.

The qualification roster uses Physical Trailblazer and Natasha. A separate
fixture composes production Clara's
[bounded representative Counter](representative-character-v1b-production.md#bounded-counter-admission)
admission with this Additional damage in both families, including charge
exhaustion. The unbound internal Counter is cancelled, not executed; genuine
Counter/Additional composition and full released Clara parity remain pending.

Seven of seventeen Weighted Curio effects now execute. The other ten,
player-facing Forge offers/slot admission, encoded equipment-command replay,
genuine Grand Miracles and complete original runs remain pending. These tests
do not terminalize a source obligation, mechanic program or semantic family.
