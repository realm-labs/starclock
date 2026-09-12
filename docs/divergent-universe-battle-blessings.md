# Divergent Universe normal-battle Blessing rewards

## Current executable scope

Both baseline run families now settle their real, explicitly selected proxy
battle into a Blessing reward node before continuing to the next logical layer.
The mode uses the shared generated pre-offer settlement boundary; it does not
add a second graph executor or RNG implementation. A lost or faulted battle
does not generate a victory reward. Each default logical layer now executes a
real [policy-bound proxy battle](divergent-universe-layer-battles.md). Full room topology,
original domain-specific base reward programs and complete-run parity remain
pending. The public Combat/Elite/Aberration route deliberately reuses this
provisional Common/Rare base pool; separately authored domain-gated Curio rewards
execute before it. This is not an exact Elite/Aberration base drop table.

The retained first Ordinary vertical-slice entry uses the same settlement and
selection path. Its curated inventory transitions now refresh the real Equation
main/sub counts, expanded set and Blessing snapshot atomically, rather than a
handwritten progress counter. This keeps its state valid for ordinary rewards.

The openpyxl-authored `BattleBlessings` sheet in
`config/divergent-universe-decisions/data/DivergentUniverseDecisions.xlsx`
owns the current reward policy. Sora 0.6.1 validates and exports it together
with the executable decision bundle. Its schema and binary enter the existing
Activity configuration identity and fresh replay component identity.

## Evidence and replaceable policy

Released structured locators use Version 4.4 transcription at
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
revision `fd978d6ef09f941fba644c731ab54abd6f7c3568`, inspected 2026-09-08.
`ExcelOutput/RogueTournBuff.json` supplies base-level identities and categories;
its Git-blob SHA-256 is
`4bcf0b062941400a7aa4a530b57a5e9e42e4a0a1d8b4a67e90c7886214c27cb7`.
Catalog inclusion alone does not establish battle-pool membership.

State 9055 explicitly binds effect 2055. The released English description,
TextMap hash `9113138745615695677`, prevents Blessing gains after combat.
The exact state, effect and text blob digests are retained in decision source
rows 12–14 and the [fragment-gain contract](divergent-universe-fragment-gains.md).
This is a negative reward effect, not a combat-stat modifier.

The public [BWIKI overview](https://wiki.biligame.com/sr/%E5%B7%AE%E5%88%86%E5%AE%87%E5%AE%99)
and [Inspiration Circuit description](https://honkai-star-rail.fandom.com/wiki/Divergent_Universe/Inspiration_Circuit)
also describe selecting Blessings after victory. Their indexed text was reviewed
2026-09-08; no full-page digest or current-version distribution is claimed.
The retained client tables inspected here did not establish the complete normal
battle reward program. Older Simulated Universe guides and nearby reward tables
are not promoted to current Divergent Universe membership evidence.

`VersionedProjectPolicyUniformUnownedSingleSelectionAvailableSubset` declares:

- Sample at most three distinct, unowned Common/Rare base identities from the
  current catalog in stable identity order. Each has base integer weight one;
  separately authored active Curio bonuses modify matching candidates.
- Present the sampled candidates in ascending numeric state-key order. Public
  option ordinals select the corresponding candidate, exactly one per battle.
- If fewer than three remain, offer the available subset. An empty pool advances
  without a Blessing; it does not mint fragments or another substitute reward.
- Snapshot the reviewed suppression state at verified settlement. Active status
  suppresses this offer; absent, destroyed or replaced status does not. Repair
  restores suppression. No RNG is drawn for suppression or an empty pool.
- Bind this policy to the current explicit baseline normal-battle route in both
  run families. This binding does not prove physical room or encounter membership.

Width, membership, weights, exhausted-pool behavior and suppression timing are
independently replaceable when released execution data or reproducible current
observations establish them. Base fragment drops, extra selections, rerolls,
base Path weighting and other reward/acquisition triggers remain implementation
gaps, not deliberately zeroed mechanics.

The separately authored [Dormant Green Miracle victory grant](divergent-universe-curio-battle-grants.md)
now runs in this settlement transaction. It is not a base fragment drop and is
not suppressed merely because the Blessing offer is suppressed.
The configuration-bound stage order is base fragments (24060), reviewed Curio
grants (23700), domain-gated Curio Blessings (24050), Curio battle lifetimes
(24110), then normal Blessing candidates (23702).
Candidate ownership, suppression and
weights read the fresh post-grant view, not the original post-carry snapshot.
Only after all stages does the destination program publish a decision. Any
later-stage or destination failure restores all stages and RNG. This
ordering is a replaceable mode policy, not evidence of original reward timing;
Sage's Leaf Robe's [victory executor](divergent-universe-sage-victory.md) runs at
its stage, but current Combat proxy bindings intentionally grant nothing from it.
Original positive Elite/Aberration producers remain pending.

### Sealing Wax Path weights

The `CurioBattleWeights` sheet has eight exact state/effect/Path joins, matching
the single-Path waxes in the [acquisition contract](divergent-universe-curio-acquisition.md).
Decision sources 7–11 and 15 retain the released 4.4 locators and file digests.
The bound descriptions explicitly increase the matching Path's post-battle
appearance chance but do not supply a coefficient. Trailblaze Wax instead
triggers on Equation acquisition, so it is not included here.

`VersionedProjectPolicyActiveStateAdditiveCandidateWeight` adds three integer
weight units per active matching state to each eligible candidate's base one.
Multiple active paths retain independent bonuses. This four-to-one candidate
weight ratio is a low-confidence approximation, not an observed fourfold Path
probability or a guaranteed matching slot. Checked addition and the existing
weighted-without-replacement sampler own arithmetic and draws. Absent, destroyed
or replaced states add nothing; repairs restore the bonus. Suppression and empty
pools still consume no draws, and already published offers retain their snapshot.

Magnitude, additive stacking, base rarity weighting, applicable reward sources
and snapshot timing remain independently replaceable. Alternatives are
multiplicative bonuses and guaranteed Path slots; positive additive weights
preserve increased chance without inventing a guarantee. Released execution
data or reproducible current observations replace these policy fields. This
does not apply weights to occurrence grants, acquisition rewards or future shops.

Production tests compare actual battle offers to independently constructed
integer weight vectors for all eight waxes in both families, including destroy,
repair, replacement, multiple active paths and Trailblaze's exclusion. They
assert that active-weight fixtures actually differ from uniform sampling.

## Atomic state and public selection

Node-scoped slot 57 stores the sorted candidate IDs; slot 58 is the accepted
selection latch. Both reset on physical node transition. The reward node shares
the battle's logical Run/Plane/Node path, without inventing a new physical room.
The winning path budget includes both battle and reward nodes: layer count plus
four visits (layers, battle, reward, finalize and terminal). A nonbattle flow
retains layer count plus two. Reaching a deterministic fault is not accepted as
a completed baseline; fresh replay tests assert the `Completed` terminal too.

Result verification and carry settlement precede candidate generation. Candidate
generation and destination offer creation commit together. An invalid result or
any failed generated operation restores the awaiting battle and RNG. The outcome
receipt explicitly distinguishes a generated offer, no available offer and no
victory rewards; it does not declare other reward types implemented.

`DivergentUniverseFlowInstance::choose_battle_blessing` verifies the bound flow,
node, decision and option. The shared generated-choice transaction adds the
selected Blessing, refreshes Equation progress, sets the acceptance latch and
advances. Calling raw `GraphActivity::choose_option` cannot bypass that latch.
Rejected, stale, hidden or repeated selections leave state and RNG unchanged.

The baseline controller, Agent API, MCP and replay reconstruction use this same
public choice path. Slot 57 exposes only the sampled player-visible IDs, not the
private pool. The default baseline with no suppression now has eight accepted
actions and three actual layer battles; this remains a limited baseline, not a complete
Ordinary/Cyclical gameplay release.

Production tests cover both run families, all three selected alternatives,
canonical sampling, acquisition/Equation state, raw-command rejection, stale and
hidden choices, duplicate submission, exhausted subsets, suppression lifecycle
and fresh replay. These do not terminalize the outstanding source-program audit.
