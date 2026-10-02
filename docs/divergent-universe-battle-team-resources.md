# Divergent Universe shared battle resources

## Current execution boundary

Production `BattleTeamResources` privately lowers the mode's shared Punchline
assembly policy through the Decision workbook and Sora 0.6.1. The original
immutable mapped party enables exactly one player-side `shared.punchline`
binding if any character has the Elation Path. It does not depend on equipping
Sapient Pen. Non-Elation-only parties and the enemy side get no binding.

The mode constructs the existing generic `KeyedTeamResourceSpec`. Combat owns
every live gain, cost, event and wave transition. No source ID, character ID or
mode branch is added to combat. Original character membership is resolved
through the build catalog before construction; newly created linked actors do
not participate in this assembly predicate. The Decision bundle digest is
already bound into the Activity definition and therefore the battle assembly,
cache key, configuration and replay identity.

Existing production `AbilityResourceDelta` rows author gains for
`shared.punchline`, including three on a Skill from source avatar locator
`1502`. Current generic data compilation omits non-cost keyed-resource deltas:
the real mapped Skill still produces no Punchline event or gain. This remains
a separate pending data-lowering responsibility, not execution credit.
The locator selects the test party through data, not a combat branch.

The [Sapient Pen definition](divergent-universe-weighted-curio-elation.md)
now separately executes its authored trigger and timed bonus against this
resource. The assembly policy alone grants no Curio execution credit. Aha actor
scheduling, Certified Banger, automatic entry gains, meter-driven damage scaling and full Elation
lifecycle remain separate pending responsibilities. No reference obligation,
mechanic-program or policy-source terminal count changes from this prerequisite.

## Released facts versus replaceable policy

Pinned released revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
Version 4.4, accessed 2026-10-02, supplies three exact source locators:

| Source | Locator | SHA-256 |
| --- | --- | --- |
| `Config/GlobalConfig/GameCoreConstValue.json` | `ElationPointMax=9999` | `5511ff36c631da99925c8aadae8ae46f50d620f4f937dbbed952fd61b16b80e3` |
| `TextMap/TextMapEN.json` | `hash=8389201339365092983`, team sharing | `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789` |
| `TextMap/TextMapCHS.json` | same hash, team sharing | `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147` |

The global field's value and bilingual team-sharing statement are exact facts.
Connecting that field to this meter, the enabling condition and the following
lifecycle are explicitly low-confidence
`VersionedProjectPolicyOriginalElationZeroClampPersist`, not observed parity:

- Start at zero in each new battle or retry reconstruction.
- Use a maximum of 9,999. Gains clamp through the generic operation, emitting
  attempted, effective and overflow amounts even when already capped.
- Costs require sufficient balance before acceptance. Rejected commands leave
  canonical state and RNG untouched. Set uses the generic checked bounds.
- Persist through waves within one battle. Do not carry the meter into Activity
  inventory or another battle.

`ExcelOutput/ElationBattleConstCommon.json` has a different, activity-specific
`ElationBattle_MaxPower=99` under ActivityID 50064. It is not the global field
and is not admitted as this meter's cap. Public reports from another mode do
not establish Divergent Universe carry or initialization behavior.

Alternatives include unconditional binding, entry gains based on roster count,
wave reset, checked overflow and cross-battle carry. Replace each policy field
independently when a released executable binding or reproducible current-mode
observation establishes it. See the [generic Elation boundary](shared-elation-runtime-boundary.md)
for the separate damage and shared-actor responsibilities.

## Verification

Data tests load the real generated bundle, compare fresh immutable lowering,
and reject unknown resource/Path/policy identities, bounds, cardinality,
cross-family key collisions and every required provenance field. Authoring
verification reads and hashes the exact pinned Git blobs without copying raw
source material into the repository.

Mode tests cover Ordinary and Cyclical assembly, zero/one/two Elation members,
no Curio dependency, unchanged Activity state, the still-unimplemented
production Skill gain boundary, fresh command reconstruction with identical
events/hashes/RNG, and new battle initialization. Controlled commands retain
the exact production resource spec
to isolate insufficient-cost rejection, cap/overflow events, spending, Set
and actual wave entry with persistent balance. These commands are lifecycle
probes, not original character behavior or full public-run evidence.

```text
node tools/divergent-universe-runtime/verify-battle-team-resource-authoring.mjs --check-source
node tools/divergent-universe-runtime/verify-decision-workbook.mjs
cargo test -p starclock-data battle_team_resource
cargo test -p starclock-mode-universe battle_team_resources
```
