# Divergent Universe Curio evolution

The production `CurioEvolutions` sheet defines four accepted transitions for
Green Miracle (`9195` → `9196` → `9197`) and
[Sage's Leaf Robe](divergent-universe-sage-acquisition.md) (`9192` → `9193` → `9194`). The private
loader validates adjacent Common/Rare/Legendary categories, unique predecessors
and successors, acyclic ownership continuity, an existing event locator,
reviewed successor acquisition effects, and provenance. Transport row order
does not determine the chain.

## Evidence and accuracy

Decision sources 4–6, 25–29 bind the released 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata)
and public cross-checks, accessed 2026-09-10. Source 25 records the exact
`RogueMiracleDisplay.json` name joins for displays 256–258, independently
distinguishing the three forms. Source 26 records handbook events 165/166,
Divine Treasures II/III, including variant/progress locators. Referenced NPC
graphs `RogueNPC627401.json` and `RogueNPC727401.json` are absent at that revision.

[Awakened](https://honkai-star-rail.fandom.com/wiki/Green_Miracle_%28Awakened%29)
and [Exalted](https://honkai-star-rail.fandom.com/wiki/Green_Miracle_%28Exalted%29)
community pages explicitly describe the consecutive upgrades in Divine
Treasures II and III, and explain their absence from the Gallery of Possibilities.
The separately digested BWIKI detail pages corroborate the related events:
[觉醒, revision 77066](https://wiki.biligame.com/sr/index.php?title=%E7%BB%BF%E5%A5%87%E8%BF%B9%EF%BC%88%E8%A7%89%E9%86%92%EF%BC%89&oldid=77066)
and [升华, revision 77072](https://wiki.biligame.com/sr/index.php?title=%E7%BB%BF%E5%A5%87%E8%BF%B9%EF%BC%88%E5%8D%87%E5%8D%8E%EF%BC%89&oldid=77072).
This is independent public evidence, not proof of current-profile placement.

The BWIKI Exalted page reports a conflicting 64-fragment full-HP reward. The
executable amount is **32**, from pinned effect 2197 `ParamList[1]`, supported
by the released description. Effect 2196 declares 16. The conflicting community
amount is recorded but never imported as runtime data. Source 29 records the
effect file digest and exact parameter/text locators; no bulk prose is committed.

## Accepted transition contract

`VersionedProjectPolicyActiveSameOwnerResetAndAcquire` applies the reviewed
sequence to current mode-copy identities. It provides a separate runtime owner
alias to Green Miracle's handbook identity `9155`, or the robe's `9154`;
it does not modify frozen source membership.
There remain 179 Curios, 235 states and 223 source handbook-bound states.
Four additional states have explicitly labeled runtime evolution-owner aliases.
The state API keeps `curio()` (source join) separate from `evolution_owner()`.
Neither alias is appended to the ordinary acquisition candidate list.

The trusted `evolve_accepted_state` boundary requires an active predecessor.
It removes the predecessor's status, charges and activation count, inserts
the successor as active with declared initial charges and zero activations,
and executes its full reviewed acquisition grant: 300 or 600 fragments for
Green Miracle, or two 2-star / three 3-star Blessings for the robe.
The new grant uses the same global fragment-gain pipeline as other credits.
Inventory, counters, grants, events and RNG commit through one shared generated
Activity transaction. Invalid, stale, repeated, skipped-tier, reverse, destroyed
or overflowing transitions cannot retain partial state. Direct acquisition and
generic replacement into evolution-only targets reject; existing ownership also
prevents reacquiring the base form alongside an evolved form.

The ordinary snapshot, destruction and repair read the current owned form.
Green Miracle's victory settlement also reads that form: old-form effects cease
and only the new full-HP reward applies. The robe's separate victory executor
also reads only the current active form; later public domain choices enable it
under the [explicit route policy](divergent-universe-layer-battles.md).
Destroying an evolved form suppresses it, without reinstating the old form.

Current-profile binding, active-only eligibility, counter reset and full reward
rather than difference-only credit remain independently replaceable policy
fields. Confidence in these hidden details is low. Alternatives include retained
counters, destroyed-state evolution and standalone variant ownership. Replace
each field when released execution topology or reproducible observations resolve
it; retain exact state/effect amounts independently.

## Verification and remaining production work

Tests exercise both run families, unique ownership, source/alias separation,
ordinary-pool exclusion, direct/generic bypass rejection, lifecycle gating,
counter reset, successive grants, duplicate/stale/reverse requests, snapshot
contributions, no extra RNG, and late bonus-overflow rollback. Verified actual
battle handoffs with explicitly counterfactual full-HP projections isolate the
16/32 victory rewards and destroyed-state suppression; these projections are
not claimed as naturally observed combat outcomes.

The accepted API itself is **not** a player-facing upgrade event. The separate
[Divine Treasures event binding](divergent-universe-evolution-events.md) now
authenticates public options, implements their costs/probabilities and rewards,
and records their selection replay for Green Miracle and Sage's Leaf Robe under
explicit layer-placement policy. Original victory-domain placement, other Divine
Treasure families and original room placement remain pending.
Event 165/166 locators alone do not authorize a player command. No obligation or mechanic program
becomes terminal solely from this transition implementation. Full completion
still follows [Goal 22](goals/22-divergent-universe-runtime.md).
