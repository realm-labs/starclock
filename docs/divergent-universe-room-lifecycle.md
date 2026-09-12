# Divergent Universe room lifecycle

## Current boundary

The mode can bind the released `RogueTourn_Goup_WaitDialogue` completion
program to a logical room, accept node-bound completion notifications, and
finish that room before opening its exits. This is a reusable content-executor
boundary, not a complete event implementation or a map-membership assertion.
The source-program notification boundary still lacks complete production
content binding. Separately, the initial authored occurrence now commits its
inventory grant and headless completion latches atomically before opening exits;
see the [occurrence reward contract](divergent-universe-occurrence-rewards.md).
It does not execute missing acquisition effects or establish physical room
membership, and does not complete this source program's release gate.

The source is `Config/Level/Maze/MazeRogue/RogueTourn/RogueTourn_Goup_WaitDialogue.json`
at released revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` of the
turnbasedgamedata repository. Its source locator and digest are retained in the
production Sora mechanic catalog. The exact source-file SHA-256 is
`2bddbca2731e612f24db7247ab91ccb07c2f682fa9307cae1ad7104261a373d6`.
The mode binds this reviewed path/digest and accepts only the five known
operation shapes, each occurring once; additional/missing shapes fail closed.
It does not load the upstream JSON at runtime. A changed file with identical
operation names is rejected: shape counts do not preserve operands or task
topology, so they cannot authorize another source program.

The first start sequence waits for dialogue and a predicate, then runs
`SetRogueRoomFinish` followed by `SetAllRogueDoorState`. A second start sequence
waits for room content updates. The data contains no predicate operand or
explicit target door state. Operation names alone do not establish their
omitted defaults or the scheduling relationship between the two sequences.

## Explicit headless synchronization policy

Policy identity: `divergent-universe.policy.room-dialogue-completion`.

- Missing facts: the default predicate, default door-state value and whether
  the separate content-update wait gates traversal in the released game.
- Selected behavior: latch dialogue completion, predicate satisfaction and
  content-update completion independently; require all three before recording
  room completion and then opening exits in one ordered Activity transaction.
- Notification authority: only the owning content executor may report work
  that has actually committed. Loading a row, running a source-shape test or
  merely receiving a player-supplied result is not such evidence. The current
  public methods are domain integration APIs, not untrusted adapter commands.
- Alternatives: opening doors after dialogue alone could skip an unresolved
  predicate/update; unconditional success would discard known source waits;
  permanently rejecting completion would not provide a usable lifecycle.
- Rationale: conservative synchronization preserves the known finish-before-
  door order and prevents pending content from being bypassed. It does not
  invent any event choices, costs, rewards or map membership.
- Confidence: high in the headless invariants, unverified for game timing and
  omitted source defaults. This is `ProjectPolicy`, not observed parity.
- Replacement condition: released structured defaults or reproducible
  observations establish the predicate, target door state and synchronization
  relation for this exact source program. Replace the corresponding gate;
  retain rejection atomicity and the source's ordered room/door effects.

## State and command ownership

Seven mode-owned typed slots live in the shared Activity Node scope: a room
content-capability flag, bound source-program key, three completion latches,
room-finished flag and doors-open flag. All reset at Node start, including
transitions into battle and finalization. These slots do not replace or invent
the optional factual room-candidate identity.

Binding is allowed only at a content-capable Choice node with a pending
decision. It closes exits for the bound room. Signals carry the expected state
hash and originating node; stale, duplicate, wrong-node and unbound signals
are rejected without mutation. Completion requires the bound source program,
all three latches and a not-yet-completed room. Repeated completion is rejected.

Route options include a transactional `Require` before `Traverse`. An offer
captured before dialogue binding therefore cannot bypass the gate. An offered
route may remain visible while blocked; it is not accepted until completion.
Finalization and battle nodes cannot bind room dialogue. A subsequent room
can bind the same source program again: once-scope is the room, not the run.

No separate Activity machine, global mutable registry, RNG stream, direct state
write, raw-source interpreter or content-ID branch in shared crates is added.

## Verification and remaining work

`room_lifecycle::room_completion_unlocks_the_actual_route_only_after_all_requirements`
checks early and pre-captured route rejection, exact room/door event order,
actual traversal, Node resets and late-notification rejection against a
production-compiled flow. The companion notification test checks stale and
duplicate input and deterministic reconstruction from fresh production inputs.

These are lifecycle integration tests. They do not prove that a full run
executes an event's choices/costs/outcomes. The full production-content release
test remains separate. All 663 runtime programs remain pending behavioral
acceptance; the room program is partial rather than terminal.

Next integration must lower real choices and operands from released evidence
or explicit field-level policies, bind them to logical rooms, and generate
notifications from committed content outcomes. The empty released layer-room
table and missing current event graphs remain unresolved source boundaries;
retained older graphs must not be promoted by name similarity.
