# Divergent Universe domain-deck programs

## Evidence and policy boundary

The [official Arcadian Chronicles announcement](https://www.hoyolab.com/article/44302656)
describes a draw pile, selection from drawn domains, discard of the displayed
domains, and recycling the discard pile after exhaustion. Reviewed 2026-09-12
for the current Version 4.4 implementation; this public description does not
establish the pinned build's hidden RNG, initial mask decks or fixed-room selector.

The base compiler uses explicit replaceable policies: uniform integer sampling
over sorted stable card-instance IDs; draw width supplied by the owner (1–5);
a short remaining pile produces a short hand without topping up; refill occurs
at the next draw preparation. There is no shuffle RNG on refill itself: uniform
sampling without replacement owns the random order. Replace these policies if
released selectors or reproducible observations establish different behavior.
No mask, beacon, initial deck or room-content membership is inferred here.

## Execution boundary

`domain_deck::DomainDeck` compiles immutable card instances into shared Activity
slot definitions, preparation/offer programs and a Graph-labeled random offer.
Copies of a domain definition need distinct nonzero instance IDs. The owner must
bind both the authored content definitions and these programs into its graph;
the compiler does not load source JSON, create a second state machine or mutate
an Activity outside accepted commands.

Draw/discard piles and the selected instance persist across sections. The hand
is the authenticated pending Route offer. Internally the unsettled draw slot
still contains the reserved hand until selection commits; `observe` subtracts
that hand from the available draw pile, without mutation or RNG consumption.
The three observable piles are disjoint and conserve every card instance.
The exact slot declarations (including the node-reset guard) are checked before
observation or selection. Selecting one card atomically removes
every displayed card from draw, adds all of them to discard, records the chosen
instance, and traverses into the owning room executor. A node-reset acceptance
guard prevents raw generic selection from bypassing that settlement. Downstream
rejection restores the offer, authoritative state and RNG. A complete exhausted
deck refills before the next draw; malformed or unsupported card mutations are
not silently repaired. Fixed-room nodes can bypass draw preparation entirely.

The compiler currently handles the closed base deck. Redraw, deletion/addition,
retention, beacon mutation, mask-specific replacement and card-level changes
need explicit extensions, not direct edits of these slot sets.

## Current verification and remaining integration

Shared-Activity tests execute six draw/selection phases across three sections,
check whole-hand discard, short hands, recycling and exact fresh reconstruction,
and reject raw, stale, unoffered, foreign-policy and downstream-failing commands.
Additional cases cover malformed partitions and wrong slot scopes. Fixed-room
reward offers have no domain hand and consume no Graph RNG while entering the
room; the deck resumes on the next draw preparation. Configuration identity
binds the whole graph, so identical master seeds on different graphs do not
promise identical hands.
These are compiler/transaction tests, not a production playable-run claim.

Run the focused behavior checks with:

```text
cargo test -p starclock-mode-universe domain_deck
```

The production Ordinary/Cyclical baseline is not yet bound to this compiler.
Initial mask/deck authoring, the current 60-position layout, actual room payloads,
fixed-room behavior and public replay/adapter commands remain required before
domain-deck or full-run release credit. The existing three-battle proxy is not
treated as completion of those requirements.
