# Divergent Universe Curio acquisition

## Current executable boundary

The typed `CurioAcquisitions` sheet in the decision workbook lowers reviewed
immediate fragment, Path and rarity-filtered Blessing effects. `DivergentUniverseCurioRuntime` appends these effects
to accepted inventory insertion, using the same operation generator for direct
trusted acquisition and production occurrence rewards. There is no source-ID
dispatch, upstream text interpreter or second Activity transaction.

| State suffix | Effect locator | Immediate grant | Ownership admission |
|---|---|---|---|
| 9028 | 2028 | 300 fragments | Current handbook identity |
| 9053 | 2053 | 40% of current fragments | Current handbook identity |
| 9167 | 2167 | 500 fragments | Current handbook identity 9131 |
| 9195 | 2195 | 150 fragments | Current handbook identity 9155 |
| 9196 | 2196 | 300 fragments | Evolution-only runtime owner alias; direct acquisition rejects |
| 9197 | 2197 | 600 fragments | Evolution-only runtime owner alias; direct acquisition rejects |

The eighteen state/effect/parameter joins are validated against the production
reference catalog. A known state and effect do not prove handbook ownership,
original reward-pool membership or an upgrade relationship. In particular, the
four source-unbound states are not added to the current reward pool. Their
reviewed [evolution transitions](divergent-universe-curio-evolutions.md) now
execute these grants through a separate trusted accepted boundary. Reference trigger
tags were derived by text matching and are not executable trigger definitions.

[Sage's Leaf Robe](divergent-universe-sage-acquisition.md) adds states
9192/9193/9194: one 1-star, two 2-star and three 3-star Blessings respectively.
The base state has source ownership; its upgrades have separate evolution aliases.

| Sealing Wax state | Effect locator | Immediate Blessing paths (one each) |
|---|---|---|
| 9043 | 2043 | Elation 126 |
| 9044 | 2044 | The Hunt 124 |
| 9045 | 2045 | Destruction 125 |
| 9046 | 2046 | Remembrance 121 |
| 9047 | 2047 | Nihility 122 |
| 9048 | 2048 | Propagation 127 |
| 9049 | 2049 | Erudition 128 |
| 9147 | 2147 | Harmony 129 |
| 9187 | 2187 | All eight current active paths above |

These are mode-copy states, not additional handbook identities. State 9147
belongs to handbook 9113 and state 9187 to handbook 9149. The eight single-Path
waxes have canonical Common states in the current event pool. Trailblaze Wax is
Legendary and is excluded by that event's Common/Rare policy, although the
trusted accepted-acquisition boundary can execute its reviewed grant.

## Evidence

Released Version 4.4 source revision:
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
reviewed 2026-09-08. The checked local source files are unchanged from that Git
revision. Exact row locators and provenance are authored in `Sources`; no bulk
upstream prose is committed.

| Source path | Exact locator | File SHA-256 |
|---|---|---|
| `ExcelOutput/RogueTournMiracle.json` | Eighteen reviewed `MiracleID` values, `TournMode=Tourn3` | `176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438` |
| `ExcelOutput/RogueMiracleEffect.json` | Eighteen reviewed `MiracleEffectID` values, `ParamList[0]` | `fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35` |
| `TextMap/TextMapEN.json` | Exact description hashes listed in the authored source row | `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789` |
| `ExcelOutput/RogueTournBuffType.json` | Eight path IDs above | `f9163c3414bf357c2a66d3a8c26b94e0ffda0a520a233af74d764f09a406c7f5` |
| `ExcelOutput/RogueTournUseBuffType.json` | `TournMode=Tourn3`, `UseBuffTypeList` | `506229616f84b385e61608d39541c1da0876e6fbe1b7775bc87f84f0849784f7` |

The exact effect amounts and immediate-acquisition timing come from these
bindings and descriptions. The parser preserves decimal strings; `0.4` becomes
the integer ratio `4/10`, never a host float. Data validation rejects a different
state, effect, parameter index, value or missing provenance/policy explanation.

## Explicit execution policy

`VersionedProjectPolicyStableStateOrderFloorBeforeEachGrant` resolves only
batch ordering, balance snapshot and fractional rounding:

- Validate all selected identities and inventory changes before committing.
- Insert the accepted batch, then run its reviewed immediate effects in stable
  state-key order, independent of input or random-selection order.
- Read the balance immediately before each grant. A prior grant in the same
  batch is visible to a later proportional grant.
- Floor nonnegative fractional fragments. Compute the quotient and remainder
  separately so a valid large balance does not overflow an intermediate product.
- Commit holdings, effects, consumed choice, events and RNG through the shared
  transaction. Duplicate, unbound or overflowing acquisitions cannot keep a
  partial reward; stale or repeated acquisition cannot grant again.

The source text does not settle simultaneous batch ordering, rounding or whether
a proportional grant snapshots before other acquisitions. Alternatives include
draw order, batch-start snapshots and other rounding modes. Stable state order
and sequential current-balance reads provide an explicit deterministic rule;
confidence in game parity is low for these fields. Replace each independently
when released execution topology or reproducible observations establish it.

`VersionedProjectPolicyStableStateOrderUniformUnownedPathRejectExhaustion`
governs the reviewed wax acquisition rewards. In stable state-key and path-key
order, sample current path-matching unowned identities uniformly across all
rarities, without replacement across the whole batch. Both base and enhanced
holdings are excluded. Reward RNG uses the shared labeled stream and purpose
23651. Coalesce the mandatory Blessing batch after stable-order fragment effects,
refresh Equation progress once, then run separately authored
[bounded expansion rewards](divergent-universe-equation-expansion.md) before room
completion. Merge any resulting discard into the command's final Curio holdings.
Other Blessing-acquisition triggers remain separate work.

Preflight a complete distinct assignment for every selected Curio before draws;
an exhausted path, pending Blessing offer or dirty Equation progress rejects the
entire acquisition. Event Curio pools and their offer predicates exclude waxes
whose mandatory rewards are unavailable. Rarity-filtered robe rewards use the
same gate. Mixed Path/rarity draws exclude candidates that would prevent the
remaining mandatory assignments; the linked robe contract records this explicit
distribution policy. Candidates without mandatory Blessing rewards remain eligible.
Hidden pool membership/rarity weights, ordering, exhaustion and pending-offer
scheduling are explicit low-confidence replaceable policies, not observed parity.
Alternatives include rarity weighting, duplicate/substitute rewards and queued
acquisition; released execution evidence or reproducible observations replace
each policy field independently. The path/count evidence is retained separately.

## Verification and remaining work

The acquisition fixtures cover exact amounts, fractional boundaries, a balance
above the direct-multiplication overflow threshold, input permutation, once-only
ownership, overflow rollback and rejection of unbound upgrades. The production
occurrence fixture exercises the actual offered choice with only two unowned
eligible identities, proving acquisition gains precede room completion. Its
preloaded inventory is a controlled test setup, not an account-entry feature.

Wax fixtures cover all nine states, cross-wax path competition, one remaining
candidate, exhaustion before RNG, stable input permutation and late fragment
overflow after random selection. Both production run families verify wax grants
precede room completion, and initial offer predicates agree with reward-pool
validation for exhausted inventories and dirty progress.

Reviewed fragment grants also use the shared
[fragment-gain pipeline](divergent-universe-fragment-gains.md); the amounts above
are base grants before active gain modifiers.

This is partial effect coverage. Other acquisition effects, additional fragment-gain
modifiers, replacement/upgrade triggers, domain debt/expiry and post-battle
income remain required. The eight single-Path waxes now influence normal-battle
offers through the separately authored [weight policy](divergent-universe-battle-blessings.md).
Trailblaze Wax's later rewards use the separate
[Equation-acquisition contract](divergent-universe-equation-grants.md). Absence from this
table is an implementation gap, not an inertness claim or an accepted no-effect
policy. No source obligation or
mechanic program becomes terminal from these rows alone. See the
[occurrence contract](divergent-universe-occurrence-rewards.md) and
[full runtime goal](goals/22-divergent-universe-runtime.md).
