# Goal 11 Foundation

`G11-P0-B1` starts `divergent-universe-reference-v1` from the remotely
published Goal package commit
`b0cd3cb912c9f2ec887c3ae29f79353c4a861643`. Goal 11 produces a
Candidate-quality Version 4.4 Divergent Universe reference bundle. It does not
lower the bundle into the runtime or change `starclock-combat`,
`starclock-activity`, shared runtime operations, shared schemas or shared
generated readers.

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

The Standard source inventory supplies 64 hashed
`ExcelOutput/RogueTourn*.json` files, all previously classified as
`other_mode`. They are a discovery seed, not a Divergent Universe
denominator. `G11-P0-B2` must close over explicit selectors and references
into shared Rogue tables, both TextMaps, StageConfig, enemy/wave definitions
and ability/configuration programs.

Goal 03 recorded some source bytes from a CRLF checkout. This macOS worktree
checks out the same Git blobs with LF endings. Foundation verification accepts
only an exact byte match or an LF-to-CRLF transformation that reproduces both
the recorded byte count and SHA-256.

## Source-cache reproduction

The existing cache under the main worktree was inspected read-only. Both
repositories were clean at the exact required commits, used the expected
origins, contained readable commit objects and passed
`git fsck --connectivity-only --no-dangling`.

Goal 11 owns a POSIX reproducer at
`tools/divergent-universe-reference/fetch-sources.sh`. It uses partial clone
and sparse checkout, resolves both pinned commits, leaves both repositories
detached and clean, and checks Git object connectivity. Reproduction is run
against a new temporary directory so an active Goal's shared source cache is
never reconfigured. The focused verifier accepts an explicit
`--source-cache` path and independently checks origin, exact HEAD, detached
state, clean status, connectivity, all 64 inherited `RogueTourn` files,
TextMaps, StageConfig and the six direct ability/layout files.

The remote-only proof was attempted first. GitLab's HTTP/2 promisor transfer
failed with `curl 92 ... PROTOCOL_ERROR` and an early EOF. A second HTTP/1.1
attempt resolved the fixed commit but stalled while retrieving checkout blobs
and was terminated after a bounded wait. The accepted substitute uses the
fetcher's optional second argument: a read-only, clean, connected cache at the
same commit supplies an isolated filesystem clone (`clonefile` on macOS and
reflink-when-supported on Linux). The reproduced repositories retain the
canonical `origin`. Two consecutive runs against that isolated target must
pass. This proves deterministic cache materialization and idempotence without
presenting the unavailable fresh network transfer as successful.

The initial sparse boundary intentionally includes all Rogue tables, selected
TextMaps, StageConfig and the six direct ability/layout files. `G11-P0-B2`
expands from this seed through explicit Git-tree selectors and transitive
references into shared configuration, encounter and enemy records; sparse
checkout availability itself never grants membership. Bulk source material
remains ignored cache data.

## Parallel ownership checkpoints

Goal 08 was inspected at committed revision
`c283c7f195dcfe05854f3b212df73444ee89255a` with tree
`932978ed94349b5c8c39ad993a1b260283906f72`. Its manifest freezes 7,913
obligations: 7,199 `GoldAndGears` and 714 `Shared`. No configured remote branch
contained that checkpoint during this audit, so it is immutable local
reconciliation input only. Goal 11 must replace it with a remote-backed or
merged checkpoint during `G11-P4-B3`.

Goal 09 was inspected at remote-backed revision
`d5d261a3c0b151eda85cdca52bf12c46a8ff04f4` with tree
`dfa0a443e99a7659be11112b70f3a5a05217befd`. Its manifest freezes 6,963
obligations: 6,305 `SwarmDisaster` and 658 `Shared`.

Goal 10 was inspected at remote-backed revision
`a2e64e1ddf40dd5e4570e576650be0472044794d` with tree
`29863ffc5345b1a9eb772274ea9de8e9ec70745f`. Its manifest freezes 5,377
obligations: 5,243 `UnknowableDomain` and 134 `Shared`.

The other Goal worktrees continued running during the audit and contained
their own uncommitted next-batch changes. Those files were not read as
authoritative checkpoints and were not modified. Later advancement of their
branches does not mutate the exact commits frozen here.

None of these checkpoints grants Divergent Universe membership. Shared facts
reconcile by source path, stable row locator and evidence digest. A table
prefix, adjacent ID range, module label or identical localized name is never
sufficient.

## Scope and authoring boundary

Goal 11 includes released-source inventory, unique systems and lifecycles,
reachable shared and mode-owned pools, services, occurrences, encounters,
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
which is not accepted as authority. Phase 3 must resolve the
repository-pinned 0.3.0 tool from `policy/sora-toolchain.json`.

## Parallel artifact isolation

Goal 11 runs on `codex/goal11-divergent-universe-reference` and owns only:

- `content-manifests/divergent-universe-v1/`
- `content-reference/divergent-universe-v1/`
- `config/divergent-universe/`
- `config/divergent-universe-generated/`
- `tools/divergent-universe-reference/`
- `evidence/divergent-universe-reference-v1/`
- its three Goal 11 documents

Standard, Gold and Gears, Swarm Disaster, Unknowable Domain and Goal 07
artifacts are protected. Shared generated directories
`config/universe-generated/` and `config/generated/` are also protected.
Ownership or semantic conflicts are recorded for merge coordination instead
of changing another Goal's rows.

## Execution shape

The plan contains 29 atomic batches across five phases. Phase 0 freezes source,
manifest and schema/fixture contracts before content import. Each later batch
owns its normalized rows, bilingual mechanical summaries, evidence, coverage
and semantic fixtures. Every completed batch is pushed to the configured
remote branch and its full commit ID is remotely resolved before the next
batch starts.
