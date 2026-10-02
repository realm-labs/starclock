# Encouragement for You data boundary

## Current state

Production `WeightedCurioEncouragements` privately lowers one current
Encouragement for You (给你鼓励) definition through openpyxl and Sora 0.6.1.
The exact operand is canonical decimal string `1.5`, an additive 150% follow-up
critical-damage ratio. The definition records both required clauses: Elation
damage also qualifies as follow-up damage, and qualifying follow-up critical
damage increases. Neither effect executes yet. Equipped Curio 1008 still fails
closed before battle assembly; no native handler or no-op admission was added.
The ten executable effects, seven unsupported effects and all terminal
reference/program dispositions are unchanged.

## Released source joins

Pinned released revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-10-02, supplies the four exact sources below. The definition
resolves the generic `MazeBuff` that the reference-only `RogueMazeBuff` lookup
did not contain; it does not promote that reference payload to runtime-lowered.

| Source | Locator | SHA-256 |
| --- | --- | --- |
| `ExcelOutput/RogueTournHex.json` | `HexID=1008; TournMode=Tourn3; MazeBuffID=633408; AvatarType=Elation; DisplayID=1029` | `51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455` |
| `ExcelOutput/MazeBuff.json` | `ID=633408; Lv=1; ParamList=1.5; StageAbilityBeforeCharacterBorn; StageAbility_633408` | `2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac` |
| `TextMap/TextMapEN.json` | `hash=2693397682820933078` | `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789` |
| `TextMap/TextMapCHS.json` | `hash=2693397682820933078` | `ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147` |

The actual `Tourn3` row proves current membership and Elation Path eligibility,
with no element restriction. Both text maps bind parameter one to the critical
damage ratio. The independently inspected
[Curio page](https://honkai-star-rail.fandom.com/wiki/Encouragement_for_You)
corroborates those current Arcadian Chronicles clauses. The earlier
Remembrance/memosprite auxiliary-ability variant is excluded, not imported as a
current mechanic. No file containing `633408` exists in the pinned Git object
tree, so it does not supply an executable StageAbility program.

## Reviewed policy and pending native work

`VersionedProjectPolicyOriginalElationFollowUpDamage` is an explicit,
low-confidence envelope, not observed parity. It selects these replaceable
choices for the subsequent runtime batch:

- Original mapped Elation party owners qualify; linked summons, memosprites and
  shared actors do not inherit eligibility through a shared form or bundle.
- Add orthogonal follow-up **damage** semantics to Elation hits. Keep
  `DamageClass::Elation`, its dedicated calculator, action kind/origin, costs,
  hit count and Crit grouping. Do not create another action, hit or event.
- Existing follow-up hits from eligible owners also qualify. Add `1.5` once to
  their hit-scoped `CritDamage` query before critical multiplication. Do not
  alter Crit Rate, Elation stat, ordinary DMG Boost or noncritical damage.
- Keep battle-lifetime ownership through waves; reconstruct from accepted
  equipment for each fresh battle without changing Activity state. DoT, Break,
  Super Break and True damage receive no critical bonus.

Shared native classification must consistently reach formula filters, event
filters and selectors without duplicating reactions. Action-tag replacement
alone would incorrectly relabel non-Elation hits in mixed actions. A global
CritDamage stat buff would incorrectly affect unrelated damage. Both shortcuts
are outside this policy. The generic combat crate must not contain this Curio
identity or query the build catalog.

Alternatives include replacing labels, whole-action reclassification, inherited
linked eligibility or a formula-stage rather than scoped-stat bonus. Replace
each hidden choice independently when a released program or reproducible
current observation establishes it. No original Forge offer/slot placement,
public equipment replay or complete-run release claim follows from these rows.

## Validation boundary

Data tests load the actual production binary, compare fresh reconstruction,
preserve the exact ratio and reject wrong membership/effect joins, changed
parameter indices, noncanonical strings, floating numeric transport,
empty/false-parity policies, duplicate/missing rows, aliased family keys and
forged provenance at each required source. The authoring verifier independently
checks pinned Git blob hashes and current joins. Sora drift checks compare clean
schema/readers/exports and a freshly openpyxl-authored workbook.

Those checks prove data validity only. Command-level both-family damage,
classification/trigger, linked-actor, Crit/RNG, wave, unequip, rejection and fresh
reconstruction fixtures remain required before runtime admission.
