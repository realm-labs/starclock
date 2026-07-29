# Reference authoring contracts

Goal 13 uses three independent machine contracts:

| Contract | SHA-256 | Frozen responsibility |
|---|---|---|
| `normalized-schema.json` | `db436a34b66bacc6811bf0b6101d58317691faccd106cf29024f2271cca0a12a` | 37 normalized files, bilingual row envelope, evidence/mechanism quality, canonical values, approximation and shared-source reconciliation. |
| `authoring-contract.json` | `66a959881e4877d172b57ea1ff1196a63f878c72ee34991345a51b628bc5b850` | Three complete `.xlsx` files, openpyxl 3.1.5, Sora 0.3.0, isolated paths, no-overwrite generation and visual QA. |
| `fixture-contract.json` | `5d523aa85c7716cabaea6ce8d95db1f1a4558b4363adda1e44c99246da58a5f8` | 18 semantic families and their minimum cases, deterministic traces and coverage rules. |

## Authoring boundary

Excel is the editable production surface. JSON remains normalized
research/debug staging and is never a runtime input. Sora 0.3.0 is the only
schema validation, code-generation and production export authority. The three
workbooks are:

- `AnomalyArbitration.xlsx`;
- `AnomalyArbitrationBindings.xlsx`; and
- `AnomalyArbitrationReview.xlsx`.

The authoring path must create complete clean targets through Python
`openpyxl`; it may not patch an existing designer workbook or edit an `.xlsx`
as a ZIP. Phase 3 owns actual schemas, templates, workbooks, generated readers,
exports and rendered visual inspection.

## Reconciliation boundary

Every shared-source receipt records the Goal 13 stable record ID, source path,
stable row locator, evidence SHA-256, peer Goal/record, classification, state,
note and replacement condition. A semantic or ownership conflict is recorded
as `Conflict` and deferred to merge coordination. Goal 13 never rewrites
another Goal's manifest, normalized row, workbook or generated output.

## Approximation boundary

An unavailable field is never silently inferred. An
`ApproximateFromReleasedText` or `ProjectPolicy` field records:

- the unavailable fact and exact field path;
- the selected policy and rejected alternatives;
- rationale, affected fixture IDs and confidence; and
- a concrete stronger-evidence replacement condition.

Neither a normalized row nor a semantic fixture makes a runtime-executability
or observed-parity claim.
