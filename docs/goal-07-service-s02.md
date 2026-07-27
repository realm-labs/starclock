# Goal 07 Service Partition S02

`G07-P4-M14-S02` resolves the inherited Trailblaze Bonus classification and
executes the three Standard-owned rows in this partition. It owns 16 records
and 16 rule bindings. IDs `2`, `3` and `4` belong to Standard Simulated
Universe; the other 13 rows belong to named expansion profiles and are
retained only because Goal 07's exact-once denominator was frozen earlier.

## Authoritative profile ownership

`UniverseService.profile_owner` and `source_event_id` are formal Excel/Sora
fields. A Standard Trailblaze Bonus must have owner `Standard`, a source event
and a typed effect payload. A non-Standard row must have its real profile
owner, mode owner `EvidenceOnly`, no Standard offer-pool membership and no
Standard effect payload.

The released source partitions are:

- IDs `1`–`6`: Standard Simulated Universe;
- IDs `101`–`106`: Swarm Disaster;
- IDs `201`–`205`: Gold and Gears;
- IDs `401`–`432` and `501`–`530`: Divergent Universe.

The frozen S02 assignment contains Swarm Disaster IDs `101`–`106`, Gold and
Gears IDs `201`–`205`, and Divergent Universe IDs `401`–`402`. These rows lower
to an explicit `ProfileExcluded` action. Attempting to execute one through the
Standard service handler returns `ProfileUnavailable` before payload
validation, currency mutation or RNG consumption. They never enter a Standard
offer pool.

## Standard entry effects

The three Standard rows in S02 execute through typed shared Activity
primitives:

- ID `2`, ordinary position 2: uniformly grant one eligible, unowned
  one-star Blessing;
- ID `3`, ordinary position 3: spend 50 Cosmic Fragments and uniformly grant
  one eligible, unowned Curio in the same atomic transaction;
- ID `4`, enhanced position 1: grant 150 Cosmic Fragments.

The production entry graph offers exactly three ordinary choices from IDs
`1`–`3`. When the Ability Tree projects
`run.trailblaze_bonus.enhanced`, it offers exactly IDs `4`–`6` instead. The
choice is a spatial-free external outcome; random grants use the authoritative
Reward RNG stream and stable candidate ordering.

Standard runs initialize Cosmic Fragments from the configured currency row
before applying Ability Tree contributions. The selected entry bonus then
settles through the same checked currency and inventory operations used
elsewhere in the run. Tests cover all six Standard entry effects now because
the topology and validation contract must be complete even though IDs `5` and
`6` retain their later frozen partition assignments.

## Replay and engine boundary

The added entry decision is represented by ordinary Activity commands and
events. Replay v3 encodes current nested battle event payloads directly while
the released replay-v2 encoder remains on its historical payload revision.
CLI, Agent, MCP and future engine adapters therefore observe the same legal
decision and deterministic state transition without introducing UI or 3D
navigation state.

No native content handler is admitted. Standard effects use shared service,
currency, inventory and RNG primitives; expansion rows use the generic
profile boundary. Excel/Sora definitions retain all content ownership.
