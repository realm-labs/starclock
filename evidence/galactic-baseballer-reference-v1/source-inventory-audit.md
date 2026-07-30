# Goal 16 Focused Source Inventory Audit

`G16-P0-B2` freezes a reproducible discovery inventory for both released
Galactic Baseballer profiles. It does not yet freeze content membership or the
exact-once denominator; those belong to `G16-P0-B3`.

## Structured source closure

The isolated source cache is produced by:

```text
tools/galactic-baseballer-reference/fetch-sources.sh \
  .cache/galactic-baseballer-source \
  /Users/mikai/CLionProjects/starclock/.cache/content-reference
```

The tool copy-on-write clones the two clean fixed-revision seeds before setting
Goal 16 sparse paths. It does not alter the main checkout's sparse-checkout or
working tree.

The inventory contains 81 exact Git-blob receipts:

| Classification | Files | Meaning |
|---|---:|---|
| `departure-or-shared-candidate` | 41 | `EvolveBuild*` tables/programs that require profile reachability classification. |
| `demon-king-candidate` | 23 | `EvoBdSC*`, `EvolveBuildSC_*` and `_SC` sequel tables/programs. |
| `shared-closure-seed` | 10 | Stage, battle-event/target, MazeBuff, enemy and CHS/EN TextMap tables. |
| `identity-cross-check` | 7 | Pinned StarRailRes metadata, bilingual achievements and character indexes. |

The 64 mode-family files expose 1,653 top-level JSON rows/objects. Their 29
dedicated Excel-output tables contain:

- 697 `EvolveBuild*` rows across 14 original/shared table families;
- 831 `EvoBdSC*` rows across 15 Demon King table families.

The remaining candidate rows are the 35 ability/layout/character-program files
covering experience/level, weapons, accessories, strategies/cards, store,
new-player behavior, scaling, team bonuses, boss scoring, tutorials, extra
rules, Demon King behavior and special weapon actors.

Every receipt binds repository, fixed revision, source path, Git blob OID, byte
count, SHA-256, JSON shape, row count and first-row fields. Checked-out bytes
are independently re-hashed using Git's canonical blob framing before
acceptance.

Canonical inventory SHA-256:

```text
2430f3f2e3c117249defd3d3ec0f53d3aca0ba7a440063a437dfc522f9f47525
```

## Localization locator closure

The generator extracts exact unsigned 64-bit `Hash` references from every
candidate table/program without parsing them through JavaScript numbers. It
then reconciles those keys against the pinned Simplified Chinese and English
TextMaps.

The result contains:

- 1,739 unique candidate-owned hash locators;
- 1,510 referenced locators found in each locale;
- additional direct title/profile matches retained as discovery evidence;
- 3,403 total bilingual locator receipts.

Locator receipts contain only source identity, row locator, value byte count,
value SHA-256 and referring source paths. Bulk source prose is not committed.

Canonical localization-locator SHA-256:

```text
a1e084632a30733c74450b5dfdcac127e36c4dbdc51ccfbfbe1cf29d0eaea1ce
```

The 229 candidate hash locators not found in each selected TextMap are not
silently dropped. `G16-P0-B3` must classify them as nonlocalized, presentation,
obsolete, indirect or blocking before freezing the content denominator.

## Public released-source inventory

Five publisher-operated pages are registered:

- Version 2.2 update;
- original event notice;
- publisher-operated HoYoWiki original entry;
- Version 3.3 update;
- Version 3.4 released correction notice.

Eleven mechanical community pages are pinned by MediaWiki page ID, revision ID,
timestamp, MediaWiki SHA-1, byte count and SHA-256. They cover the root,
Adventure Index, planets, Cosmic Reputation and Cosmic Store for Departure,
plus the root, Adventure Index, Adventure Strategy, planets, Cosmic Reputation
and Cosmic Store for Demon King. Story and gallery pages are excluded.

Canonical public-source inventory SHA-256:

```text
8560c2be3d4c4c3f398f9dbed856ba03de3e7f7356f566661fabe2374b93b7fa
```

Community pages are cross-checks, not structured membership authority.
Publisher facts receive per-claim digests when normalized rows freeze in
`G16-P0-B3`.

## Verification

Executed commands:

```text
node tools/galactic-baseballer-reference/inventory.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/public-sources.mjs --refresh
node tools/galactic-baseballer-reference/verify-inventory.mjs \
  --source-cache .cache/galactic-baseballer-source
```

The verifier regenerated all structured receipts and localization locators
from fixed blobs and compared them byte-for-byte with the committed JSON. It
also validated the committed public-source receipt projection.

## Boundary

This inventory is deliberately broader than active membership:

- `EvolveBuild` does not automatically mean Departure-only;
- `_SC`/`EvoBdSC` identifies a sequel source family but not active reachability;
- shared StageConfig/enemy tables remain seeds until explicit stage references
  select rows;
- display names and ID adjacency never establish recipe or profile membership;
- account rewards, story, assets and presentation remain excluded.

`G16-P0-B3` owns exact profile selectors, row-level ownership, shared
reachability, exclusions and frozen denominators.
