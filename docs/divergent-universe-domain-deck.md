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
No released mask-offer membership, beacon effect or room-content eligibility
is inferred from the sampling algorithm.

## Authored source decks

The production decision bundle now includes `DomainDecks` and `DomainCards`:
nine **explicitly caller-selected** source decks, 125 distinct Starclock card
instances and 15 referenced presets. Source order and duplicate presets are
retained, including levels 1 and 2. `compile_domain_deck(key, width, slots)` on
the production factory compiles one named definition into the same shared
Activity programs described below; unknown keys reject. The decision digest
binds the complete authored bundle. The caller must use that identity when
assembling the owning graph and resolve selected instances through the catalog.

The released 4.4 source revision is
`fd978d6ef09f941fba644c731ab54abd6f7c3568`, reviewed 2026-09-19.
Sources 67–69 retain repository, revision, locator and byte digest:

| File under `ExcelOutput/` | SHA-256 |
| --- | --- |
| `RoguePersonaStyle.json` | `49b1c63d1f9dcad5201ae6f6446c01b9f401506b5625c39fc4da4b62378c6834` |
| `RoguePersonaRoomPreset.json` | `a4cc8bbd6e3a4db5fecb62b83a0c5de7ad2f4a9820ad5ae9180b30ae33c346b5` |
| `RoguePersonaRoomCompType.json` | `c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5` |

The explicit source selections are styles 101–108 and 110. Their
`JEHDKAKMCGC` values join preset `LIIPLGLNPGB`; composition `LLICIMBCNPF`
joins the published type enum, and `AAGKEBFHLMC` supplies the interpreted level.
All selected presets have empty `FJIKMHCJMKH` attribute lists. Stable deck/card
keys and instance IDs are Starclock identities; style/preset numbers remain
source locators. A duplicated preset is never deduplicated into one card.

`VersionedProjectPolicyExplicitSourceDeck` interprets the source list as an
initial deck. The joins and numbers are reproduced exactly, but that field
role and caller-selected admission are not proven released runtime selectors.
An alternative is that these are preview/default records rather than the
actual entry-selected deck. Confidence is high for values and joins, limited
for selector semantics. Named released selectors or reproducible observations
replace this interpretation. Style 901 is not selected by this policy; this
does not terminally exclude it. Mask unlocks, initial random offers, effects,
pass costs, redraws and room execution remain separate requirements.

The owning author uses openpyxl and refuses an existing output workbook. The
source check below validates exact bytes, list order/multiplicity and joins;
fresh-authoring/Sora drift checks validate production export reproduction.

```text
.cache/g22-python/Scripts/python.exe tools/divergent-universe-runtime/domain_deck_rows.py --verify-source .cache/content-reference/turnbasedgamedata
```

This policy does not promote the reference manifest's Persona rows or award
terminal runtime credit. The current source-audit dispositions remain intact.

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
An additional runtime test loads all nine decks through the production Sora
factory and executes six draw/selection phases for each, preserving instance
identity, card conservation and exact fresh reconstruction. Data tests reject
missing cards/decks, wrong counts or order, orphan ownership, invalid preset
levels, malformed identities and unbound provenance.

Run the focused behavior checks with:

```text
cargo test -p starclock-mode-universe domain_deck
```

The production Ordinary/Cyclical baseline is not yet bound to this compiler.
Automatic mask/deck selection, the current 60-position layout, actual room payloads,
fixed-room behavior and public replay/adapter commands remain required before
domain-deck or full-run release credit. The existing three-battle proxy is not
treated as completion of those requirements.
