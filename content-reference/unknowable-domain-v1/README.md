# Unknowable Domain V1 Normalized Reference

This directory contains generated, implementation-facing Version 4.4
Unknowable Domain reference rows. It is not a runtime configuration source.
Production authoring remains the isolated Excel workbooks under
`config/unknowable-domain/`, validated and exported by Sora 0.3.0.

Every row uses a Starclock-owned stable ID, short independent bilingual
mechanical summaries, explicit ownership/coverage/evidence labels and ordered
source references. Upstream numeric IDs remain provenance locators only.

Regenerate and verify the isolated Sora schema with:

```text
node tools/unknowable-domain-reference/generate-sora-schema.mjs .
node tools/unknowable-domain-reference/verify-sora-schema.mjs
```

The normalized JSON pack remains `ForbiddenReferenceOnly`. Sora and Excel are
an isolated authoring/validation surface; runtime lowering belongs to a later
goal.

The frozen Candidate release has normalized-pack SHA-256
`f48f264fb55221e2494156c5ab7911719d703ec47f492c9c0e2d7fd2c8123b28`.
It closes 5,377/5,377 source obligations across 43 categories, with 41
reference-only mechanic rules, 4,473 provenance rows, 24 executed semantic
fixture families and 24 nonblocking research boundaries. Release review and
evidence verify with:

```text
node tools/unknowable-domain-reference/verify-semantic-fixtures.mjs .
node tools/unknowable-domain-reference/audit-release.mjs .
node tools/unknowable-domain-reference/verify-release-acceptance.mjs .
node tools/unknowable-domain-reference/verify-release.mjs .
```

The isolated Sora review bundle has SHA-256
`05114105b6d905c2858865df08d7ab551cb0fb056b3871b959897a4a590451ec`.
It is not a runtime bundle, does not enable
`unknowable-domain.profile.v1`, and does not imply runtime loading, lowering,
handlers, controller/API exposure or seeded end-to-end runs.
