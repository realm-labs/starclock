# Divergent Universe fixed-stock purchase boundary

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

Public shop menus, Flow/controller dispatch, original merchant stock/weights,
prices/modifiers, refresh, source-position admission, default topology and encoded
profile replay remain incomplete. Other missing Curio effects are not inert or
completed by being buyable. This boundary earns no terminal source/mechanic,
Shop-family or complete-run coverage.
