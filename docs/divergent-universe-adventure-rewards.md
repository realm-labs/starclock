# Divergent Universe Adventure settlement

The current runtime compiles the reviewed level-one Adventure card preset
`1011` into a shared Activity graph fragment. It accepts one explicit host
result (`AdventureEarnedChests::{None, One, Two, Three}`), executes production
Sora-authored reward IR, finishes the room, opens doors and offers an independent
Leave. This implements settlement, not challenge simulation or the complete
Adventure semantic family.

## Evidence and replacement policy

Pinned released Version 4.4 source revision
`fd978d6ef09f941fba644c731ab54abd6f7c3568` identifies the reviewed card via
`ExcelOutput/RoguePersonaRoomPreset.json` and `RoguePersonaRoomCompType.json`.
The current English TextMap description bounds level-one rewards at three
chests. Production Sources rows 68, 69 and 72 record revision, access date,
file digests and exact locators; schema and domain validation preserve these
joins. No old service candidate gains membership from a shared table or name.

Exact chest reward composition, amounts, challenge selection, scoring and timing
are unavailable after bounded released-source research. The authored
`VersionedProjectPolicyExternalEarnedChestCountAggregateFragmentsOnly` pays
100 base Cosmic Fragments per externally earned chest. It aggregates the count
before the existing active-Curio Fragment gain pipeline, including that
pipeline's integer rounding. Zero earns no currency. No RNG or entry payout
is introduced. The policy records disabled settlement and guessed mini-games
as rejected alternatives, parity as unproven, behavioral fixtures and the
replacement condition: current released chest programs or reproducible
observations establish reward composition, operands and timing.

Challenge score/timer simulation, source-selected mini-games, higher-level
Blessing/Curio rewards and Adventure-specific Curio time/extra-chest/rank
effects remain unimplemented. The baseline's equal-priority result offer uses
its ordinary stable-ID policy (zero chests), not inferred challenge success.
Positive results require an explicit host input.

## Execution and authority

The compiler uses a host slot at or above 70, reset on physical node entry:
`-1` means unresolved and `0..3` is the settled count. Current entry Curio
lifecycle executes exactly once at the logical room entry. The external-result
node offers all four declared outcomes and no early Leave. Each option owns
complete typed IR; `ActivityInteractionBinding::authored_option` requires no
native registrations, callbacks, payload or random policy.

An accepted result credits the aggregate, writes the result, adds one Run
completion receipt, marks `ROOM_FINISHED`, marks `ROOM_DOORS_OPEN`, then offers
Leave without traversing. Receipt-only behavior is not counted as gameplay:
positive counts change authoritative currency through the real gain pipeline.
Repeated result submissions cannot re-pay a settled room. Leave executes the
guarded exit through the shared atomic generated-choice boundary, so a failing
next entry preserves the pending Leave without re-paying the reward.

Binding verifies exact node/program/edge/exit shapes, state declaration,
Run/Plane/Node logical scopes and all result bindings. Room RNG policies,
native handlers, bypasses and changed reward IR reject. Whole-definition
authentication includes interactions and the immutable registry digest as well
as graph, state, participants, bootstrap and random policies. Fresh structurally
identical definitions work; a claimed configuration hash alone is insufficient.

## Verification and remaining release work

Production Sora fixtures cover the reviewed card and malformed identity,
amount, count, metadata and provenance. Runtime fixtures cover every count,
active Curio gains, order, overflow, raw-choice/duplicate/foreign rejection,
fresh reconstruction and real sampled Adventure cards in both run families
with three real nested Boss proxy battles. Other payloads remain explicit
probes, not released complete-run evidence.

Default adapter topology, encoded source-position replay, original challenge
admission and full Adventure gameplay remain pending. No source obligation,
mechanic program, semantic family or full-run release gate receives terminal
credit from this partial settlement boundary.
