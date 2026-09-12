# Divergent Universe fragment gains

## Current behavior

`CurioFragmentGains` is a typed Excel/Sora table in the decision authoring project.
It lowers four reviewed global gain components, separately from each Curio's
other effects. All current production fragment credits use the same mode-owned
operation generator: currency commands (including gamble coin gains), curse
chests, occurrence rewards, reviewed immediate Curio acquisition grants and
[base battle fragments](divergent-universe-battle-fragments.md).
The intrinsic 9071 [domain-entry grant](divergent-universe-curio-domain-expiry.md)
also uses this pipeline, before its own limiting-entry discard.
Spends, balance resets and Workbench Heat are not fragment gains.

| Mode-copy state | Effect | Handbook identity | Additional gain |
|---|---|---|---|
| 9055 | 2055 | 9055 | 50% |
| 9070 | 2070 | 9068 | 50% |
| 9079 | 2079 | 9068 | 30% |
| 9159 | 2159 | 9123 | 30% |

States 9070 and 9079 are mutually exclusive copies of one identity, not two
independently ownable Curios. Destroyed states do not contribute; repair restores
contribution, and replacement removes the old state's rate immediately. These
are live ownership reads, not a cached multiplier or activation marker.

## Evidence and policy

The unchanged released Version 4.4 source revision is
`fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
reviewed 2026-09-08. Authored sources 12–14 bind:

- `ExcelOutput/RogueTournMiracle.json`: the four state/effect joins above,
  `TournMode=Tourn3`; SHA-256
  `176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438`.
- `ExcelOutput/RogueMiracleEffect.json`: the four effect IDs and `ParamList[0]`;
  SHA-256 `fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35`.
- `TextMap/TextMapEN.json`: hashes 9113138745615695677, 7205940712072507551,
  3429135723207096768 and 4340701535028103385; SHA-256
  `afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789`.

Exact percentages and global gain applicability come from those bindings and
descriptions. Decimal strings lower to bounded integer fractions, never host
floats. The following independent uncertainties are explicitly covered by
`VersionedProjectPolicyActiveStateAdditiveOriginalBaseFloorEachBonus`:

- Snapshot the original nonnegative gain before adding any balance. Each active
  state's bonus uses that same base, never a previous bonus or updated balance.
- Add the base, then independently floor and add each bonus in stable state-key
  order. For active 50%, 50% and 30% components, a base of 3 produces 5, not 6.
- Accepted acquisition batches insert all holdings before their ordered effects;
  a gain modifier in the same batch is already active for those gains.
- Include positive credits, including any future refund routed through this
  path. Exclude spends and resets; do not turn losses into negative gains.

Stacking, rounding, acquisition activation timing and refund applicability have
low parity confidence. Alternatives include multiplicative stacking, summing
fractions before rounding, delayed activation and refund exclusion. Replace each
field independently when released execution data or reproducible observations
establish it. The current policy avoids recursive gains and keeps arithmetic and
rollback deterministic; it is not presented as an observed game formula.

## Transaction contract

Node-local slot 56 holds the original gain while its ordered operations execute,
then clears to zero in the same transaction. This temporary value is not an
inventory or a second economy state machine. Fixed and proportional acquisition
gains share the same pipeline; a proportional gain snapshots the current balance
before that gain, including earlier completed gains in the same batch.

Each addition requires sufficient `i64` headroom before mutation. Fractional
arithmetic uses quotient/remainder decomposition to avoid overflowing an
intermediate product when the result fits. A later bonus overflow rejects all
earlier gains, inventory changes, scratch writes, events and RNG draws through
the owning shared Activity transaction. No new native handler, recursive trigger
or content-ID branch is added to Activity or combat.

## Verification and remaining work

Tests cover each exact rate, fractional boundaries, large representable values,
additive/nonrecursive stacking, fresh reconstruction and input permutation,
destroy/repair/replacement, proportional snapshots and same-batch acquisition.
Overflow is tested after both base credit and a preceding Wax random reward.
Both production run families execute modified event and curse-chest gains;
curse-chest losses retain their original amount.

This implements gain components, not complete Curios. State 9055's suppression
now applies to the [normal-battle Blessing offer](divergent-universe-battle-blessings.md).
The [three/five-domain expiry of 9070/9079](divergent-universe-curio-domain-expiry.md)
now consumes active allowances at accepted future domain selections and discards
the holding before the limiting domain's rewards, under an explicit timing policy.
State 9159's price penalties and other battle reward programs remain pending. Base post-battle fragment credits now
execute under a separate fixed-domain policy; their original amounts remain
unverified. These components grant
no terminal source-obligation or mechanic-program disposition by themselves.
See [Goal 22](goals/22-divergent-universe-runtime.md) and the
[acquisition boundary](divergent-universe-curio-acquisition.md).
