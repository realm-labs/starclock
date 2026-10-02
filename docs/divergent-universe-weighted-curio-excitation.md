# Genius' Confusion policy execution

The current decision bundle contains an immutable definition for Weighted Curio
1006, **Genius' Confusion / 天才的迷茫时间**. The
[accepted loadout boundary](divergent-universe-weighted-curio-loadout.md) admits
its native shared Rule IR assembly in both run families. Ten Weighted Curio
effects execute; seven still reject as `UnsupportedBattleEffect`. Hidden
timing and the ordinary Entanglement fallback remain replaceable project
policy, not observed original-game parity or complete Forge admission.

## Released facts

The pinned released revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` binds
`RogueTournHex` HexID 1006 in **Tourn3** to MazeBuff 633406, DisplayID 1011,
Quantum eligibility and no Path restriction. `MazeBuff` ID 633406, level one,
binds `StageAbility_633406` before character birth. Its five canonical operands
are independently joined to both text maps, description hash
`332970425552739522`:

| Parameter | Released meaning | Canonical value |
|---|---|---|
| 1 | Team-wide Excitation gained after a Quantum ally gains Skill Points | `2` |
| 2 | Team-wide Excitation consumed after a Basic or Skill attack | `1` |
| 3 | Quantum Additional damage as a fraction of ATK | `2.5` |
| 4 | Base chance to inflict Entanglement | `0.5` |
| 5 | Entanglement duration in turns | `1` |

Source rows retain repository, revision, game version, access date, exact row
locators and all four file digests. No number is converted through floating
cells. The current
[public Curio list](https://honkai-star-rail.fandom.com/wiki/Divergent_Universe%3A_Arcadian_Chronicles/Curio)
cross-checks only these player-facing clauses, not hidden execution behavior.
The pinned repository object tree has no `StageAbility_633406` program; this is
not an inference from a sparse checkout.

## Replaceable review policy

`EffectiveGainActionResolvedTeamConsumption` is explicitly
`VersionedProjectPolicy`, with low confidence for undisclosed fields:

- Original mapped party elements qualify positive **effective** Skill Point
  gain events. Linked/shared actors cannot inherit the original owner's
  qualification. Gain once per resource event, not per point; overflow-only
  events do not grant stacks.
- Grant two stacks to living/present original party members. The separate
  `policy_maximum_stacks = 65535` is an unsigned-domain safety bound, not a sixth
  released operand. Stacks are nondispellable, start at zero for a fresh battle,
  persist across waves/provider defeat and do not carry through Activity.
- Original party Basic/Skill actions must be tagged Attack. Observe
  `ActionResolved`, AfterAction, priority zero, once per action. The actor must
  have a positive stack; subtract up to one from every living/present original
  party member before the Additional damage. Multi-hit attacks do not consume
  once per hit. A qualifying gain inside the action precedes this consumption.
- Deal one actor-attributed Quantum Additional packet to each surviving
  attacked target in formation order, using trigger-boundary ATK times `2.5`
  and ordinary target-dynamic stages, with an independent Crit draw. Then
  attempt Entanglement through the ordinary effect-hit/control-resistance
  pipeline and labeled battle RNG. Neither operation declares another action.
- Entanglement must retain action delay, subsequent hit accumulation and delayed
  damage; a marker-only effect or an action skip does not preserve the mechanic.
  The reviewed shared Quantum base-effect calculation is the selected fallback,
  **not observed parity** for this non-Break application. Do not change Toughness
  or Weakness Broken, deal initial Break damage or add universal Break delay.
  Refresh same-definition Entanglement across casters without repeating delay
  while active; cleanse removes it without expiry damage.

Alternatives, rationale and replacement conditions are authored alongside the
operands. Replace each policy field independently with a released executable
binding or reproducible current observation. Ordinary weak-point Quantum Break
exists in the shared core, but applying that state through an ordinary
resistible effect now has a separate native shared capability. It uses the
ordinary effect/chance/cleanse store and never applies Weakness Break. The
[shared contract](11-rule-ir-and-native-handlers.md) explicitly defines capture,
five subsequent damaging hits, one initial delay, cross-caster refresh,
target-turn expiry, actual broken-state factors and declared applier teardown.
Real generic commands verify these lifecycle boundaries, live Effect Hit Rate,
Effect RES and template-declared Control Resistance, and fresh reconstruction.
The mode assembly now binds this lifecycle to the Curio's policy through normal
resistible `ApplyEffect` emissions. It captures the first successful applier's
actual battle-level Break base, rather than assuming equal party levels.
Unavailable initial level entries fault without guessing. Core fallback tests
and mode commands establish execution, not original-game parity.

Shared Rule IR now distinguishes `BalanceChanged` from `MaximumChanged` through
the typed `resource_event` fact and filter. Both `SkillPoints` and
`SkillPointMaximum` retain the Skill Points address, and `resource_delta` on the
latter remains the maximum's delta, including when the balance is clamped.
A positive delta alone is insufficient: the native mode assembly requires
`BalanceChanged`, the Skill Points address and a positive
effective delta. Real generic battle commands verify that cap-only increases,
cap-induced clamps and overflow-only balance events cannot satisfy that
combination. The mode binds that filter to a separate nondispellable per-member
stack effect and consumes it through checked `AdjustEffectStacks` operations.
No production Sora filter column is authored for the resource axis yet.

The immutable Sora operands and decision digest determine the mode assembly.
Original membership uses mapped form and formation together; linked ownership
is excluded even if the linked actor inherits the producer's rule bundle.
Owner selectors use a direct `First` anchor, not an all-team candidate pool.
After consumption, `DamageFromOwner` produces Quantum Additional damage with
an independent Crit context. A source-filtered `DamageApplied` AfterEvent
reaction reselects the surviving damaged primary before attempting Entanglement.
This avoids attaching control to a target defeated by the Additional packet;
unrelated Additional damage cannot invoke the Curio's application program.
Each trigger has priority zero and an explicit Event or Action once-scope.
Stacks and effects remain battle-local; no live Activity mutation occurs.

Independent shared level-70/80 vectors additionally verify cross-caster capture
retention and expiry. No production Sora Entanglement payload or Control
Resistance column is authored yet; this mode uses native catalog composition,
not an alternative runtime loader or a content-specific resolver branch.
Complete public battle/Activity equipment replay remains pending.

## Validation and remaining boundaries

Native data fixtures verify all five operands, eligibility/reference joins,
policy-cap separation, fresh catalog reconstruction, malformed/noncanonical
values, cross-family key collisions and forged/missing/duplicate provenance.
The authoring verifier optionally re-reads and hashes the pinned Git objects.
Sora drift validation covers the schema, workbook, readers, binary and debug
export. The workbook contains 45 sheets, including the definition and its four
provenance rows; rendering checks the policy cells.

Those are **data checks**, not battle fixtures. Separate Ordinary and Cyclical
fixtures execute unchanged production Quantum Basic attacks and controlled real
commands for effective gains versus point counts, overflow, non-Quantum and
empty Quantum rosters, cap-only changes, negative balance, recipient-local
partial-team counts, the safety cap, multihit once-per-action consumption,
nonattack/Ultimate exclusions, multi-target Additional damage and independent
Crit. Actual Summon, Memosprite and SharedActor commands retain inherited
producer rules while proving they cannot gain or consume original-party stacks.
Effect commands exercise Effect RES, Control Resistance, labeled chance draws,
one delay, refresh capture, zero-to-five hit accumulation, expiry damage,
cleanse, lethal Additional exclusion, provider defeat and wave carry. Fresh
construction compares event payloads, hashes and RNG; stale commands are inert
and unequip removes both definitions. Fresh battles start with no stacks.
Forge admission, full-run release and terminal reference/program dispositions
remain pending. Catalog loading grants none of those credits.

```text
node tools/divergent-universe-runtime/verify-weighted-curio-excitation-authoring.mjs --check-source
node tools/divergent-universe-runtime/verify-decision-workbook.mjs
cargo test -p starclock-data weighted_curio_excitation
cargo test -p starclock-mode-universe weighted_curio_excitation
cargo test -p starclock-test-kit --test combat_suite resource_event
```
