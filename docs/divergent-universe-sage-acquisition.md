# Divergent Universe Sage's Leaf Robe acquisition

The decision workbook defines the three released acquisition effects and two
trusted evolution transitions for Sage's Leaf Robe. Six public Divine Treasures
options execute through the separate [event binding](divergent-universe-evolution-events.md).
The separate [victory reward executor](divergent-universe-sage-victory.md) is
implemented with public policy-bound Elite/Aberration choices; original room
placement and enemy programs remain pending.

| Form / state | Effect | Full acquisition reward | Runtime ownership |
|---|---|---|---|
| Dormant / 9192 | 2192 | One random 1-star Blessing | Source handbook 9154 |
| Awakened / 9193 | 2193 | Two random 2-star Blessings | Evolution-only alias to 9154 |
| Exalted / 9194 | 2194 | Three random 3-star Blessings | Evolution-only alias to 9154 |

## Evidence and policy

Sources 32–35 bind released Version 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-09-12. `RogueTournMiracle.json` binds the three `Tourn3` states to
effects and displays. `RogueMiracleEffect.json` supplies acquisition count from
`ParamList[0]`; its second parameter belongs to the separate victory effect.
`TextMapEN.json` description hashes 18402293723530033109,
6941038067972437758 and 1542688008986925663 establish the respective rarities.
`RogueMiracleDisplay.json` displays 253–255 bind the three named forms. Exact
file digests and locators are retained in the workbook, not bulk source text.

The counts and rarities are released facts. `FeasibleRarityAssignments` is an
explicit `VersionedProjectPolicy` for hidden membership, weights, order and
exhaustion. Its current pool uses unowned current Blessing identities, excluding
both base and enhanced holdings. State and Path keys determine request order.
Each mandatory reward gets a distinct identity across the complete Curio batch.

Path and rarity pools can overlap: a wax could otherwise consume the only
remaining common Blessing needed by the robe. Before drawing, a bounded
augmenting-path matching check proves a complete assignment exists. For each
request, candidates that prevent the remaining assignment are excluded; one
unit-weight candidate is then drawn in stable identity order using shared Reward
RNG purpose 23651. This is sequential feasibility-conditioned sampling, **not**
a claim of uniform probability over complete assignments. It does not add RNG
for matching or introduce a second Activity state machine.

An impossible batch, pending Blessing offer or dirty Equation progress rejects
without state/RNG changes. Rewards are never reduced or changed to another rarity.
The same single-Curio preflight informs public Curio pool eligibility and offer
predicates. Joint batches are revalidated before their acquisition draws. The
generated transaction owns inventory, rewards, Equation refresh, fragment effects
and rollback. Each Curio's complete grant refreshes Equation progress before
later effects; sampling does not mutate a shadow Activity.

The two [accepted evolution edges](divergent-universe-curio-evolutions.md) follow
the named tiers under explicit current-profile policy. Targets remain absent
from source handbook joins and ordinary reward pools. Upgrading replaces the
active predecessor and counters, then grants the full new-state reward, not
the difference. Exhaustion leaves the predecessor intact. Public event identity
locators 165/166 do not themselves authorize these trusted commands.

Replace pool membership, weights, feasibility conditioning, exhaustion,
profile/event binding, alias ownership, counter reset and full versus incremental
grants independently when released graphs or reproducible observations settle
them. Hidden distribution and scheduling confidence is low. Alternatives include
weighted or engine-ordered draws, duplicate conversion, retained counters and
difference-only rewards. Exact count/rarity facts remain separate from policy.

## Verification and remaining work

### Separate victory contract

The same released effect rows bind `ParamList[1]` to victory counts 1, 2 and 3
for Dormant, Awakened and Exalted respectively. Unlike acquisition rewards,
**every form's victory pool spans 1–3 stars**. The descriptions require victory
in `room_comp_type:2` or `room_comp_type:4`; neither normal Combat nor Boss is
included. Do not reuse each form's acquisition rarity as its victory filter.

The pinned revision's `ExcelOutput/RoguePersonaRoomCompType.json`, inspected
2026-09-12, has SHA-256
`c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5`.
Its current obfuscated field `LLICIMBCNPF` is the row locator:
row 2 has `LHLKJIDFLIN=Elite`, display hash 2832393198030600814;
row 4 has `LHLKJIDFLIN=Encounter`, display hash 2381374495552649823.
The same pinned EN TextMap resolves these names to Elite and Aberration.
Rows 3/Battle and 1/Boss identify the excluded Combat and Boss labels. These
are exact referenced label joins, not proof of current-profile room placement,
enemy membership or original encounter topology. No additional room mechanics
are inferred from this shared table.

The baseline first battle is Combat, and later battles use public domain choices,
even though all selected source stages carry elite markers. Victory grants
use an authenticated mode-owned domain binding, never infer eligibility
from source stage flags, an adapter-supplied label or the graph's generic
Encounter decision kind. The shared staged settlement now permits immediate
Blessing grants to update holdings before normal reward candidates are drawn;
the [victory contract](divergent-universe-sage-victory.md) now lowers these grants
and tests positive effects both in controlled fixtures and publicly selected
Elite/Aberration contexts with fresh replay. Original domain placement remains
pending. Pool membership/weights,
exhaustion, active-state gating and suppression interactions are executable
replaceable policies, not observed parity.

### Acquisition verification

Tests execute all three full grants and both transitions in each run family,
verify exact rarity counts and six draws, preserve one ownership identity, and
reconstruct identical state. Scarce-pool cases exercise feasible and impossible
wax/robe overlap, input permutation, upgraded-pool exhaustion and late fragment
overflow rollback. Matching is cross-checked against all three-candidate request
and occupied-set combinations. Public tests obtain the base robe through the
actual initial event, finish three real proxy battles and verify fresh selection
replay. That initial placement remains the existing explicit event-pool policy,
not evidence of the original Divine Treasures I producer.

Original event topology, elite/aberration placement and enemy programs,
complete Blessing battle effects and current-profile reachability remain required
by [Goal 22](goals/22-divergent-universe-runtime.md). No original obligation or
mechanic program becomes terminal from this partial Curio implementation.
