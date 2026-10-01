# Divergent Universe Weighted Curio Harmony shields

## Released facts

`WeightedCurioShields` authors Silent Song / 静谧的歌声, current Hex 1002 / Tourn3,
Display 1015, generic MazeBuff 633402. Sources 77–80 pin released Version 4.4
revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-09-27. Exact digests/locators live in the production Sources sheet;
no source programs or bulk prose are committed.

- `RogueTournHex.json`, HexID 1002: current Tourn3, Shaman / Harmony,
  no element restriction, MazeBuff 633402.
- Generic `MazeBuff.json`, ID 633402 / Lv 1: canonical parameters `0.35` and `2`,
  binding `StageAbility_633402` before character birth.
- EN/CHS TextMap hash `18303557854201536316`: Harmony Basic/Skill/Ultimate on
  allies gives all allies shields based on their respective maximum HP for
  two turns. The binding phase does not establish trigger timing or refresh.

The Hex verifier checks pinned digests, eligibility, both operands, bilingual
ally direction and the authored row. A bounded pinned filename search did not
locate a 633402 executable file; this does not prove program absence. Bounded
public research did not establish hidden timing/refresh. Preview/leak results
were not admitted. Earlier-module text and community reports are not authority
for current numbers/lifecycle. The reference catalog's unresolved RogueMazeBuff
locator is not absence of generic MazeBuff facts.

## Executable policy

`VersionedProjectPolicyAllyActionResolvedReplaceTargetTurnShield` implements:

- Mapped Harmony players produce shields at `ActionResolved` / `AfterAction`,
  once per Basic/Skill/Ultimate with nonempty committed allied targets. Self/team
  actions need no controller primary. Enemy-only/non-Harmony/other action kinds
  cannot produce it; damage/heal events are not substitutes.
- Visit current living present recipients in formation/stable-ID order. Each
  base capacity uses its own current maximum HP times `0.35`, checked fixed-point
  floor and normal shared shield boosts. Without other modifiers, HP
  `101/203/307` produces capacity `35/71/107`.
- Remove only this definition's old capacity/effect, then install one
  non-dispellable replacement. Repeated actions refresh rather than add capacity.
  Other shield definitions coexist under `ConcurrentLargest` absorption.
- Exact duration two uses `TargetTurnEnd`; an already-running recipient turn
  counts at that clock edge. Each mapped recipient owns expiry cleanup,
  independent of caster survival. Removed effects clear shields unless a
  replacement already exists; delayed old-effect removal cannot erase refresh.
  Existing shared stores and Rule IR execute this, not a mode-specific engine.
- Equipment is Run-scoped; battles get fresh source-attributed producer/cleanup
  bundles. Locked builds/base stats/old bindings/enemies remain intact. No live
  Activity mutation or shield RNG; linked/new summon inheritance is not implicit.
  Unequip affects later snapshots, not previously constructed battles.

Phase, committed-versus-declaration targets, alive/present scope, immediate turn
tick, shield boosts, multi-caster replacement and coexistence are low-confidence
policy, **not observed parity**. Alternatives include declaration-time shields,
independent per-caster layers or duration-only refresh. Replace fields independently
with released programs or reproducible current traces; preserve exact
Harmony/action/direction/HP/fraction/duration facts.

## Verification and remaining work

Production-backed fixtures in both families execute shared commands for all three
action kinds, multi-hit once-only grants, distinct recipient HP, self/team targets,
enemy-direction and non-Harmony rejection, fresh-factory determinism, replacement,
damage absorption and timed removal. Data tests reject wrong joins, malformed
fractions/durations, missing policy/provenance and duplicates. A real production
battle handoff also runs with equipment, not original Forge admission.

The workbook has 37 tables / 380 rows. Three of seventeen Weighted Curio effects
execute, including [Automated Experience](divergent-universe-weighted-curio-attack-debuff.md);
fourteen remain fail-closed. Original Forge offers/slots, encoded equipment
replay and complete 13/17/20-position runs remain incomplete. No original
obligation, program, family, gap or policy-source terminal credit is added.
