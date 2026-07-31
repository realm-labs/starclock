# Goal 16 Profile Membership and Denominator Audit

`G16-P0-B3` converts the fixed P0-B2 discovery inventory into a
non-shrinking exact-once denominator. It does not claim that every fact is
already normalized or DataReady.

## Versioned profile membership

Two independent profiles remain selectable at the Version 4.4 reference
baseline:

| Profile | Released edition | Dedicated source family | Rows |
|---|---:|---|---:|
| `galactic-baseballer.departure.v2_2` | 2.2 | `EvolveBuild*.json` | 697 |
| `galactic-baseballer.demon-king.v3_3` | 3.3 | `EvoBdSC*.json` | 831 |

The source families prove version-owned obligations, not cross-profile
identity. A matching name, adjacent ID or similar parameter never merges two
records. Demon King extends a reference-only shared system and does not replace
the Departure profile.

The manifest also retains all 35 selected configuration programs. Exact `SC`
and `S2` identifiers mark Demon King programs. Generic programs are modeled as
`SharedBase` under an explicit ProjectPolicy boundary; this is not presented as
observed parity.

## Exact denominator

`content-manifest.json` freezes 2,232 obligations across 29 categories:

| Group | Exact obligations |
|---|---:|
| Versioned profiles and all dedicated table rows | 1,530 |
| Configuration programs | 35 |
| Explicitly reachable shared stage/wave/enemy rows | 647 |
| Required semantic fixture families | 20 |
| **Total** | **2,232** |

Of these, 2,207 must eventually reach `DataReady`. The 21 reward rows and four
presentation/dialogue rows are retained in the denominator as `EvidenceOnly`
locators so later work cannot silently discard them or leak them into the
simulation core.

The active ownership reconciliation is:

| Ownership | Records |
|---|---:|
| Departure | 698 |
| Demon King | 845 |
| Shared base policy | 42 |
| Explicit shared stable-ID closure | 647 |

Every source record binds Version 4.4, repository revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568`, path, row locator, canonical
evidence SHA-256, quality, ownership, selector, disposition and note.
Sixteen-or-more-digit integer tokens are preserved as decimal strings before
canonical hashing.

## Explicit shared reachability

The only admitted shared stage/wave/enemy facts are reached through exact
fields:

```text
versioned StagePeriod.StageID
  -> StageConfig._StageInfiniteGroup
  -> StageInfiniteGroup.WaveIDList
  -> StageInfiniteWaveConfig.MonsterGroupIDList
  -> StageInfiniteMonsterGroup.MonsterList
  -> MonsterConfig.MonsterTemplateID / SkillList
  -> MonsterTemplateConfig / MonsterSkillConfig / MonsterStatusConfig
```

This closure contains:

| Shared family | Rows |
|---|---:|
| Stage configurations | 22 |
| Infinite-stage groups | 22 |
| Infinite waves | 74 |
| Infinite monster groups | 74 |
| Enemy variants | 88 |
| Enemy templates | 70 |
| Enemy skills | 287 |
| Enemy statuses | 10 |

`StageID=4140116` is explicitly referenced by both versioned StagePeriod
tables. This is source-backed shared reachability; no other cross-profile
content identity is inferred.

P0-B3 extends the isolated sparse cache with the three `StageInfinite*` tables.
Their Git blob OIDs, sizes and byte SHA-256 values are recorded under
`source_augmentation`. They do not modify the committed 81-file P0-B2
discovery inventory.

## Fail-closed boundaries

Twelve unresolved expansions remain explicit and non-blocking for this batch:

- three legacy `StageID` values (`3097`, `3098`, `3099`) occur in exact
  Departure StagePeriod rows but have no matching StageConfig row at the pinned
  revision;
- nine enemy skill `ExtraEffectIDList` values have no matching
  `MonsterStatusConfig.StatusID`.

The referring source rows remain counted. Only their unproven transitive
expansion is excluded. Each boundary records a replacement condition requiring
a released pinned mapping; later work may not guess from IDs or names.

## Reproduction

```text
tools/galactic-baseballer-reference/fetch-sources.sh \
  .cache/galactic-baseballer-source \
  /Users/mikai/CLionProjects/starclock/.cache/starclock-sources
node tools/galactic-baseballer-reference/manifest.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/verify-manifest.mjs \
  --source-cache .cache/galactic-baseballer-source
```

The manifest is generator-owned. A second generation in `--check` mode must be
byte-identical.
