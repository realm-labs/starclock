# Swarm Disaster V1 Normalized Reference

This directory contains generated, implementation-facing Version 4.4 Swarm
Disaster reference rows. It is not a runtime configuration source. Production
authoring remains the isolated Excel workbooks under `config/swarm-disaster/`,
validated and exported by Sora 0.3.0.

Every row uses a Starclock-owned stable ID, short independent bilingual
mechanical summaries, explicit ownership/coverage/evidence labels and ordered
source references. Source numeric IDs remain provenance locators only.

Regenerate and verify the Phase 1 topology partition with:

```text
node tools/swarm-disaster-reference/import-topology.mjs
node tools/swarm-disaster-reference/verify-topology.mjs
```

`map-edges.json` is explicitly `ProjectPolicy`: the pinned released files expose
coordinates but no edge list. All exact nodes, events, weights and creation
rules remain separately preserved so verified engine evidence can replace the
policy without changing their factual records.
