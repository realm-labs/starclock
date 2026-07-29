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
node tools/swarm-disaster-reference/import-domains.mjs
node tools/swarm-disaster-reference/verify-domains.mjs
```

`map-edges.json` is explicitly `ProjectPolicy`: the pinned released files expose
coordinates but no edge list. All exact nodes, events, weights and creation
rules remain separately preserved so verified engine evidence can replace the
policy without changing their factual records.

`rooms.json` preserves the exact released room-to-section relation. The source
row does not publish a domain or encounter-pool join, so those fields remain
empty and explicitly deferred to `G09-P2-B5`; numeric room IDs are never
decoded as an inferred rule. Domain replacement, copying, blanking and beacon
generation are retained as typed consequences of the 13 released Audience Die
face rows that change topology. Stable target ordering and empty-target no-op
behavior remain labeled `ProjectPolicy`.
