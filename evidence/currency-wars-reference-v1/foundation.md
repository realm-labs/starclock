# Goal 12 Foundation

`G12-P0-B1` starts `currency-wars-reference-v1` from the remotely published
Goal package commit
`e3c7b7a18bcad2e3c8305b4512f007f4e39fa19b`. Goal 12 produces a
Candidate-quality Version 4.4 Currency Wars reference bundle. It does not
lower the bundle into the runtime or change `starclock-combat`,
`starclock-activity`, `starclock-build`, shared runtime operations, shared
schemas or shared generated readers.

## Frozen prerequisite

Goal 03 is complete at commit
`60ca52ed98c5c83d867d33bff7f88c69e0b389de` with tree
`3fc33d2b45ad9344522faf6e470c861ab75ff4c5`. Its registered release policy
and evidence hashes match the current immutable snapshot registry. The source
boundary remains game Version 4.4, accessed on 2026-07-22:

| Repository | Revision |
|---|---|
| `Dimbreath/turnbasedgamedata` | `fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| `Mar-7th/StarRailRes` | `7b349e39ee0f6f3bf814567995829b99c95e7a93` |

The inherited Standard source inventory contains 64 hashed
`ExcelOutput/RogueTourn*.json` files classified as `other_mode`. Goal 12 adds
11 `RoguePersona*.json` files as focused discovery seeds. Neither family is a
Currency Wars denominator. `G12-P0-B2` must close explicit `Tourn3`
selectors and transitive references into shared Rogue/build tables, TextMaps,
StageConfig, enemies, waves and configuration/ability programs.

## Source-cache reproduction

Both repositories were rebuilt from their configured remote in a fresh
`mktemp` directory outside every worktree. The reproduction used partial
clones, sparse checkout and detached checkout of the exact pinned commits.
Both repositories were clean, both commit objects were readable and
`git fsck --connectivity-only --no-dangling` passed.

The reproduced source contained exactly 11 Persona and 64 Tourn seed files.
The canonical `{path, bytes, sha256}` list digest is
`9d597bb119388800f394c81d89a9f9220c927180f3bc5be02aab0c9c6a6cb273`.
The exact routing, StageConfig, TextMap, direct S3 ability/layout and
StarRailRes root-file hashes live in
`content-manifests/currency-wars-v1/foundation.json`. They matched the main
worktree's existing cache when compared read-only.

The reproducible command is:

```text
cache_root=$(mktemp -d /tmp/starclock-g12-source-repro.XXXXXX)
tools/currency-wars-reference/fetch-sources.sh "$cache_root"
node tools/currency-wars-reference/verify-foundation.mjs \
  --source-cache "$cache_root"
```

The initial sparse boundary is only a source-entry contract. `G12-P0-B2`
expands it through source references; sparse availability, a filename suffix
or a table prefix never grants ownership.

## Currency Wars selector

Three independent routing facts agree:

- `RogueActivityResidentConfig`: Activity 105, `TournRogue`, module 6002201;
- `RogueTournModule`: MainTourn 3, SubTourn 1, module 6002201;
- `RogueTournAreaGroupByTourn`: an enabled `Tourn3` row.

These facts select the module but do not classify every row in a shared table.
Membership requires an enabled selector, a transitive reference or an
inherited stable-ID closure. Prefixes, suffixes, IDs and localized names remain
discovery hints only.

## Parallel ownership checkpoints

The audit froze committed, immutable inputs without reading another Goal's
uncommitted files as authoritative data:

| Goal | Commit | Tree | Ownership checkpoint |
|---|---|---|---|
| Goal 08 | `d7031b834a72dc118b661f5dfdd2080431729bcc` | `63e8518ef35e80a32d7d8f350f688562aa103ec3` | 7,913 rows: 7,199 Gold and Gears, 714 Shared; local committed checkpoint without a configured remote branch. |
| Goal 09 | `77e83ed2adee63316ad390a597e0362c5af641e3` | `5cfbcca0d926f1d75acba7eed92381bb3bba1347` | 6,963 rows: 6,305 Swarm Disaster, 658 Shared; remote-backed. |
| Goal 10 | `3064e550068429fe92df5bcadeda5dbf8b7eb115` | `61e8aa6736cf91b0e229e0a6c85375a3648d046e` | 5,377 rows: 5,243 Unknowable Domain, 134 Shared; remote-backed. |
| Goal 11 | `b0cd3cb912c9f2ec887c3ae29f79353c4a861643` | `6dab4260090a877adb1e83b9b489b5e4c94197d7` | Remote-backed setup only; no committed ownership manifest was available. Its uncommitted foundation work is protected. |

The other Goal worktrees continued running during this audit. Later branch
advancement does not mutate these exact commits. Goal 08 must be replaced by a
remote-backed or merged checkpoint, and Goal 11 by its first committed
manifest, at `G12-P4-B3`.

No checkpoint grants Currency Wars membership. Shared facts reconcile only by
source path, stable row locator and evidence digest. Conflicts are recorded for
merge coordination rather than changing another Goal's artifacts.

## Scope and authoring boundary

The included boundary is the complete released Version 4.4 mode flow and
lifecycle; Squad HP and action-value projection; roster, shop, star, position,
Empowerment, Bond, build, equipment and Persona systems; reachable shared and
owned content; encounters; bilingual evidence; semantic fixtures; and
isolated Excel/Sora authoring.

Runtime lowering, handlers, controllers, CLI, Agent, MCP, playable flow,
shared runtime changes, story/presentation, assets, audio, UI, account
rewards, unreleased content and unrelated modes are excluded.

Excel workbooks authored with Python `openpyxl` are the production authoring
surface. Sora 0.3.0 remains schema, generation and export authority. The host
`PATH` currently reports Sora 0.2.0, which is not authoritative.

## Artifact isolation

Goal 12 runs on `codex/goal12-currency-wars-reference`, tracks
`origin/codex/goal12-currency-wars-reference` and owns only:

- `content-manifests/currency-wars-v1/`
- `content-reference/currency-wars-v1/`
- `config/currency-wars/`
- `config/currency-wars-generated/`
- `tools/currency-wars-reference/`
- `evidence/currency-wars-reference-v1/`
- its three Goal 12 documents

All other Goal artifacts and shared/production generated directories are
protected. The 29 atomic batches span five phases. A completed batch is pushed
and remotely resolved before the next batch starts.
