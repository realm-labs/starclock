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
node tools/swarm-disaster-reference/import-countdown.mjs
node tools/swarm-disaster-reference/verify-countdown.mjs
node tools/swarm-disaster-reference/import-audience-dice.mjs
node tools/swarm-disaster-reference/verify-audience-dice.mjs
node tools/swarm-disaster-reference/import-dice-faces.mjs
node tools/swarm-disaster-reference/verify-dice-faces.mjs
node tools/swarm-disaster-reference/import-communing.mjs
node tools/swarm-disaster-reference/verify-communing.mjs
node tools/swarm-disaster-reference/import-communing-trail.mjs
node tools/swarm-disaster-reference/verify-communing-trail.mjs
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

`countdown-and-disarray.json` preserves all 19 common DLC constants, the
released movement/transition text and the three published Disruption bands.
The initial value, cross-plane carry and same-boundary ordering are isolated
replaceable policies. `boss-decay-levels.json` retains all 42 frozen manifest
rows, but only the 15 rows whose released text names Swarm: True Sting are
enabled for Swarm compilation; the other 27 shared-DLC rows fail closed.

`audience-paths.json` binds the eight selectable mode Paths to the inherited
Standard Universe Path identities and preserves the two released effect slots.
`audience-dice.json` binds each Path to its die and all 42 authored face IDs;
the exact face effects and roll/reroll/cheat controls remain owned by
`G09-P1-B5`.

`dice-faces.json` and `dice-rarities.json` preserve the 42 released faces and
three rarity rows exactly. One typed target rule per face and four roll
controls make stable ordering, resource failure and empty-target behavior
explicit; these operational details remain replaceable `ProjectPolicy`.

The Communing partition preserves 21 Aeon-aligned branch choices, 31 cabinet
objectives, seven capped dimensions and 55 exact point increments. Branch
choices increment a separate Aeon-choice counter because their source rows do
not publish permanent point changes. Cabinet outgoing unlocks are inverted
into prerequisites, and persistence/clamp timing is labeled `ProjectPolicy`.

The Communing Trail partition preserves all 63 released nodes, their exact
dimension thresholds and one typed gameplay effect per node. The released
table does not publish graph edges, so the 56 within-dimension predecessor
relations are derived from stable threshold and talent-ID order and labeled
`ProjectPolicy`. Effect-domain classification keeps five activity-only effects
out of `BattleSpec`; two mixed effects project only their battle contribution.
