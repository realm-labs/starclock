# Goal 19 Foundation Audit

## Result

`G19-P0-B1` reproduces the two pinned caches in an ignored Goal-owned path and
freezes the Version 4.4 released boundary without admitting runtime content.

- repository base: `92febad080dd4cf9997718d64b3648fc198ab1f8`;
- plan commit: `d3380a6b4a749968ad0d56a37f84df73b1d1bff4`;
- branch/upstream: `codex/goal19-fate-star-rail-night-reference` /
  `origin/codex/goal19-fate-star-rail-night-reference`;
- structured source: `fd978d6ef09f941fba644c731ab54abd6f7c3568`,
  tree `2df8981c1bea512e21c8c900920c63002b381056`;
- identity cross-check: `7b349e39ee0f6f3bf814567995829b99c95e7a93`,
  tree `1e6892227905e0dad002bb117d63464d7a5640a6`;
- focused discovery seed: 25 dedicated tables and 64 configuration files;
- seed receipt digest:
  `e75abcf749370ac1c03b729adc844eed9f873e5bd3d7dd1827772b5b38440cad`.

Both repositories are detached, clean, origin-checked, commit-readable and
connectivity-checked. A second fetch run is idempotent. The seed remains an
inventory obligation rather than a completeness denominator.

## Boundary decisions

- The released collaboration activity and its retained permanent gameplay are
  in scope; limited account rewards are not.
- `FateRin` and `Config/Gameplays/Fate` are discovery selectors. Names and ID
  adjacency do not prove ownership.
- Currency Wars Fate Bonds and `Config/Activity/RtBattle` remain adjacent
  exclusions unless an explicit FateRin-originating reference proves sharing.
- Goal 19 owns only its six isolated artifact roots and its goal documents.
- Excel/openpyxl is the editable authoring path; Sora 0.3.0 is the only schema,
  generation and export authority; runtime lowering remains excluded.

## Commands

```text
tools/fate-star-rail-night-reference/fetch-sources.sh \
  .cache/fate-star-rail-night-sources \
  /Users/mikai/CLionProjects/starclock/.cache/content-reference
node tools/fate-star-rail-night-reference/freeze-foundation.mjs \
  --source-cache .cache/fate-star-rail-night-sources
node tools/fate-star-rail-night-reference/freeze-foundation.mjs \
  --source-cache .cache/fate-star-rail-night-sources --check
git diff --check
```
