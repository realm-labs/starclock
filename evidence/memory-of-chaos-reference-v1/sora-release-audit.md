# Goal 17 Sora and workbook release audit

## Deterministic artifacts

- Sora: `0.3.0`
- Schema tables: 27
- Schema lock SHA-256: `b072066c0e01c98e23f88aed9735d77cd8d92e42fee931281bcb995767dd4119`
- Generated Rust files: 31
- Generated schema/template/reader tree: `7b207ce042c59718bf9163a28998df22c93d53d8dc93ee22dc153d911f7fae3b`
- Workbook sheets/rows: 27 / 1,521
- Workbook semantic digest: `9f213165d8284ae8a7f77b1f65aefebdc0844a7cde1ff5ab59d22af2ac709680`
- Binary bundle bytes: 3,313,243
- Binary bundle SHA-256: `a743e5f1459d636c5d906605416a91f1bae2636c96b2598bb66ab736daf32019`
- Debug JSON files: 27
- Canonical debug tree SHA-256: `51f8bd1a29d43efef7821d626cd4a1abd36094e96d3f0b7cb7889d1c584cf25d`
- Generated-reader load: 27 nonempty tables / 1,521 rows
- Runtime publication: forbidden

Two clean workbook generations, binary exports and debug exports are
byte-identical to the committed artifacts. The isolated loader compiles the
generated Sora readers against exact pinned dependencies and loads every row.
These terminal values include the Phase 4 linked correction that replaced three
obsolete semantic-fixture claim aliases with frozen manifest IDs and bound the
event/config gap to an existing fixture case.

## Visual review

- Tool: bundled Artifact Tool 2.8.6+
- Sheets: 27/27
- Rendered column bands: 81
- Schema-column coverage: complete
- Human disposition: `PassedHumanInspection`
- Severe defects: 0

| Contact sheet | SHA-256 |
|---|---|
| `MemoryOfChaos-contact.png` | `28180cd009ad6836e8850951775a368926130058be0c59591e99688d598a8699` |
| `MemoryOfChaosBindings-contact.png` | `9cb9fb23573f5ed06c482ba0387a2de9a0c737b94283dfed91c055fe5ae0bebe` |
| `MemoryOfChaosReview-contact.png` | `d50698efa1adf1cfbbc88134adfad6bf0e0b020b4189873d76caef4779d0422f` |

All metadata, header and representative authored rows are readable. No visible
clipping, overlap, formula error or broken style remains; blank space is limited
to short sheets and final partial column bands.

## Phase gate

`node tools/repository-check/run.mjs --full` passed in 140.0 seconds,
including Clippy and all 33 workspace test harnesses.
