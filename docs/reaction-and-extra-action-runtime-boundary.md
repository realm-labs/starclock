# Reaction and extra-action runtime boundary

`G01-P4-B6` completes the shared scheduling boundary for Ultimate interrupts,
follow-ups, counters, extra actions, extra turns and delayed actions. These
families all lower to the common `ActionPlan` envelope. Content does not call a
character-specific resolver and no character identity participates in runtime
dispatch.

## Queue ownership and lifetime

The transaction owns an ephemeral `ReactionQueue`. An authored queue operation
commits its actor, ability, primary target and target-invalidation policy, then
records a transaction-local monotonic reaction insertion sequence in the event
and mutation journal. The queue and counter are excluded from canonical state
and must be empty before an accepted command can commit; work that
must survive an external decision belongs to persistent pending state instead.

Entries are ordered by boundary, signed priority, side, formation, spawn
sequence, source definition, optional rule/instance/trigger identities, actor,
ability and insertion sequence. No comparison can fall back to container order.
The rules revision permits at most 256 drained reactions in one command,
including recursively produced work. Exceeding the bound rolls the command back
with a deterministic budget fault.

The four executable boundaries are `AfterHit`, `AfterPhase`, `AfterAction` and
`BeforeTimeline`. A delayed action is ordinary queued work whose authored
boundary is later than the boundary at which it was created; presentation time
and animation completion never affect eligibility.

## Action and turn ownership

`NormalTurn` remains the only origin that owns normal-turn gauge reset and
turn-end duration processing. `UltimateInterrupt`, `FollowUp`, `Counter`,
`ExtraAction`, `ExtraTurn`, `Forced` and `DelayedAction` use the same action,
phase, hit, operation and resource envelope without ending or resetting a
normal turn. In particular, an extra turn is not represented as 100% Action
Advance and does not tick normal-turn duration clocks.

Crowd control can block follow-ups and counters through the generic
`ControlledAction::FollowUp` capability. Other automatic actions are not
broadly disabled by that check. A queued actor must still be alive, present and
bound to the queued ability when its entry becomes eligible.

## Cause and target policy

The queue event retains the incoming cause: root command, attacker, applier,
source and attacked primary target. The scheduled action creates a new action
cause whose owner/actor is the selected reacting unit and whose primary target
is the committed original attacker. Its first action event is parented to the
queue event, preserving the complete ancestry without nested trigger calls.

Target legality is re-evaluated before the scheduled action is declared. The
ability's explicit invalidation policy may retain or retarget a committed
target. If it yields no legal target, or if the actor/ability is no longer
eligible, the resolver emits `Action::Cancelled`. It never chooses a random,
nearest or otherwise implicit fallback. Cancellation does not consume RNG
unless the authored retarget policy itself requires an RNG draw.

## Excel/Sora and Rule IR boundary

The generated `QueueAction`, `DelayAction` and `GrantExtraTurn` payloads lower
to typed Rule IR proposals. Queue proposals retain distinct actor and target
selectors, an executable ability identity and signed priority. The resolver is
still the sole state mutator. Trigger rows lower with typed phase, once-scope,
priority, condition and source filter; unsupported event points, contextual
filters and condition forms fail catalog loading rather than being ignored.

The disabled Clara V1a workbook binds the prepared Version 4.4 Talent,
Ultimate and Skill records to a Hit-ended counter trigger. Its program queues
the counter before decrementing one Clara-owned bounded enhanced-counter charge.
The workbook is a deterministic design probe and receives zero production
coverage credit. The four Clara observation cases remain `Researching`: the
probe proves the fixed cause, queue and shared-state contracts but does not
claim the still-missing live observations for ally/attacker variants, target
invalidation across presence transitions, AoE charge coalescing or per-target
mark consumption.

## Evidence

- `crates/starclock-combat/tests/reaction_scheduler.rs` proves common-envelope
  execution, total ordering, cause ownership, control cancellation, explicit
  invalidation cancellation, delayed boundaries and RNG preservation.
- `config/probes/v1a/clara-counter/` is regenerated twice by pinned Sora 0.3.0
  and rejected by the production loader.
- `crates/starclock-data/src/probe_tests.rs` proves trigger and program lowering
  plus authored queue-before-charge-consumption order.
