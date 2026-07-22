# Standard Simulated Universe Normalized-Data Design

This document freezes the staging model used to promote released Version 4.4
Standard Simulated Universe facts into Excel/Sora. The machine contract is
[`schema.json`](../content-reference/standard-universe-v1/schema.json). JSON is
research/bootstrap input only; runtime loading remains forbidden.

## Identity and ownership

Every record has a stable Starclock ID. Extracted numeric IDs are retained in
`source_ids` and provenance locators, never silently adopted as public runtime
identity. `mode_owner` is `Standard`, `Shared`, or `EvidenceOnly`. A shared row
is enabled only when the frozen content manifest proves main-world membership.

Definitions are separate from mutable variants:

- one Blessing owns one or more level records;
- one Curio owns explicit intact, damaged, repaired, exhausted or replacement
  states;
- one Occurrence owns base-mode variants and a choice graph;
- one encounter pool references weighted groups, while groups reference
  concrete enemy variants and waves;
- Path Resonance and each Resonance Formation are independent rule sources.

Ability Tree rows reverse released successor edges into explicit
`prerequisite_ids`. Their `effects` are typed contributions with operation,
target, exact string value, unit and condition; source parameter order and
external account-unlock IDs remain available for audit without becoming
runtime identity.

This prevents an extracted state row or historical schedule row from inflating
logical coverage.

## Common record envelope

All authored records contain:

```text
id, enabled, mode_owner
name_en, name_zh_cn, summary_en, summary_zh_cn
quality, mechanism_quality, quality_overrides
coverage_state, provenance_ids, source_ids, note
```

`DataReady` requires non-empty bilingual names and independently written short
summaries, resolved stable-ID references, and at least one evidence record.
Copied story/dialogue paragraphs and asset paths are not normalized content.

## Evidence and quality

Each evidence record binds a fact to repository/URL, revision or access date,
game version, path/page, row locator and SHA-256 digest. Allowed quality labels
are:

- `ExactStructured` — released structured value or relationship;
- `ExactPublicText` — exact public-facing value or rule;
- `Observed` — consistently observed behavior without a complete formal source;
- `ApproximateFromReleasedText` — bounded interpretation of released text;
- `ProjectPolicy` — deterministic Starclock behavior for an unpublished rule.

Approximation is field-level. Every approximate/policy rule states its reason
and a concrete replacement condition, such as discovery of a released weight,
rounding boundary or hidden outcome table.

## Canonical encoding

Files use UTF-8, LF, two-space indentation and a terminal newline. Canonical
hash input recursively sorts object keys. Arrays representing sets are ordered
by stable ID; source-semantic sequences retain their declared order. Exact
decimals are strings matching the six-decimal domain grammar and never pass
through JavaScript, Python or Excel binary floating-point values.

Pack index entries bind each file's byte length, row count and SHA-256 digest.
The complete pack digest hashes the ordered `(relative_path, sha256)` sequence.

## Review fixture contract

A review fixture is not a runtime golden battle. It proves that authored data
captures one distinct mechanic family and contains:

```text
id, mechanic_family, input_ids, initial_state, commands
expected_facts, quality_floor, provenance_ids
```

`expected_facts` are small semantic assertions: threshold, state transition,
cost, reward class, selection constraint, source attribution or battle-rule
contribution. Each distinct Path, Blessing keyword, Curio lifecycle,
Occurrence outcome class, service, Ability Tree effect and encounter policy has
at least one fixture. Project-policy fixtures assert the declared policy, not a
claim of game equivalence.

## Validation order

Validation is deterministic and fail-fast:

1. schema/enums and canonical scalar grammar;
2. unique stable IDs and manifest membership;
3. bilingual and evidence requirements;
4. reference closure and graph-cycle policy;
5. main-world ownership and DLC-leak rejection;
6. coverage and mechanic-family fixture coverage;
7. pack-index and double-regeneration byte identity;
8. Excel/Sora promotion and generated-reader loading.

The workbook stores the same domain fields in normalized tables. `openpyxl`
only creates and inspects workbooks; Sora 0.3.0 remains the validation,
code-generation and export authority.
