# Divergent Universe Sage's Leaf Robe victory rewards

`CurioVictoryBlessings` is a separate executable production sheet from Curio
acquisition. It defines Dormant/Awakened/Exalted states 9192/9193/9194, effects
2192/2193/2194, with respective victory counts 1/2/3. Every form draws from
**1–3-star Blessings**, not its acquisition-only rarity. Only Elite and
Aberration domains qualify; Combat and Boss do not.
The same table also includes [Tawot state 9072](divergent-universe-tawot-victory.md)
under a separate positive-domain-allowance policy. Its all-domain eligibility
and policy rarity pool do not change the Sage forms' source-backed restrictions.

## Evidence and executable policy

Sources 32 and 36–38 bind released 4.4 revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568`, inspected 2026-09-12.
`RogueMiracleEffect.json` `ParamList[1]` supplies victory counts, independently
from `ParamList[0]` acquisition effects. The pinned EN descriptions identify
`room_comp_type:2` and `room_comp_type:4` and the 1–3-star range. Their exact
display joins and digests are recorded in the [source contract](divergent-universe-sage-acquisition.md#separate-victory-contract)
and workbook. The shared room display table does not establish original
current-profile placement or extend the eligible set through adjacent rows.

`VersionedProjectPolicyBoundDomainUniformUnownedAvailableSubset` declares:

- Trigger per verified won battle with a mode-owned eligible domain binding.
  A source enemy's elite marker and the generic Encounter decision kind are
  not domain authorization. Only the current active owned form grants.
- Reuse the normal battle policy's authored suppression states. Under this
  explicit interaction policy, an active suppressor also blocks these immediate
  rewards; destruction disables suppression and repair restores it.
- Process grant states in stable identity order. Filter current unowned base
  Blessing identities by the authored rarity range; enhanced holdings also
  exclude their base identity. Draw with unit weights in stable identity order,
  without replacement across the full batch. Sealing-wax offer weights do not
  alter this immediate reward pool.
- Use labeled Reward RNG purpose 24051 once per granted identity. If too few
  remain, grant the available subset; an empty pool draws nothing and does not
  convert the reward into fragments or another rarity. Dirty Equation progress
  or an incompatible internal Blessing offer rejects settlement atomically.
- Commit final holdings and Equation refresh in stage 24050, after fragment
  victory grants (23700), which follow base domain fragments (24060), and before
  Curio battle lifetime settlement (24110) and normal candidate generation (23702).
  Later stages read the post-grant state, excluding every granted identity.
  Result verification, carry, all grants, all draws and destination advancement
  share the existing rollback boundary. Duplicate results cannot grant again.

Pool membership, weights, per-battle versus per-domain timing, state order,
suppression scope and exhaustion are independently replaceable low-confidence
fields. Alternatives include weighted rarities, fixed domain pools, duplicate
conversion, once-per-domain grants and suppression of base drops only. Replace
each when released graphs or reproducible observations resolve it; preserve
the exact known counts, rarity range and referenced domain labels.

## Verification and remaining scope

Real battle handoffs and verified result contracts exercise all three forms
under counterfactual Elite/Aberration context, with fresh-factory reconstruction.
Controlled inventory pools prove that every form can grant Common, Rare and
Legendary rewards, including rarities unlike its acquisition reward. Tests
cover partial/empty pools, exact RNG counts, post-grant normal offer exclusion,
late-stage rollback/retry, unique current-form ownership, destruction/repair,
replacement, active/destroyed/repaired suppression, loss and duplicate results.

The first production battle remains a Combat proxy: separate tests verify that
source elite flags do not enable any Sage reward for any form or run family.
Later [public domain choices](divergent-universe-layer-battles.md#authenticated-public-domains)
now produce eligible Elite/Aberration bindings under an explicit placement
policy. Public tests acquire Sage through the initial event, upgrade it, choose
Elite then Aberration, receive 2/3 immediate rewards after real battles and verify
fresh selection replay. Controlled domain substitution remains private to
fixtures; adapters can select only authored options, not override battle labels.
Original Elite/Aberration placement, complete room graphs and Blessing battle
effects remain required by [Goal 22](goals/22-divergent-universe-runtime.md).
No original mechanic program or obligation becomes terminal from this executor.
