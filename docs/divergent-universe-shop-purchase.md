# Divergent Universe fixed-stock purchases and bound shop menus

## Current executable contract

`shop_purchase::ShopPurchaseRuntime` compiles explicitly selected current Sora
Blessing identities and ordinary Curio states into immutable stock. The owning
service supplies a positive integer Cosmic Fragment price for each item and a
host slot at or above 70. There is no original merchant/pool/price inference.
`ShopItemId` is a Starclock stock address in 1..=64, not an upstream locator.
Items are canonically ordered, owner-unique and bounded to 64. Unknown rewards,
duplicate addresses/owners, unbound/evolution-only or negative Curios and invalid
prices reject before a runtime is produced.

`purchase_accepted` is a **trusted owning-service boundary**, not an untrusted
player command or authenticated public menu. The caller must prove the active
shop and offered item, use the same factory and bind the stock digest into its
immutable profile identity. The method itself checks the expected state hash,
exact host declaration, an active logical room, current sold-out state, funds
and acquisition legality. Completed activities and pending battles reject.
It does not establish NPC/profile affinity on the caller's behalf.

One shared generated Activity transaction commits, in order:

1. Exact fixed-price Cosmic Fragment debit.
2. The existing complete acquisition plan for one base-level Blessing or one
   ordinary Curio, including authored immediate rewards and Equation expansion.
3. The logical-room sold-out mark for that item.
4. The independent Run-wide shop purchase receipt (`0x2263_0001`).

Eligibility and acquisition/expansion planning use the pre-payment view.
Fragment-reward expressions execute after debit and inventory insertion, using
the resulting balance and modifiers. The purchase does not call a nested live
acquisition mutation, strip mandatory rewards or run a separate RNG/state machine.
Any generation, reward, arithmetic, receipt or operation failure restores
canonical state, inventory, balance, sold-out marks, events and RNG. Rejection
before acquisition planning consumes no RNG.

The purchased-items map is Player-visible, logical-Node scoped, with at most 64
entries of value one. It survives physical menu moves in the same logical room,
and resets only at a new logical room. Run receipts persist. Existing ownership,
including destroyed/evolved Curio owners, still prevents purchasing another copy;
a shop does not repair, replace or enhance a held reward.

The configuration digest binds exact current component/decision inputs, stock
addresses, reward kinds/identities, prices, slot address, bounds, program/receipt
IDs and the explicit snapshot policy. Input vector order is not significant.
Owning profiles must also bind placement, topology and all other immutable inputs.

## Bound public room menu

`shop_purchase::room::ShopRoomCompiler` supplies a public Shop decision at an
exact current reviewed `Card(Shop)` context. The factory validates the context's
source position and preset role; this does not prove an original NPC, stock pool
or released profile membership. Stock and prices remain explicit caller policy.
Other room kinds and fabricated contexts reject. Hosts use
`compile_curio_domain_route`, include the two exact host declarations and bind
the room digest into their complete immutable profile identity.

The room has one entry and one regenerated menu, using only the shared Activity
graph. The Curio domain-entry prefix runs once; self-menu transitions do not
re-enter it. The menu is bounded to stock count plus one visits. Sold-out marks
survive physical menu moves in the same logical room. No refresh, sampling,
reroll, discount or duplicate-owner repair service is added.

Admission hides sold-out items, insufficient funds and already-owned rewards,
including destroyed/evolved Curio owners. Blessing rewards additionally require
a clean acquisition boundary and sufficient mandatory reward candidates. These
are deterministic eligibility predicates, not a guarantee that every later
Equation expansion, checked arithmetic or receipt operation will succeed.
Compilation and menu observation consume no RNG. Leave is always offered.

`CompiledShopRoom::bind` authenticates exact nodes, entry/menu programs, edges,
declarations and Run/Plane/Node logical paths. Extra outgoing edges, entry bypass,
missing Curio prefixes and injected room random policies reject.
`BoundShopRoom` then authenticates the whole running definition, including
participants, programs outside the fragment, bootstrap and handler bindings.
Fresh structurally identical definitions are accepted; identical claimed IDs
alone are insufficient.

All raw shared choices, including Leave, are gated. The bound `choose` method
authenticates the offered ID before reward generation and reuses the full
purchase plan. Payment, mandatory rewards, sold-out state, receipt and regenerated
menu commit in one generated-choice transaction. Leave and downstream entry also
share one transaction. Stale, hidden or foreign choices and late failures preserve
canonical bytes and RNG. A trusted external mutation does not rewrite an old
offered-ID snapshot: the purchase plan revalidates current eligibility, and the
next real menu transition recomputes admission.

Room accuracy:
`VersionedProjectPolicyExplicitStockAtReviewedShopCardsOnePurchasePerItemNoRefresh`.
It complements, rather than promotes, the fixed-stock purchase policy below.

## Position-profile and controller dispatch

`bind_position_shop_rooms` attaches compiled Shop capabilities to an immutable
position battle profile. It checks current factory inputs, area/layer placement,
distinct room nodes and each exact fragment against the full definition.
The profile owner must already include every stock/price/room digest in its
payload. Empty, duplicate, repeated, foreign, changed-price and changed-slot
attachments reject; attachment order does not change identity or initial state.

`DivergentUniverseFlowInstance::offered_shop` observes the authenticated current
menu. `choose_shop_option` uses its bound generated-choice transaction for both
purchases and Leave. The baseline controller dispatches scored Shop decisions
through this API. Missing capabilities cannot execute the gated raw menu.
Observation and failed selections do not grant rewards or advance RNG.

Production-catalog integration fixtures for Ordinary and Cyclical enter Shops
through actual sampled source-card hands, using a deterministic controller hint
that prefers Shop cards only when offered. They buy the three explicit test
items and compare exact payment/reward/receipt behavior and Leave. Each run
completes three real nested Boss proxy battles, including battles after purchases.
Independently constructed factories produce
identical accepted steps and canonical state at every boundary. Insufficient-budget
menus still leave; receipt overflow after a wax reward draw restores state/RNG
and can be retried once the injected test fault is cleared.

The insufficient-budget case uses explicit high test prices because battles
before the first Shop can earn Fragments even from zero initial funds.
Stock, prices, initial funds, selected source deck/width and Boss proxies are
explicit fixture policies. Other room payloads remain probes. These integration
tests establish attached-profile execution, not default topology, encoded profile
replay, original merchant membership or complete-run release acceptance.

## Evidence and explicit replacement policy

Current production Sora catalogs establish the reward identities and existing
authored acquisition effects. They do not establish the proposed shop stock,
merchant eligibility, prices, discounts, refresh or per-room inventory limits.
This code adds no factual authoring row, Sora bundle change or source-mechanic
terminal disposition. Source-only merchant graphs/offers and shop-price records
remain independently pending; matching reward names/IDs is not membership proof.

Accuracy:
`VersionedProjectPolicyExplicitFixedStockAndPriceBaseRewardsPrePaymentEligibility`.

| Unavailable fact | Current deterministic policy | Alternative and rationale | Replacement condition |
| --- | --- | --- | --- |
| Merchant stock/weights | Explicit owner-unique stock, no sampling | Automatically reusing catalog pools would assert unproven membership | Released selector graph or reproducible merchant-stock observations |
| Prices/discounts | Positive caller-selected fixed Fragment price, no discount | Reference price-step shape alone does not prove item/merchant binding | Provenance-bearing item/merchant/price/modifier relationships |
| Stock refresh/quantity | One purchase per selected item per logical room, no refresh; at most 64 items | Infinite rebuy or automatic refresh changes cost/reward semantics | Released inventory/refresh/quantity rules or reproducible observations |
| Reward level/state | Base-level Blessing or ordinary nonnegative Curio; no duplicate-owner service | Implicit enhancement/repair is a different mechanic | Evidence for a distinct shop enhancement/repair/replacement action |
| Acquisition snapshot timing | Pre-payment eligibility/expansion plan, debit before generated reward operations | Post-payment eligibility would require a different checked planning snapshot | Released ordering evidence or a reproducible timing counterexample |

Original-parity confidence for these policy fields is unproven. Tests freeze
the stated replacement behavior, not original parity. Promotion must author
complete current Excel targets through openpyxl and Sora 0.6.1 with field-level
provenance; it must not silently replace designer-edited workbooks.

## Verification and remaining work

Production-catalog fixtures cover Ordinary and Cyclical with independently
constructed factories and identical canonical traces. They buy a base Blessing,
a wax that actually draws/grants a Blessing, and a Curio whose 40% Fragment grant
uses the post-payment balance. Prices and initial 250 Fragments are trusted
fixture setup, not recovered shop facts or merchant rewards.

Tests cover canonical stock ordering, price/slot digest sensitivity, invalid
counts/addresses/owners/rewards/prices, stale/unknown/insufficient-fund purchases,
already-owned/destroyed rewards, duplicate-owner mode copies, pending Blessing
offers (without skipping a bought Curio's mandatory reward), ordered payment/
reward/sold-out/receipt events, exact host declarations, corrupt stock, sold-out repeat
rejection, physical-menu persistence and logical-room reset. Receipt overflow
after a wax draw/payment/reward fails repeatedly without changing bytes/RNG;
clearing the injected fault allows exactly one purchase. Completed activities
reject without mutation. The isolated fixture graph supplies no full-run or
real-battle release evidence.

Bound-menu tests reconstruct identical Ordinary/Cyclical purchase and Leave
traces from fresh factories, exhaust all 64 stock addresses, check insufficient
funds and active/destroyed ownership, and hide exhausted mandatory Blessing grants.
Pending Blessing offers and dirty Equation progress reject stale eligibility
without draws. Regeneration preserves Activity-scoped dirty-progress exclusion;
existing physical-node-scoped Blessing offers reset at the menu transition.
Hostile context, price, program, slot, edge and logical-path changes reject;
an old capability also
rejects a foreign external program with unchanged claimed identity. Receipt
failure after reward planning preserves the current offer; downstream entry
failure restores scope, inventory and sold-out state. These isolated source-context
fixtures do not execute nested battles or complete a released profile.

Original merchant stock/weights, prices/modifiers, refresh, original NPC/profile
admission, default topology and encoded profile
replay remain incomplete. Other missing Curio effects are not inert or
completed by being buyable. This boundary earns no terminal source/mechanic,
Shop-family or complete-run coverage.
