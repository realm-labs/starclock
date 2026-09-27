# Fixed Conversion battle-substitute policy

The production factory's existing `BattleRoomCompiler` now exposes
`compile_conversion_sequence(context, BattleRoomSequenceLength)`. It admits only
a currently joined fixed preset 1004 / Conversion / level 1. Its immutable
`CompiledBattleRoom` binds through `battle_room_identity` and `bind_battle_rooms`
to the same source-position profile, battle assembly and shared settlement as
ordinary battle fragments. No second battle engine or mode command processor
is introduced. The public default remains the three-battle proxy.

## Released constraint and missing facts

Pinned released Version 4.4 evidence is
`Dimbreath/TurnBasedGameData@fd978d6ef09f941fba644c731ab54abd6f7c3568`:

- `ExcelOutput/RoguePersonaRoomPreset.json`, preset 1004, composition type 23,
  level 1; SHA-256
  `a4cc8bbd6e3a4db5fecb62b83a0c5de7ad2f4a9820ad5ae9180b30ae33c346b5`.
- `ExcelOutput/RoguePersonaRoomComposition.json`, type 23 / level 1; the current
  reviewed composition has no admitted wave selector or reward program; SHA-256
  `662e0730248b6c80c9b46109a66322b04c356a9482d39216135a0a83f9484929`.
- `ExcelOutput/RoguePersonaRoomCompType.json`, type 23 / Conversion;
  SHA-256 `c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5`.
  Description hash `4494024472627880387` requires enemy-wave progress to affect
  rewards while defeat does not end exploration. Level text hash
  `14342336820829885572` disallows leveling this domain.
- `TextMap/TextMapEN.json`, those exact hashes; SHA-256
  `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789`.

The current area-position joins place it at position four in layers 3012 and
3022. See [current source layout](divergent-universe-domain-layout.md). A bounded
public cross-check on 2026-09-27 agrees with the defeat-continuation constraint:
[Arcadian Chronicles domains](https://honkai-star-rail.fandom.com/wiki/Divergent_Universe%3A_Arcadian_Chronicles/Domains).
That secondary page is not selector, reward-tier or recovery evidence. Older
Protean Hero observations are not current-domain admission evidence.

The reviewed source closure and public cross-check do not establish an executable
current enemy-wave selector, timer/end threshold, partial-wave reward tier,
recovery rule or reward offer weights. The exact join and text constraint are
separate from the following deliberately non-parity execution policy.

## Explicit replaceable policy

`VersionedProjectPolicyConversionSameCandidateSequenceLossEndsRoomWithoutHealing`
uses the caller's already reviewed candidate group/stage and authored reward
domain. The caller explicitly selects one through four real sequential battles;
this substitutes complete battles for unavailable original Conversion waves.
It neither counts enemy objects as battles nor adds a wave counter to Activity.
Any actual waves inside a `BattleSpec` execute in the existing combat engine.

Every verified victory uses the current ordered fragment, Curio, Blessing and
participant-carry settlement. Intermediate reward acceptance or suppression
advances to the next encounter in the same logical room. A verified loss skips
all remaining battles and advances through the final empty reward node to the
next source position. No loss victory grant, reward sample, partial-wave reward,
retry, healing or revival is added. Previously earned rewards remain owned.
A battle fault retains its independent fault terminal; it is not defeat.

Curio domain entry occurs once. Every actual won/lost battle consumes its normal
battle lifetime through the existing settlement. Each encounter clears the prior
candidate/acceptance gates, so defeat cannot reuse an earlier offer. Exact
HP, energy, life and the existing `DepartIfDefeated` presence carry remain
authoritative after the room exits,
even if no participant remains alive. Continuing exploration does not imply
that the unchanged party can legally win its next battle.

The sequence length, handoffs, exact context, source/decision inputs and policy
tag are configuration identity. Observation does not draw RNG. Victory rewards
use existing labeled streams and stable candidate order; defeat/fault do not
sample victory rewards. Settlement plus all automatic advancement is one shared
transaction: a failing successor restores pending result, carry, Curio lifetime,
events, visits, state bytes/hash and RNG. Duplicate or foreign results reject.
No production workbook or source ownership is altered by this caller policy.

Rejected alternatives: reusing ordinary defeat termination contradicts the
published constraint; treating loss as victory fabricates rewards; implicit
healing invents recovery; counting nested battles as observed original waves
would misstate parity. Confidence is high in the joined preset and defeat
constraint, low in substitute challenge/reward parity. Replace the substitute
when released selectors/programs or reproducible current observations establish
wave thresholds, reward operands and post-defeat recovery independently.

## Verification and delivery boundary

Focused native tests bind the real Sora-backed factory and source-position
profiles in both run families. They cover policy/context/digest authentication,
actual repeated battles and natural defeat with unchanged carried participants,
fresh separately constructed execution, and repeated duplicate rejection.
Separate explicitly counterfactual result-contract probes cover loss/fault at
every handoff of lengths one through four, absent loss rewards and complete
rollback after a failing successor, including an active battle-limited Curio.

Other room payloads in these profiles remain explicit test probes. This does
not complete original Conversion waves or partial-wave rewards, source enemy
admission, a default position-run assembler, encoded source-position replay,
terminal reference coverage or complete Ordinary/Cyclical release runs.
