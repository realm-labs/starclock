# Standard Simulated Universe v1 Manifests

This directory freezes the Version 4.4 main-world Standard Simulated Universe
denominator used by Goal 03.

`source-inventory.json` is generated from the pinned ignored released-data cache
by `tools/universe-reference/inventory.mjs`. Its table-level families are an
audit boundary, not final content membership. `standard_candidate` and
`shared_requires_reachability` rows are examined during `G03-P0-B3`; only rows
proven reachable from a frozen main-world manager/pool enter the concrete
category manifests.

Other-mode, presentation, account and source ability records remain evidence
only and never count as Standard SU content.
