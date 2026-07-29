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
