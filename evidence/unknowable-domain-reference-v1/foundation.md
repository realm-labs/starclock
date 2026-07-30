# Goal 10 Foundation

`G10-P0-B1` starts `unknowable-domain-reference-v1` from the registered Goal 03
completion tree. Goal 10 produces a Candidate-quality Unknowable Domain
reference bundle. It does not lower the bundle into the runtime or change
`starclock-combat`, `starclock-activity`, shared runtime operations, shared
schemas or shared generated readers.

## Frozen prerequisite

The immutable prerequisite is Goal 03 completion commit
`60ca52ed98c5c83d867d33bff7f88c69e0b389de` with tree
`3fc33d2b45ad9344522faf6e470c861ab75ff4c5`. Its release policy, release
evidence and completed ledger remain historical state. The inherited source
boundary is game Version 4.4, accessed on 2026-07-22:

| Repository | Revision |
|---|---|
| `Dimbreath/turnbasedgamedata` | `fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| `Mar-7th/StarRailRes` | `7b349e39ee0f6f3bf814567995829b99c95e7a93` |

The Standard source inventory supplies 32 hashed
`ExcelOutput/RogueMagic*.json` rows, all previously classified as
`other_mode`. They are a discovery seed, not an Unknowable Domain denominator.
`G10-P0-B2` must close over explicit selectors and references into shared Rogue
tables, TextMaps, StageConfig, enemy/wave definitions and ability/configuration
programs.

Goal 03 recorded source bytes from a CRLF checkout. This macOS worktree checks
out the same Git blobs with LF endings. Foundation verification accepts only
an exact byte match or an LF-to-CRLF transformation that reproduces both the
recorded byte count and SHA-256.

## Source-cache reproduction

The repository's PowerShell source fetcher could not run on this host because
`pwsh` is not installed:

```text
pwsh -File tools/content-reference/fetch-sources.ps1 -CacheRoot .cache/content-reference
zsh:1: command not found: pwsh
```

Goal 10 therefore owns a POSIX equivalent at
`tools/unknowable-domain-reference/fetch-sources.sh`. It uses partial clone and
sparse checkout, resolves both pinned commits, leaves both repositories
detached and clean, and checks Git object connectivity. Two consecutive runs
completed successfully, demonstrating that the cache is reproducible and the
operation is idempotent. The verifier independently checks remote URL, exact
HEAD, detached state, clean status and connectivity.

The sparse boundary intentionally includes all `RogueMagic*` structured
tables, both selected TextMaps, StageConfig, shared Rogue tables, encounter
configuration, enemy definitions and broad ability/configuration roots needed
for the transitive closure. Bulk source material remains ignored cache data.

## Parallel ownership checkpoints

Gold and Gears was inspected at local committed revision
`2f7b3ccf699c52c2738136b8636d140e053bb2eb` with tree
`7ede07b5531dc322ecffdf0096f9855d8548fb24`. Its manifest freezes 7,913
obligations: 7,199 `GoldAndGears` and 714 `Shared`. That commit was not
reachable from a configured remote during this audit, so it is informative
and optional for foundation verification. Goal 10 must reconcile against a
remote-backed or merged Gold and Gears checkpoint in `G10-P4-B3`.

Swarm Disaster was inspected at remote-backed revision
`1f9019a2a29ed5300eeee5925f67f4ac9e495ae2` with tree
`9617eb1eb6956d9c34dcee2591dc2d7c21f3bdd5`. Its source inventory contains
2,882 records: 2,873 from `turnbasedgamedata` and 9 public index cross-checks
from `StarRailRes`. The remote branch
`origin/codex/goal09-swarm-disaster-reference` contained that commit; later
Goal 09 batches may advance the branch without invalidating this checkpoint.

Neither checkpoint grants Unknowable Domain membership. Shared facts reconcile
by source path, stable row locator and evidence digest. A prefix, adjacent ID
range or identical localized name is never sufficient.

## Scope and authoring boundary

Goal 10 includes released-source inventory, unique systems and lifecycles,
reachable shared and mode-owned content pools, services, encounters,
battle-visible and cross-battle rule contributions, bilingual independent
summaries, row evidence, semantic review fixtures and isolated authoring
artifacts.

Story prose, presentation, assets, audio, UI, account rewards, unreleased
content and other modes are excluded except as mechanical locators or explicit
ownership/exclusion evidence. Runtime lowering, handlers, CLI, Agent, MCP,
playable flow and shared runtime changes belong to a later goal.

Excel `.xlsx` files authored through Python `openpyxl` are the production
authoring surface. Sora 0.3.0 is the schema, generation and export authority;
JSON is staging/debug only. The host `PATH` currently resolves `sora 0.2.0`,
which is not accepted as authority. Phase 3 must resolve the repository-pinned
0.3.0 tool from `policy/sora-toolchain.json`.

## Parallel artifact isolation

Goal 10 runs on `codex/goal10-unknowable-domain-reference` and owns only:

- `content-manifests/unknowable-domain-v1/`
- `content-reference/unknowable-domain-v1/`
- `config/unknowable-domain/`
- `config/unknowable-domain-generated/`
- `tools/unknowable-domain-reference/`
- `evidence/unknowable-domain-reference-v1/`
- its three Goal 10 documents

Standard, Gold and Gears, Swarm Disaster and Goal 07 artifacts are protected.
Shared generated directories `config/universe-generated/` and
`config/generated/` are also protected. Ownership or semantic conflicts are
recorded for merge coordination instead of changing another goal's rows.

## Execution shape

The plan contains 28 atomic batches across five phases. Phase 0 freezes source,
manifest and schema/fixture contracts before content import. Each later batch
owns its normalized rows, bilingual mechanical summaries, evidence, coverage
and semantic fixtures. Every completed batch is pushed to the configured
remote branch and its full commit ID is remotely resolved before the next batch
starts.
