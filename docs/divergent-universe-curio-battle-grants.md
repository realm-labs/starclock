# Divergent Universe Curio victory grants

The executable `CurioBattleGrants` worksheet binds Green Miracle (Dormant)
state `9195`, effect `2195`, handbook `9155`: after victory, grant eight
Cosmic Fragments for each qualifying full-HP character. Its immediate 150-fragment
[acquisition grant](divergent-universe-curio-acquisition.md) remains separate.
The public event's Curio option can produce this state, and the ordinary battle
settlement path executes the reward without another player command.

[Accepted evolution](divergent-universe-curio-evolutions.md) additionally enables
Awakened `9196`/effect `2196` and Exalted `9197`/effect `2197` rewards of 16 and 32
per qualifying character. Only the currently owned active form pays. Their
[public upgrade choices](divergent-universe-evolution-events.md) now execute
under explicit stable multi-treasure layer-placement policy; these forms are not ordinary
acquisition candidates. Source 29 binds the upgraded amounts at the same pinned
revision, reviewed 2026-09-10.

Sources 21–23 bind released 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
reviewed 2026-09-09:

- `ExcelOutput/RogueTournMiracle.json`, `MiracleID=9195`, current `Tourn3`
  state/effect/handbook joins; SHA-256
  `176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438`.
- `ExcelOutput/RogueMiracleEffect.json`, `MiracleEffectID=2195`, zero-based
  `ParamList[1]=8`; SHA-256
  `fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35`.
- `TextMap/TextMapEN.json`, description hash `1741960291225087902`, victory and
  full-HP trigger; SHA-256
  `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789`.

`VersionedProjectPolicyFullHpPresentRosterAfterCarry` makes the unavailable
scheduling details explicit. Count only current locked participants whose
verified carried state is Alive and Present, with positive maximum HP equal to
current HP. Snapshot after the shared battle-result carry step and before
additional mode healing. An absent, destroyed or replaced Curio does not grant;
repair restores the active grant. Loss/fault never triggers it.

Credit one aggregate amount through the [fragment-gain pipeline](divergent-universe-fragment-gains.md)
before publishing the Blessing offer. Active global gain bonuses apply once to
that aggregate; each bonus retains its independent floor rule. For example, four
qualifying characters produce a base 32; an active 30% global gain component adds
9, yielding 41, not four separately rounded gains of 10. Zero qualifying
characters creates no credit and consumes no extra RNG.
The separate [base battle credit](divergent-universe-battle-fragments.md) runs
first and is not included in that Curio-only amount. Its gain bonuses are
calculated from its own original base.

Result validation, participant carry, credit, Blessing RNG and next-node pumping
remain one shared transaction. Credit operations run in a separate bounded stage
before Blessing candidate generation reads a fresh post-credit view; both stage
IDs and this ordering bind mode configuration identity. Overflow restores the exact pending battle,
prior ledger, state, events and RNG; replayed results cannot pay twice. No new
state slot, random stream or parallel settlement executor is introduced.

Snapshot timing, off-field eligibility, aggregate versus per-character credit,
healing order and interactions with battle-only modifiers are independently
replaceable when released execution data or reproducible observations establish
them. Confidence in these hidden scheduling/stacking fields is low. Alternatives
include pre-carry or post-healing HP and per-character rounding.

Original base battle amounts, battle-specific gain multipliers/suppression and
original Occurrence placement remain unimplemented. A fixed-domain base credit
now executes as explicit policy. States `9196`/`9197` have accepted
transitions and public policy-bound producers, and are not added to the ordinary
acquisition pool. Source-program
audit obligations remain pending until their complete effects and producers are
accounted for.

Tests cover zero through four qualifying participants, damaged/reserved/defeated
exclusions, active/destroyed/repaired/replaced states, global bonus aggregation,
Blessing suppression, loss, duplicate results and late overflow rollback. Separate
public-producer runs acquire the Curio through the real event, earn a positive
reward in actual combat, and verify fresh replay for both run families.
