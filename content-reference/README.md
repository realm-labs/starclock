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

The isolated Version 4.4 Gold and Gears Candidate reference lives under
`gold-and-gears-v1/`. It has its own manifest, normalized pack, four
Excel/openpyxl workbooks, Sora 0.3.0 project, provenance, coverage,
approximation register and semantic fixtures. It remains reference-only and
does not enter the Standard profile or production runtime bundle.

The isolated Version 4.4 Swarm Disaster Candidate reference lives under
`swarm-disaster-v1/`. It has its own frozen manifest, normalized pack, four
Excel/openpyxl workbooks, Sora 0.3.0 project, provenance, coverage,
reconciliation evidence, approximation register and semantic fixtures. It
remains reference-only and does not enter the Standard profile or production
runtime bundle.

The isolated Version 4.4 Unknowable Domain Candidate reference lives under
`unknowable-domain-v1/`. It has its own frozen manifest, normalized pack, three
Excel/openpyxl workbooks, Sora 0.3.0 project, provenance, coverage,
reconciliation evidence, approximation register and semantic fixtures. It
remains reference-only and does not enter the Standard profile or production
runtime bundle.

The isolated Version 4.4 Divergent Universe Candidate reference lives under
`divergent-universe-v1/`. It has its own frozen manifest, normalized pack,
three Excel/openpyxl workbooks, Sora 0.3.0 project, provenance, coverage,
reconciliation evidence, approximation register and semantic fixtures. It
remains reference-only and does not enter the Standard profile or production
runtime bundle.

The isolated Version 4.4 Currency Wars Candidate reference lives under
`currency-wars-v1/`. It has its own frozen manifest, normalized pack, three
Excel/openpyxl workbooks, Sora 0.3.0 project, provenance, coverage,
reconciliation evidence, approximation register and semantic fixtures. It
remains reference-only and does not enter any Simulated Universe or production
runtime bundle.

The isolated Version 4.4 Anomaly Arbitration Candidate reference lives under
`anomaly-arbitration-v1/`. It has its own frozen manifest, normalized pack,
three Excel/openpyxl workbooks, Sora 0.3.0 project, provenance, coverage,
peer reconciliation, approximation register and semantic fixtures. It remains
reference-only and does not enter any production runtime bundle.

The isolated Version 4.4 Pure Fiction Candidate reference lives under
`pure-fiction-v1/`. Its frozen 796-obligation manifest is fully DataReady; the
pack includes three Excel/openpyxl workbooks, 37 Sora tables, provenance,
shared-row reconciliation, 25 rules and 18 semantic fixtures. It remains
reference-only and does not enter any production runtime bundle.

The isolated Version 4.4 Memory of Chaos Candidate reference lives under
`memory-of-chaos-v1/`. Its frozen 477-obligation manifest is fully DataReady;
the pack includes three Excel/openpyxl workbooks, 27 Sora tables, exact
season/Tierce encounter closure, 305 shared reconciliations and 18 semantic
families. It remains reference-only and does not enter any production runtime
bundle.

The isolated Version 4.4 Apocalyptic Shadow Candidate reference lives under
`apocalyptic-shadow-v1/`. Its frozen 129-obligation manifest and 1,246-row
pack include three Excel/openpyxl workbooks, 35 Sora tables, 81 shared
reconciliations and 42 fixtures. It remains reference-only and does not enter
any production runtime bundle.

The isolated Version 4.4 Fate/Star Rail Night Candidate reference lives under
`fate-star-rail-night-v1/`. It has a frozen 1,904-obligation manifest, a
2,018-record normalized pack, four Excel/openpyxl workbooks, a 48-table Sora
0.3.0 review bundle, provenance, peer reconciliation, thirteen replacement
boundaries and 58 semantic fixtures. It remains reference-only and does not
enter any production runtime bundle.

The post-merge audit for all six Candidate packages is generated at
`evidence/reference-integration-v1/merged-mode-audit.json`. It binds their
final completion commits, proves the merged manifests and release evidence are
unchanged, covers all 15 mode pairs and fails on any factual evidence conflict
or runtime-boundary leak.

The separate four-way post-merge audit for Pure Fiction, Memory of Chaos,
Apocalyptic Shadow and Fate/Star Rail Night is generated at
`evidence/high-priority-reference-integration-v1/merged-mode-audit.json`. It
checks all six pairs at both literal receipt and canonical upstream-identity
layers, records identity coordination required before runtime lowering and
fails on factual evidence drift or runtime leakage.
