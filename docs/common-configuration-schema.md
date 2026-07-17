# Common Configuration Schema

`config/schema/common.toml` owns the shared Sora 0.3.0 vocabulary used by the
Goal 01 content families. It does not expose generated rows to combat, build or
activity APIs.

## Shared tables

- `ContentIdentity` owns the positive `i32` transport ID, globally unique
  stable key, content kind, original English/Simplified Chinese metadata,
  introduced/snapshot versions, release state, enabled flag, coverage state and
  top-level source references.
- `SourceRecord` owns publisher/site identity, direct URL, access date,
  applicable version, category, confidence, usage note, optional conflict note
  and a lowercase SHA-256 evidence digest.
- `EvidenceRecord` names a source payload, released-text hash, observation,
  golden fixture or project-policy artifact without storing copied long text.
- `ContentEvidenceBinding` attaches an ordered fact key to both a source and an
  evidence record. Unique `(content_id, sequence)` and `(content_id, fact_key)`
  indexes prevent ambiguous or order-dependent attribution.
- `ConfigManifest` is the singleton version/revision contract carried by an
  exported bundle. The `config.sora` digest is computed after export and belongs
  beside the bundle/replay header rather than recursively inside this row.

The generic identity row supplies metadata and provenance only. Character,
ability, equipment, enemy, rule and activity tables added by later batches
reference its primary key and retain their own typed mechanics. Display names,
summaries and `fact_key` values never select runtime behavior.
The confidence vocabulary retains every label already present in the frozen
research register rather than collapsing those records into a generic rank.

## Transport and domain boundary

Sora transport IDs and authored order fields are bounded positive `i32` because
the pinned 0.3.0 Rust decoder cannot decode unsigned schema primitives. The
`starclock-data` boundary later performs checked conversion into typed nonzero
unsigned domain IDs. A generated Sora type is never a public domain type.

`ProjectFixture` and `SyntheticFixture` exist only for isolated schema evidence.
Domain validation must reject those labels from a production bundle and must
also enforce release/enabled/coverage combinations, date/URL syntax, lowercase
SHA-256 syntax and allowed content-kind relationships that Sora cannot express.

## Canonical decimals

Every authoritative fractional source field uses a name ending in `_decimal`,
Sora `string`, and a maximum source length of 32. `f32` and `f64` are forbidden
from the schema family. The source grammar is:

```text
^-?(?:0|[1-9][0-9]*)(?:\.[0-9]{0,5}[1-9])?$
```

It permits at most six fractional digits, forbids exponent/locale/plus forms,
leading zeroes, negative zero and redundant trailing fractional zeroes, and
must fit signed 64-bit millionths. `tools/config-schema/canonical-decimal.mjs`
implements this check using only `BigInt`; it never routes through a JavaScript
number. Field-specific bounds remain domain validation owned by B10/P2.

## Fixture boundary

`config/schema-fixtures/common` uses small TOML rows solely to exercise Sora
schema validation, references, indexes, code generation and deterministic
exports before production workbooks exist. The verifier also requires the exact
six-workbook Excel template projection without hashing unstable ZIP metadata.
Its identity is disabled and
explicitly labeled synthetic. It is not a production authoring path, runtime
input, V1a probe or coverage row.

`G01-P1-B10` owns the deliberate table-source migration to generated/synchronized
`.xlsx` workbooks, the production Sora project, generated reader and validated
row-to-domain conversion. No production command reads this fixture bundle.
