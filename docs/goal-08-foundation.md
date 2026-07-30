# Goal 08 Foundation

`G08-P0-B1` starts `gold-and-gears-reference-v1` from the registered Goal 03
completion tree. Goal 08 produces a Candidate-quality Gold and Gears reference
bundle. It does not lower that bundle into the runtime or change shared runtime
operations, schemas or generated readers.

## Frozen prerequisite

The direct prerequisite is Goal 03 completion commit
`60ca52ed98c5c83d867d33bff7f88c69e0b389de` with tree
`3fc33d2b45ad9344522faf6e470c861ab75ff4c5`. Its release policy and evidence
remain immutable. The inherited structured-source boundary is game Version
4.4, accessed on 2026-07-22:

| Repository | Revision |
|---|---|
| `Dimbreath/turnbasedgamedata` | `fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| `Mar-7th/StarRailRes` | `7b349e39ee0f6f3bf814567995829b99c95e7a93` |

The existing Standard source inventory has twenty-one hashed
`ExcelOutput/RogueNous*.json` rows. Those rows are a discovery seed, not the
Gold and Gears denominator. Reachable shared Rogue tables, TextMap records,
StageConfig rows and ability programs still require focused closure in
`G08-P0-B2`.

Goal 03 recorded source bytes from a CRLF checkout. On this macOS worktree the
same Git blobs are checked out with LF endings. Foundation verification accepts
only an exact byte match or an LF-to-CRLF transformation that reproduces both
the recorded byte count and SHA-256; arbitrary normalization is not accepted.

## Scope boundary

Goal 08 includes released-source inventory, mechanically relevant mode
systems, pool ownership and reachability, encounter identities, semantic
review fixtures, isolated Sora tables/readers and isolated Excel workbooks.
Story presentation, dialogue prose and account/collection rewards are
excluded except where a mechanical unlock locator is required.

The goal ends at a Candidate reference bundle. Runtime lowering, integration,
controller/API exposure and seeded complete runs belong to a later goal.
Standard Universe, Swarm Disaster, Unknowable Domain and Divergent Universe
rows fail closed unless a Gold and Gears reachability edge is proven.

## Parallel artifact isolation

Goal 08 runs on a `codex/goal08-*` branch and owns only its named artifact
roots:

- `content-manifests/gold-and-gears-v1/`
- `content-reference/gold-and-gears-v1/`
- `config/gold-and-gears/`
- `config/gold-and-gears-generated/`
- `tools/gold-and-gears-reference/`
- `evidence/gold-and-gears-reference-v1/`

It must not mutate Standard reference manifests/workbooks, production
`config/generated/`, Standard staging `config/universe-generated/`, completed
Goal 03 evidence or Goal 07 tooling and partitions. This makes source research,
normalization, isolated workbook authoring and isolated Sora export safe while
Goal 07 runs in another worktree.

## Execution shape

The plan contains 28 atomic batches across five phases. Phase 0 freezes source,
denominator and authoring contracts before broad import. Every later content
batch owns its normalized rows, bilingual mechanical summary, evidence,
coverage and semantic fixtures. Excel is authoritative, Sora 0.3.0 owns schema
export, and JSON remains staging/debug data only.
