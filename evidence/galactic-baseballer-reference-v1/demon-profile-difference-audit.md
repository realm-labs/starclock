# Goal 16 Demon King Profile and Difference Audit

`G16-P2-B1` freezes Version 3.3 Demon King as an independent profile over the
shared Galactic Baseballer system. It does not overwrite, alias or otherwise
replace the Version 2.2 Departure profile.

## Released profile boundary

| Fact | Frozen value | Evidence |
|---|---|---|
| Profile ID | `galactic-baseballer.demon-king.v3_3` | Goal 16 profile manifest |
| Released version | 3.3 | HoYoLAB Version 3.3 update |
| Retained reference baseline | 4.4 | Pinned Version 4.4 structured tables |
| Released entry requirement | Trailblaze Level 21 | HoYoLAB Version 3.3 update |
| Early access | Finality's Vision supported | HoYoLAB Version 3.3 update |
| Activity module | `5003501` | `EvoBdSCConstValueCommon`, row 37 |
| Origin stage | `424000` | `EvoBdSCConstValueCommon`, row 49 |
| Reward unlock locator | `6070206` | `EvoBdSCConstValueCommon`, row 51 |
| Store unlock locator | `6070210` | `EvoBdSCConstValueCommon`, row 50 |
| Skip-origin unlock locator | `6020139` | `EvoBdSCConstValueCommon`, row 68 |
| Runtime disposition | ReferenceOnly; disabled | Goal scope |

The released event window and account rewards are retained as `EvidenceOnly`
locators. Mechanical gameplay is retained as
`ReferenceOnlyPermanent`. These are separate release-boundary rows.

## Stage closure

`EvoBdSCStageConfig` contributes seven exact rows:

- one `Origin` row (`424000`, Initial Planet);
- six independently selectable challenge rows (`424001` through `424006`);
- the challenge names are V612, C996, F233, M078, D007 and Demon King's Den.

All 56 `EvoBdSCStagePeriod` rows are mapped exactly once. Each row preserves
its ordered phase owner, exact `StageID`, event, rank, wave count, countdowns,
weakness order, score terms, battle area and selection weight. Every referenced
shared `StageConfig` is present in the frozen shared reachability closure; the
Demon King profile therefore has zero unresolved shared-stage references.

No cross-profile stage alias is inferred. Similar planet names do not make a
Demon King stage the same record as a Departure stage. Sharing is admitted
only where a `StagePeriod.StageID` explicitly points to the same frozen shared
stage configuration.

## Complete constant comparison

The edition-difference index compares the two dedicated constant tables by one
deliberately narrow key: remove only the exact `EvolveBuild_` or
`EvolveBuildSC_` prefix. It compares canonical lossless values and yields:

| Relationship | Rows |
|---|---:|
| Value explicitly repeated in both editions | 38 |
| Demon King value differs | 25 |
| Added by Demon King | 13 |
| Departure-only and not inherited | 7 |
| **Compared constants** | **83** |

The 45 changed/added/removed relationships are all explicit in the generated
index. Matching names or values remain comparison facts, not shared record
identity. Detailed ownership remains responsibility-bounded:

- arsenal and synthesis details: `G16-P2-B2`;
- progression and store details: `G16-P2-B3`;
- encounters and score details: `G16-P2-B4`.

## Released Version 3.4 corrections

The [official Version 3.4 update](https://www.hoyolab.com/article/39751178)
identifies three Demon King corrections:

1. incorrect level 7 and level 8 effects for `RuinBot`;
2. abnormal Adventure Score acquisition under specific conditions on
   `D007 - Blissdream Planet`;
3. Boothill Ultimate visual effects against the Black Cloak Demon King.

The first two are retained mechanical correction boundaries. The pinned
Version 4.4 structured rows are authoritative after the fixes. The publisher
does not disclose the erroneous values, the D007 trigger or the obsolete score
delta, so the reference package does not reconstruct them. Each boundary
records the unknown fields, two rejected alternatives, rationale, affected
fixtures, confidence and a released-evidence replacement condition.

The Boothill correction is explicitly visual. It remains `EvidenceOnly` and
does not enter combat or scoring data.

## Reproduction

```text
node tools/galactic-baseballer-reference/normalize-demon-profile.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/normalize-demon-profile.mjs \
  --check --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/verify-demon-profile.mjs \
  --source-cache .cache/galactic-baseballer-source
```

The generator owns six Demon King fragment files. P3-B1 will merge them with
the Departure outputs after all Phase 2 families close.
