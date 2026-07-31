# Goal 17 Sora and workbook release audit

## Deterministic artifacts

- Sora: `0.3.0`
- Schema tables: 27
- Schema lock SHA-256: `b072066c0e01c98e23f88aed9735d77cd8d92e42fee931281bcb995767dd4119`
- Generated Rust files: 31
- Generated schema/template/reader tree: `7b207ce042c59718bf9163a28998df22c93d53d8dc93ee22dc153d911f7fae3b`
- Workbook sheets/rows: 27 / 1,521
- Workbook semantic digest: `eb944598c653ebdbb2051cfa8d3f584d24bc550a152df0f90d66679fa2e02189`
- Binary bundle bytes: 3,313,323
- Binary bundle SHA-256: `2d95f797ad87abc168eec20913f9007672bfab1ef4bbce4b57b5afe482bd505f`
- Debug JSON files: 27
- Canonical debug tree SHA-256: `80aa962720eee721009878365f08a45719670b33c73fdea804d76e2bb7ec11bb`
- Generated-reader load: 27 nonempty tables / 1,521 rows
- Runtime publication: forbidden

Two clean workbook generations, binary exports and debug exports are
byte-identical to the committed artifacts. The isolated loader compiles the
generated Sora readers against exact pinned dependencies and loads every row.

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
| `MemoryOfChaosReview-contact.png` | `9bc3138d37069d6f74db3413d55b3649a3ea88a3432ae50b79e51c3fdc6d2b5c` |

All metadata, header and representative authored rows are readable. No visible
clipping, overlap, formula error or broken style remains; blank space is limited
to short sheets and final partial column bands.

## Phase gate

`node tools/repository-check/run.mjs --full` passed in 140.0 seconds,
including Clippy and all 33 workspace test harnesses.
