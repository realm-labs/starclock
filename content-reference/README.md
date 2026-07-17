# Starclock Content Reference Pack

This directory contains pre-implementation, normalized combat facts. It is not
the Sora runtime bundle and does not expose source-project IDs as Starclock
runtime identities.

The Version 4.4 pack under `v4.4/` is generated from pinned public released-data
repositories plus Starclock's independently authored character behavior
profiles. It contains canonical decimal strings, project-owned stable keys,
source locators and evidence hashes. Long source descriptions and assets are
deliberately omitted.

See:

- `docs/content-reference/README.md` for purpose and source policy;
- `docs/content-reference/schema.md` for record semantics;
- `docs/content-reference/authoring-contract.md` for the Excel promotion gate;
- `docs/content-reference/coverage.md` for current counts and approximation
  boundaries;
- `tools/content-reference/README.md` for reproducible generation.

`pack-index.json` hashes every generated file. Goal 01 binds this digest before
it freezes its implementation manifests.
