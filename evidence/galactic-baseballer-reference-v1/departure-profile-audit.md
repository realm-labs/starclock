# Goal 16 Departure Profile and Stage Audit

`G16-P1-B1` authors the Version 2.2 Departure profile, its Version 4.4
retention boundary, six stage definitions and all 57 stage-period rows.

## Profile and release boundary

The stable profile ID is:

```text
galactic-baseballer.departure.v2_2
```

It records released version `2.2`, retained baseline `4.4`, source season
`EarlyAccess`, activity module `5003501` and exact entry-unlock quest
`6070207`. The profile remains ReferenceOnly and runtime-disabled.

Gameplay retention and limited account rewards are separate normalized rows:

- `ReferenceOnlyPermanent` retains the mechanical profile;
- `EvidenceOnly` retains the limited reward/window locator without projecting
  account rewards into simulation data.

Both rows cite publisher-operated released pages. They do not copy event prose.

## Stage closure

All six `EvolveBuildStageConfig` rows are mapped exactly once:

| Stable ID | Chinese | English | Difficulty | Initial weapon |
|---|---|---|---:|---|
| `departure.stage.414001` | 火山星球 | Volcanic Planet | 1 | `3106002` |
| `departure.stage.414002` | 齿轮星球 | Cogwheel Planet | 1 | `3106004` |
| `departure.stage.414003` | 糖霜星球 | Sugarfrost Planet | 1 | `3106005` |
| `departure.stage.414004` | 微缩星球 | Miniature Planet | 2 | `3106007` |
| `departure.stage.414006` | 甜梦星球 | Blissdream Planet | 2 | `3106012` |
| `departure.stage.414007` | 永恒黑洞 | Eternal Black Hole | 3 | selectable |

Each row retains exact phase-period lists, team-bonus MazeBuff ID, unlock quest,
trial avatars, recommended weapon/level pairs, recommended accessories and
ordered `C/B/A/S/SS` score thresholds.

All 57 `EvolveBuildStagePeriod` rows are also mapped exactly once. Each retains
its stage/event identity, rank, wave count, countdown sequence, weakness order,
preferred weaknesses, period/stage scores, emotion thresholds, battle area,
deadline position, selection weight and special-monster score structure.

The three rows with source IDs `3097`, `3098` and `3099` remain present and
marked `unresolved_shared_stage=true`; no StageConfig identity is invented for
them. Tutorial periods `414001` and `414002` are resolved to shared stage
configurations even though they are not children of the six displayed stage
rows.

## Provenance and encoding

Every normalized row resolves to its P0-B3 manifest obligation and carries the
pinned Version 4.4 source path, row locator and evidence digest. Localization
uses the exact 64-bit TextMap key as a decimal string. Binary floating-point
values are never authored: fractional source values are canonical decimal
strings.

Generated artifacts:

| File | Rows | SHA-256 |
|---|---:|---|
| `profiles.json` | 1 | `731b797b4bbbb431bbe05175b2d138a9a4bcd2e8a7d46683536c521a50948b1c` |
| `release-boundaries.json` | 2 | `c1ad3d8bfd796d7b6a4d46f7836696068350d663950945924b55bdf14347f4d7` |
| `stages.json` | 6 | `64dd9cef71856995f2fca68095aeea5130d42334d6c10681e284aa070aa170f6` |
| `stage-periods.json` | 57 | `94d7144a2d9c62891822726b52e33bb717b109349580a44202cffa7a67e17c5d` |

Reproduction:

```text
node tools/galactic-baseballer-reference/normalize-departure.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/verify-departure-profile.mjs \
  --source-cache .cache/galactic-baseballer-source
```
