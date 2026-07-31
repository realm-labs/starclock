# Goal 16 Demon King Progression and Store Audit

`G16-P2-B3` closes Version 4.4 Demon King growth, Adventure Strategy,
currency, treasure, reputation, Cosmic Store and mechanical unlock data. It
does not import account rewards or executable gameplay.

## Released-source cache

The isolated source reproducer now caches all 11 revision-pinned MediaWiki
pages by exact revision ID. Each response is checked against the MediaWiki
revision SHA-1 and the inventory's SHA-256, then reproduced in offline mode.
These public pages are only identity/effect cross-checks:

| Page | Revision | Use |
|---|---:|---|
| Adventure Strategy | `452747` | released strategy identity/effect cross-check |
| Cosmic Reputation | `324128` | lower-quality 20-rank cost cross-check |
| Cosmic Store | `432557` | released store identity/effect cross-check |

The pinned Version 4.4 `turnbasedgamedata` rows and structural programs remain
the primary evidence. No preview, beta or unreleased source is admitted.

## Battle growth and Adventure Strategies

The Demon King profile retains the exact level threshold `40`, wave multiplier
`0.27`, level parameters `0.14/1/12`, and enemy experience values
`2/4/8/0` for the four published enemy classes. The special-source experience
value remains explicitly unspecified.

All 56 Adventure Strategies resolve to one exact level-one MazeBuff and one
structural program binding:

| Source type | Strategies |
|---|---:|
| General | 15 |
| Growth | 22 |
| Power | 18 |
| Demon King | 1 |
| **Total** | **56** |

The source candidate vector
`18,6,3,3,7,6,2,0,2,0,7,7,7,7,7`, three refreshes, two exclusions and one
strategy reroll are retained exactly. The source does not expose the mapping
from the 15 vector positions to candidate classes. ReferenceOnly selection
therefore uses a labeled integer RNG stream and stable Starclock strategy-ID
order under an explicit low-confidence ProjectPolicy; it does not claim
observed parity.

Standard battles expose weapon slots `4/5` and accessory slots `4/6`
(initial/maximum). Origin stages expose `3/3` and `4/4` respectively. Five
ordered inventory-operation rows cover duplicate acquisition, maximum/full
rejection, expansion and rejected-operation byte identity.

## Persistent currencies and reputation

Two profile-owned currency records are explicit:

- Raccoon Gold is item `281027`, capped at `500000`, with exact normal-1,
  normal-2, elite and boss income values `5/5/20/200`.
- Cosmic Reputation is Offering type `8`, capped at `500000`, with 20 retained
  ranks.

The 20 reputation costs use the pinned public revision at deliberately lower
evidence quality: `3000` for ranks 1–5, `4500` for ranks 6–10 and `6000` for
ranks 11–20, totaling `97500`. No structured Offering table for these costs is
present in the frozen family, so the row remains
`ApproximateFromReleasedText` and replaceable.

All account rewards are excluded. Rank/reward and tutorial rows are preserved
only as counted locators needed to distinguish mechanical unlocks from Stellar
Jade, materials, character acquisition, avatars, achievements and other
account presentation.

## Demon King's Treasure

Five exact treasure groups refer to ten exact candidate pools. Every pool
retains all ten authored entry positions, for 100 positions total; duplicate
stable IDs remain separate source facts.

The released tables do not expose selection weights, group timing or
duplicate-entry semantics. ReferenceOnly review therefore samples eligible
entry ordinals with a labeled integer RNG stream. Deduplication, inferred
positional weights and generic shuffle were rejected and recorded with a
released-logic replacement condition.

The exact chest probability vectors are retained, but their public ordinal
meaning is not guessed. That boundary is separately marked low-confidence
ProjectPolicy.

## Cosmic Store

The 16 exact store definitions expand to 60 price levels:

| Effect kind | Definitions | Price levels |
|---|---:|---:|
| Add MazeBuff | 14 | 54 |
| Initial weapon level | 1 | 5 |
| Add accessory slot | 1 | 1 |
| **Total** | **16** | **60** |

The complete purchase cost is `75600` Raccoon Gold. All 54 MazeBuff-linked
levels retain exact parameter vectors, and all four source tag rows are
preserved.

The published tables do not define rejection event ordering. The ReferenceOnly
transaction boundary validates current level and balance before committing
currency deduction, level advance and effect as one ordered operation.
Insufficient balance, wrong current level and maximum level reject without
mutation. This is an explicit low-confidence policy, not observed parity.

## Unlocks and semantic review

Ten exact constant unlock rows and 20 tutorial locators are retained. Only
mechanical unlock edges enter the normalized reference model; tutorial text
and reward payloads remain presentation/evidence locators.

Two ReferenceOnly rules and six concrete fixtures close these mechanism
families:

- Adventure Strategy acquisition, including `Surprise Windfall`;
- successful and rejected Cosmic Store transactions;
- Demon King's Treasure selection;
- Cosmic Reputation rank 20;
- the exact chest probability vector.

Four approximation records cover reputation costs, chest-vector ordinal
meaning, treasure selection and store atomicity. Each names the unavailable
fact, selected deterministic policy, at least two rejected alternatives,
rationale, affected fixtures, confidence and replacement condition.

Across Demon King arsenal, strategies and store upgrades, 308 of 315 exact
MazeBuff rows are now owned: 198 gear levels, 56 strategies and 54 store
levels. The remaining seven are stage team bonuses owned by `G16-P2-B4`.

## Reproduction

```text
node tools/galactic-baseballer-reference/cache-public-revisions.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/cache-public-revisions.mjs \
  --offline --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/normalize-demon-growth-strategies.mjs \
  --check --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/normalize-demon-progression.mjs \
  --check --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/normalize-demon-progression-fixtures.mjs \
  --check
node tools/galactic-baseballer-reference/verify-demon-progression.mjs \
  --source-cache .cache/galactic-baseballer-source
```

P3-B1 will merge these fragments into the contracted growth, strategy,
progression, rule, fixture and approximation tables.
