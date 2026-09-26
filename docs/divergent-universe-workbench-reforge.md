# Accepted Workbench Blessing overwrite

## Executable boundary

`reforge_blessing_policy_accepted` executes a trusted owning service's accepted
Blessing pair instead of rejecting every `BuffReforge` invocation. It requires
the exact active current Workbench, its released function-2 membership, an
owned current input and a different unowned current output. Either input level
is legal; the output starts at level one. No same-quality restriction is
inferred for Blessings from the distinct Equation overwrite rule.

The shared Activity transaction commits final ownership, Equation progress
and expansion transitions, Cosmic Fragment debit, function receipt and any
current expansion rewards together. Expansion Curio rewards still use only
Reward RNG. Rejection, insufficient funds, malformed progress, overflow or
expansion failure leaves authoritative bytes, events and RNG unchanged.
Payment is not a second transaction after inventory mutation.

This is not an untrusted player-command interface. The accepted pair comes
from an owning executor, not a hidden original selector. It does not publish
or sample original three-candidate offers, admit an NPC to a source room, or
authorize arbitrary caller inventory changes. The fixed Respite menu currently
exposes enhancement only and does not call this API.

## Explicit replaceable price policy

`DivergentUniverseWorkbenchBlessingReforgePolicy::new(base, increment)` supplies
checked positive integer host inputs bounded by `i64`. The price is
`base + increment * successful_function_2_overwrites`, with checked multiply
and add. The existing Activity-lifetime service receipt map counts successful
overwrites across the Run. Workbench entry resets Heat but does not reset this
count. Failed attempts never increment it. This is
`VersionedProjectPolicyAcceptedBlessingReforgeLinearRunPrice`, not observed
numeric or scope parity.

The policy selects Cosmic Fragments. Current public tutorial text corroborates
that currency, while the production reference price row still explicitly says
`UnspecifiedCurrency`. Base price, increment and reset scope remain unavailable
in the pinned function row. No rarity schedule, discount, free-attempt modifier,
Curio overwrite limit or progression discount is silently applied. Those
consumers require their own authored integration before a complete service
can claim parity. This policy serves a controlled accepted-service boundary
without those pending modifiers.

Alternatives are rejecting all calls, guessing an old-mode price table, or
giving flat/free replacements. The chosen increasing checked price preserves
the released cost-direction fact without importing those numeric claims.
Confidence in numeric parity is absent. Replace this policy when current
released evidence supplies the base/increment or price program, reset scope
and modifier ordering; candidate selection has a separate replacement
condition. A profile using these inputs must include `configuration_digest()`
in its payload. That digest binds every numeric input and names the currency,
scope and formula policy. The accepted API itself does not invent a profile
or mutate an existing configuration identity.

No production row is authored here. Host prices use the existing accepted
service boundary. Static prices/selectors still require Excel/openpyxl/Sora.

## Evidence

Pinned released Version 4.4 repository:
`Dimbreath/turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568`,
inspected 2026-09-26.

- `ExcelOutput/RogueTournWorkbenchFunc.json`, FuncID 2 / `BuffReforge`, description
  hash `9244950205012458014`: a Blessing becomes another Blessing and cost rises
  with overwrite count. SHA-256:
  `b430ce650040a1b2cf5c262f8f69f8e9d15bf6632b81c4947408026e33363078`.
- `ExcelOutput/RogueTournWorkbench.json`, released `FuncList` membership;
  Workbench 101 includes function 2. This does not prove source NPC placement.
  SHA-256: `26053803e691fe1fa7be57bf94d1d766ff0cc3e6cebda579fa013b129775d736`.
- `TextMap/TextMapEN.json` and `TextMap/TextMapCHS.json`, the description hash
  above. Current digests are recorded in the
  [fixed Respite evidence](divergent-universe-respite-enhancement.md).
  Independent summary: 祝福覆写改变持有身份，费用随覆写次数增加。
- [Current public tutorial](https://honkai-star-rail.fandom.com/wiki/Tutorial/Simulated_Universe),
  accessed 2026-09-26: indexed public text corroborates Fragment payment and
  three random output choices. It contains historical-mode sections, is not
  immutable Version 4.4 pricing evidence, and supplies no accepted selector
  weights or full-page digest. This boundary does not implement those random
  choices. Historical Protean Hero prices and preview material are not used.

## Verification and remaining scope

The focused `workbench_reforge` tests cover price vectors, changed-input
digests, every current function-2-capable Workbench, enhanced-input teardown,
base-level output, cross-Workbench escalation and fresh reconstruction for
Ordinary/Cyclical. Invalid service, stale state, unowned/owned identities,
unknown identities, dirty progress, pending offers, insufficient funds and
overflow reject without mutation. A nearly satisfied
Equation exercises real expansion and Curio rewards; malformed expansion
charges reject twice without paying, consuming allowance or RNG.

Other transformations remain fail-closed. Original NPC admission, candidate
offers/weights, exact prices, modifiers, public room/controller dispatch and
encoded profile replay are incomplete. No source/mechanic obligation is
terminalized by this accepted-settlement boundary.
