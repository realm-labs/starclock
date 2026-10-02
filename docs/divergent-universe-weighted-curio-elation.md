# Sapient Pen authoring boundary

## Current state

Production `WeightedCurioElations` now privately lowers one Sapient Pen
(智多笔) definition through the Decision workbook and Sora 0.6.1. The three
released operands are two Punchlines, an additive Elation bonus of `0.5`, and
two turns. This is authored data, not an executable Curio: equipping it still
causes typed `UnsupportedBattleEffect` rejection before battle construction.
The current eight executable Weighted Curio definitions and all runtime
coverage denominators/dispositions are unchanged.

The private definition separates exact values from a closed, reviewed
`VersionedProjectPolicyActionResolvedOriginalPartyRefresh` envelope. No
generated transport row, workbook or normalized JSON enters combat. Loading
this definition does not establish Forge admission, ordinary Curio ownership,
Grand Miracle behavior, or a complete public run.

## Released source joins

The frozen reference identity is
`divergent-universe.weighted-curio.1015`, with current eligibility Path
`Elation` and no element restriction. The production definition independently
resolves the generic MazeBuff missing from the original RogueMazeBuff lookup.
Pinned released revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-10-02, supplies:

| Source | Exact locator | SHA-256 |
| --- | --- | --- |
| `ExcelOutput/RogueTournHex.json` | `HexID=1015; TournMode=Tourn3; MazeBuffID=633415; AvatarType=Elation; DisplayID=1027` | `51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455` |
| `ExcelOutput/MazeBuff.json` | `ID=633415; Lv=1; ParamList=2,0.5,2; StageAbilityBeforeCharacterBorn; StageAbility_633415` | `2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac` |
| `TextMap/TextMapEN.json` | `hash=7976287788830292485` | `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789` |
| `TextMap/TextMapCHS.json` | `hash=7976287788830292485` | `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147` |

The actual `Tourn3` predicate proves membership; neither the adjacent Hex IDs
nor similarly named previous-module effects do. Both text maps describe
non-Elation Basic/Skill use when an Elation character is in the party, with
Punchline gain and an Elation-character bonus. Their percentage parameter is
stored as canonical decimal string `0.5`, never as an Excel floating cell.
The independent [Sapient Pen cross-check](https://honkai-star-rail.fandom.com/wiki/Sapient_Pen),
inspected 2026-10-02, supports the current Arcadian Chronicles clauses, not
hidden scheduling. Its older Quantum/Wind program is outside this definition.

## Reviewed execution policy, not yet implemented

The cached pinned source tree contains no released `StageAbility_633415`
program path. Bounded public research also did not establish hidden trigger,
recipient, stacking or meter lifecycle details. The authored choices are:

- Original immutable party combat forms determine Path membership outside
  combat. Any original Elation form satisfies presence, even if defeated.
- Each original non-Elation owner reacts to Basic or Skill at `ActionResolved`,
  `AfterAction`, priority zero, once per action. No Attack-tag requirement or
  per-hit multiplication; other action kinds and linked actors are excluded.
- First gain two on the side's unique `shared.punchline` resource binding, then
  grant `StatKind::Elation` additive `0.5` to living/present original Elation
  recipients in formation order. One dispellable effect replaces/refreshes
  ownership across casters and expires after two recipient `TargetTurnEnd`s.
- No Curio-specific RNG, live Activity access or second combat state machine.

These choices have low behavioral confidence. Alternatives include active-only
presence, per-hit grants, caster-specific stacking, nondispellable bonuses and
source-turn duration. Replace fields independently when the released stage
program or reproducible current traces establish them. The separate
[shared Elation boundary](shared-elation-runtime-boundary.md) supplies generic
stat and keyed-resource primitives. The separate
[battle resource policy](divergent-universe-battle-team-resources.md) now
assembles the meter with explicit initialization, cap, overflow and wave/battle
choices and command-level tests. That prerequisite does not execute Sapient
Pen's still-unimplemented trigger or bonus.

Runtime admission must prove actual Basic/Skill commands in both Ordinary and
Cyclical assemblies, no-Elation and no-eligible-owner cases, multihit and
nonattack Skills, excluded action/entity kinds, cross-caster refresh, duration,
defeat/presence, overflow, waves, fresh reconstruction, rejections and unequip.
That future evidence must precede any execution or terminal coverage credit.

## Verification

Data tests load the real production Sora bundle, compare fresh private lowering,
reject wrong joins and noncanonical operands, reject missing/forged provenance
at every required source, and reject cross-family key collisions. They are
not battle fixtures. Workbook QA compares all existing cells/styles/controls
against the pre-edit workbook and renders the changed numeric region using
the existing Sora template layout.

```text
node tools/divergent-universe-runtime/verify-weighted-curio-elation-authoring.mjs --check-source
node tools/divergent-universe-runtime/verify-decision-workbook.mjs
cargo test -p starclock-data weighted_curio_elation
cargo test -p starclock-mode-universe weighted_curio_unlowered_and_dirty_loadouts
```
