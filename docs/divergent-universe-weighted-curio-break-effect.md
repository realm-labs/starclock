# Converse of Entropy: entry Break Effect capture

## Released facts

The current normalized `weighted-curios.json` record
`divergent-universe.weighted-curio.1005` belongs to `Tourn3`, not a
name-inferred shared pool. Its reference pack digest is
`59fe8211da025d6526f08f9d0a8e1a9955ae60219ba8b82fa6cab9f1f2b544b6`.
The pack validator and pinned source blobs were checked on 2026-10-01.

Released Version 4.4 revision `fd978d6ef09f941fba644c731ab54abd6f7c3568`
of [Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata)
supplies these exact joins:

- `ExcelOutput/RogueTournHex.json`: HexID 1005, TournMode Tourn3,
  AvatarDamageType Wind/Thunder, no Path restriction, DisplayID 1018,
  MazeBuffID 633405. SHA-256
  `51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455`.
- `ExcelOutput/MazeBuff.json`: ID 633405, Lv 1, ParamList[0].Value `1.2`,
  binding `StageAbilityBeforeCharacterBorn` / `StageAbility_633405`,
  description hash `6003923322476481427`. SHA-256
  `2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac`.
- `TextMap/TextMapEN.json`, that description hash: entry raises Wind and
  Lightning allies' Break Effect **to**, not by, 120% of the team's highest
  Break Effect. SHA-256
  `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789`.
- `TextMap/TextMapCHS.json`, the same hash, independently supplies the
  Chinese clause. SHA-256
  `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147`.

The [current community entry](https://honkai-star-rail.fandom.com/wiki/Converse_of_Entropy),
accessed 2026-10-01, cross-checks the current 120% clause and distinguishes
the historical 90% variant. It is not a second authority for hidden timing.
The unresolved reference `RogueMazeBuff` locator does not erase the separate
released generic `MazeBuff` evidence.

## Current mechanic contract and review fixture

The production Sora definition lowers one equipment contribution into shared
typed Rule IR, with no mode
branch in the battle resolver. At one entry boundary, select the greatest
effective Break Effect `M` among the current locked player party. Each eligible
Wind/Lightning recipient with entry value `B` receives a permanent,
non-dispellable modifier with captured magnitude `max(1.2 * M - B, 0)`.
The shared fixed-point backend multiplies with `NearestTiesEven` rounding.
The effect's Scalar magnitude capture precedes modifier snapshot evaluation.

One `ApplyEffect` operation must resolve every recipient against the same
immutable input, before inserting any recipient's modifier. Binding a
separate producer per recipient without global admission would compound `M`
and violates this contract. Bind the producer to party members but admit only
the first alive/present member in formation order, through selector exclusion
and cardinality conditions. This remains correct when the first party member
is downed. Select original party forms, not enemies or newly linked units.

Native review vectors:

- party entry values 50%, 100%, 200%, 75%: all eligible recipients finish at
  240%, even when the 200% member has an ineligible element;
- a highest eligible 200% member and another eligible 50% member both finish
  at 240%, never 288% or a per-owner compounded result;
- tied and all-zero inputs have deterministic results without RNG;
- later Break Effect modifiers remain live additions, without recapturing
  this entry bonus; real Break damage changes with the captured value;
- entry roster reordering, a downed first member, unequal builds, fresh
  reconstruction, rejected commands and both run families preserve the same
  declared semantics; unequip removes future entry bindings;
- no eligible member creates no recipient effect; enemy and linked-unit
  values neither set the maximum nor inherit the bonus.

## Field-level policy and remaining acceptance

`VersionedProjectPolicyEntryHighestTeamBreakEffectCapture` chooses
BattleStarted/AfterEvent, priority zero, once per battle, as the available
shared equivalent of the source's before-character-born binding. It reads
entry-effective stats after immutable assembly modifiers and prior ordered
entry reactions, excludes downed/departed party members, and maps element
through each mapped character's canonical Basic scaling damage. No summon,
memosprite, transformed-form or later-wave inheritance is invented.

An entry-captured nonnegative Flat/Stat Break Effect addition persists only
for this battle; later changes do not turn it into a dynamic team aura or
permanent stat override. Multiple current recipients share one pre-insertion
snapshot. The source-backed `1.2`, element clause and highest-team basis are
not policies. Timing relative to other entry effects, inactive eligibility,
element mapping, linked inheritance, snapshot/teardown, rounding and negative
input handling remain independent low-confidence policies, not observed parity.

Alternatives include authored before-birth scheduling, inactive-roster maxima,
dynamic recomputation and an absolute stat override. The chosen policy preserves
the released entry/raise-to clause with the shared command/effect/modifier
pipeline. Replace each field independently when released executable programs
or reproducible current traces establish it. Data tests must reject altered
joins, noncanonical decimals and forged provenance. Native `weighted_curio_break_effect`
tests must execute entry effects and real Break damage from production lowering.

This contract alone is not execution evidence. Forge admission, encoded
equipment-command replay, all other equipment effects, genuine Grand Miracles
and complete public-run gates remain independent acceptance work. No source
obligation, mechanic program or semantic family is terminalized by this file.
