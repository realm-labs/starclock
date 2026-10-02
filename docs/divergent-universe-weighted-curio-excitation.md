# Genius' Confusion authoring and pending execution

The current decision bundle contains an immutable definition for Weighted Curio
1006, **Genius' Confusion / 天才的迷茫时间**. It does not yet execute or admit this
equipment to a battle. The [loadout boundary](divergent-universe-weighted-curio-loadout.md)
still rejects it as `UnsupportedBattleEffect`; the nine implemented Weighted
Curio effects and eight pending effects remain unchanged.

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
These are shared fallback tests, not execution of this Curio's policy or
original-game parity. Its contribution still rejects as unsupported.

Shared Rule IR now distinguishes `BalanceChanged` from `MaximumChanged` through
the typed `resource_event` fact and filter. Both `SkillPoints` and
`SkillPointMaximum` retain the Skill Points address, and `resource_delta` on the
latter remains the maximum's delta, including when the balance is clamped.
A positive delta alone is still insufficient: the eventual native mode
assembly must require `BalanceChanged`, the Skill Points address and a positive
effective delta. Real generic battle commands verify that cap-only increases,
cap-induced clamps and overflow-only balance events cannot satisfy that
combination. This shared capability does not implement Excitation stacks,
consumption or Entanglement, and supplies no Curio execution credit. Its typed
filter is available to native catalog composition; no production Sora filter
column is authored for it yet.

The next mode assembly must bind the explicit Entanglement formula and duration
to this Curio's immutable policy, through normal `ApplyEffect` emissions. No
production Sora Entanglement payload or Control Resistance column is authored
yet. Excitation acquisition/consumption, the Additional packet, eligible roster
ownership and complete both-family battle/Activity replay remain pending.

## Validation and next runtime boundary

Native data fixtures verify all five operands, eligibility/reference joins,
policy-cap separation, fresh catalog reconstruction, malformed/noncanonical
values, cross-family key collisions and forged/missing/duplicate provenance.
The authoring verifier optionally re-reads and hashes the pinned Git objects.
Sora drift validation covers the schema, workbook, readers, binary and debug
export. The workbook's existing 44 sheets are preserved, with only four source
rows and the new definition sheet added; rendering checks the new policy cells.

These are **data checks**, not battle fixtures. Before runtime admission, both
Ordinary and Cyclical fixtures must execute real gain/consumption commands,
overflow and partial-team stack boundaries, multi-target damage and Crit,
cap-only changes without false gains,
Entanglement chance/resistance/delay/hit accumulation/delayed damage, refresh,
cleanse, provider defeat, wave carry, rejection and fresh reconstruction.
Forge admission, full-run release and terminal reference/program dispositions
remain pending. Catalog loading grants none of those credits.

```text
node tools/divergent-universe-runtime/verify-weighted-curio-excitation-authoring.mjs --check-source
node tools/divergent-universe-runtime/verify-decision-workbook.mjs
cargo test -p starclock-data weighted_curio_excitation
cargo test -p starclock-mode-universe weighted_curio_excitation_authoring
cargo test -p starclock-test-kit --test combat_suite resource_event
```
