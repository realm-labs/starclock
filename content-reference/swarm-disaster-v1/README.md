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
node tools/swarm-disaster-reference/import-pathstrider.mjs
node tools/swarm-disaster-reference/verify-pathstrider.mjs
node tools/swarm-disaster-reference/import-paths.mjs
node tools/swarm-disaster-reference/verify-paths.mjs
node tools/swarm-disaster-reference/import-blessings.mjs
node tools/swarm-disaster-reference/verify-blessings.mjs
node tools/swarm-disaster-reference/import-curios.mjs
node tools/swarm-disaster-reference/verify-curios.mjs
node tools/swarm-disaster-reference/import-occurrences.mjs
node tools/swarm-disaster-reference/verify-occurrences.mjs
node tools/swarm-disaster-reference/import-services.mjs
node tools/swarm-disaster-reference/verify-services.mjs
node tools/swarm-disaster-reference/import-encounters.mjs
node tools/swarm-disaster-reference/verify-encounters.mjs
node tools/swarm-disaster-reference/finalize-pack.mjs
node tools/swarm-disaster-reference/verify-pack.mjs
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

The Pathstrider partition preserves 31 cabinet quest objectives, all 102 shared
DLC finish conditions, all 110 unlock rows and 13 mechanical chapter locators.
Cabinet `QuestID` values remain external completion conditions with their exact
description parameters. Shared DLC rows are enabled only when released text
explicitly names Swarm Disaster: Gold and Gears rows are disabled and
unresolved rows fail closed until an exact Swarm consumer binds them. Chapter
rows retain only plane, Communing threshold and declared bonus status; no story
or missing bonus payload is inferred.

The Path-system partition binds the eight released Swarm Paths to shared
Standard identities, including Propagation but excluding Erudition. It retains
32 shared Resonances/Formations, eight exact Path boost ability locators,
six run-start Trailblaze Bonuses and all 16 released 3+3 Resonance Interplays.
Distinct-blessing threshold counting and Activity commit boundaries are
replaceable `ProjectPolicy`; exact modifier bindings, parameters, bonus values,
ordered operations, unlock IDs and Path extra-effect locators remain separate.

The Blessing partition inherits exactly 18 Blessings for each of the eight
reachable Paths, with the released `8/7/3` rarity distribution and both exact
authored levels for every Blessing. The 184 explicit pool memberships cover
eight selectable Paths, 32 deterministic Resonance/Formation unlocks and 144
Blessings. Released pool weights are unavailable, so selectable Path and
Blessing candidates use stable ID order and equal integer weight `1` as
replaceable `ProjectPolicy`; no additional selected-Path weighting is claimed.

The Curio partition keeps 66 handbook identities separate from their exact
1000-series Swarm copies: 60 identities are shared and six are mode-owned.
Each copy has one typed state and lifecycle rule, including six Error Code
repair states, six numeric charge bindings, Void Wick Trimmer repair and
Shining Trapezohedron replacement. Offer-specific eligibility and weights must
come from the owning service or occurrence; missing bindings fail closed, and
replacement candidates use stable ID order as replaceable `ProjectPolicy`.

The Occurrence partition binds 75 handbook identities to 57 released Swarm NPC
graphs and expands 308 ordered choices. Twelve variants intentionally serve
multiple handbook identities. Choice conditions, costs, source parameters,
dynamic displays, printed percentages and outcome text digests remain
separate. Sixty choices name random behavior without released weights; they
use labeled Activity RNG over stable source order and fail closed on unresolved
candidate pools as replaceable `ProjectPolicy`.

The service partition binds 15 shared services, run-scoped Cosmic Fragments,
four existing beacon contributions and six Adventure rooms. Every service is
an atomic Activity transaction with inherited parameters and fail-closed offer
pools. Adventure minigames accept only validated external Tier1/Tier2/Tier3
results and opaque reward payloads; movement, aiming, physics, timing input and
unreleased threshold/reward tables are intentionally excluded.

The encounter partition expands the complete released 81-series namespace into
179 weighted groups, 347 exact StageConfig waves, 1,070 ordered enemy slots and
15 formal-difficulty/plane-tier boss pools.
Every slot resolves one of 71 inherited Version 4.4 enemy variants, and both
displayed boss choices resolve to concrete slots. Source group weights, stages,
formations and formal difficulty schedules remain exact. The unpublished
room/domain-to-group join and effective area/plane level selection remain
replaceable, fail-closed `ProjectPolicy`; numeric room or group IDs are not
decoded into claimed engine behavior.

The evidence closure contains one implementation-facing reference rule and one
semantic review fixture for each of the 23 frozen mechanic families. Coverage
is recorded per manifest obligation rather than as aggregate-only totals:
all 6,963 frozen obligations resolve to concrete DataReady rows. The source
registry deduplicates 8,139 exact, public, inherited and policy references;
31 evidence gaps are nonblocking and carry explicit replacement conditions.
All 609 facts shared with the committed Goal 08 checkpoint match by source
path, row locator/stable identity and evidence digest. `pack-index.json`
binds every non-index normalized file to its byte length, row count and
SHA-256; the canonical pack digest is recorded on every index row.
