# Lifecycle and Resolution

This document is normative for command acceptance, action resolution, defeat, wave transitions, and deterministic faults. It resolves lifecycle ambiguities in the earlier overview documents.

## Battle phases

`BattlePhase` has exactly these top-level states:

| Phase | Accepted external input |
|---|---|
| `Initializing` | `StartBattle` only. |
| `AwaitingCommand` | One of the legal commands exposed by the current decision point. |
| `Resolving` | No external command; queued operations drain synchronously. |
| `Won` | None. Terminal. |
| `Lost` | None. Terminal. |
| `Faulted` | None. Terminal for the current rules revision. |

An accepted command always reaches `AwaitingCommand`, `Won`, `Lost`, or `Faulted` before `apply` returns. Presentation acknowledgements never suspend `Resolving`.

## Command atomicity

Command processing has three boundaries:

1. **Legality validation** reads state only. Failure returns `CommandError` and state, RNG, IDs, and logs remain byte-identical.
2. **Resolution journal** records ordered mutations, generated events, RNG draws, and sequence allocations.
3. **Commit** publishes the resulting state and canonical hash.

A deterministic internal failure uses one of two explicit policies selected by the failing subsystem:

- `Rollback`: discard the uncommitted journal and then commit a single stable `Faulted` transition from the pre-command state.
- `CommitFault`: preserve already committed atomic operations, append a fault event describing the failed boundary, and enter `Faulted`.

Returning a normal error after partial mutation is forbidden. Fault diagnostics use stable enums, IDs, and numeric context; platform error text is excluded from event and state hashes.

## Decision points

A decision point contains its kind, controller owner, stable sequence ID, legal commands in canonical order, and any deadline measured in simulation units. Battle decision kinds are:

- battle start;
- normal unit action;
- interrupt/Ultimate window;
- target or battle-local mode choice required by an authored rule.

Legal commands are values, not callbacks. A controller may select only from this collection. Stable order is part of the replay contract.

Cross-battle route, roster, reward, shop, replacement, and abstract noncombat choices are `ActivityDecisionPoint` values owned by `starclock-activity`. They follow the same canonical-command rule but cannot appear while a submitted battle is resolving. See [Activity core and mode extension](19-activity-core-and-mode-extension.md).

## Action envelope

Every action receives an `ActionId`, source, ability, action kind, cause, and optional normal-turn ownership. The default envelope is:

1. validate availability, actor presence, target program, and all costs;
2. reserve costs whose rule says they are paid on commit;
3. emit `ActionDeclared` and open declared pre-action interrupts;
4. pay costs and emit `ActionStarted`;
5. run ability phases in authored order;
6. for every operation, apply the atomic-operation sequence below;
7. drain reactions eligible before the next phase;
8. emit `ActionResolved` and resolve after-action triggers;
9. perform turn-end ticks if this action owns the normal turn;
10. settle the wave/action boundary and expose the next decision.

An Ultimate, follow-up, counter, summon action, memosprite action, joint contribution, and extra turn all use this envelope. Their action kind and normal-turn ownership determine which triggers and duration clocks apply.

## Atomic operation sequence

An operation is the smallest journal unit that either completes or faults. The resolver performs:

1. revalidate dynamic selectors and declared empty-target policy;
2. allocate the operation/event sequence;
3. capture requested snapshots;
4. calculate the typed result without mutation;
5. apply the mutation;
6. emit its primary event and derived change events;
7. settle replacement effects, defeat candidates, and invalidated targets;
8. collect matching triggers in canonical order;
9. enqueue reactions and update budgets;
10. update terminal-candidate flags without automatically spawning a wave.

Damage and Toughness operations in the same authored hit remain distinct ordered operations. An ability phase must say which comes first when a kit depends on that distinction.

## Event causality

Each event has a monotonic `EventId` and one cause record:

```text
Cause {
  parent_event,
  root_command,
  action,
  phase,
  hit,
  owner,
  actor,
  applier,
  source_definition,
  primary_target
}
```

`owner` owns the rule; `actor` performs the current action; `applier` receives application credit; and `source_definition` identifies the ability, effect, equipment, enemy, or mode rule. They may differ. Trigger filters must never infer one field from another.

## Life, presence, and defeat

Life state and battlefield presence are independent:

| Axis | Values |
|---|---|
| Life | `Alive`, `Downed`, `Defeated` |
| Presence | `Present`, `Reserved`, `Departed`, `Untargetable`, `Linked`, `Transformed` |

At zero HP, resolve in this order:

1. clamp HP and emit the HP change;
2. collect prevention/replacement candidates;
3. select replacements by priority and stable source ID;
4. if unreplaced, enter `Downed` and invalidate ordinary actions/targets;
5. resolve immediate revival or transformation rules;
6. if still downed, enter `Defeated`, emit defeat credit, and remove timeline eligibility;
7. resolve defeat triggers;
8. recompute battle terminal candidates.

Revival explicitly declares restored HP, effect cleanup, timeline position, presence, action cancellation, and per-battle usage. Summon-owner defeat and memosprite teardown are authored links, not universal deletion.

## Target invalidation and retargeting

Committed targets remain attached to a hit plan. If a target becomes illegal before a later hit, the hit plan selects one explicit policy:

- `CancelRemainingForTarget`;
- `KeepIfPresent` for effects allowed to affect downed/untargetable state;
- `RetargetSamePool` using the current stable pool and declared RNG consumption;
- `RetargetPrimaryThenRebuildPattern`;
- `FailAction`, which is valid only before any state-changing operation.

There is no implicit nearest-target or random retarget fallback.

## Waves and boss phases

Defeating the last hostile unit sets a pending boundary; it does not normally insert the next wave inside the current multi-hit action. `WaveTransitionPolicy` is:

- `AfterAction` — default for encounters;
- `AfterPhase` — an authored ability phase may cross the boundary;
- `AfterHit` — only for verified cross-wave hit plans;
- `Explicit` — a scripted operation performs the transition.

At a transition: drain reactions allowed before wave end, emit `WaveEnded`, remove/depart wave-owned actors and effects, create the next wave in stable slot order, apply declared persistence/reset policies, emit `WaveStarted`, and run wave-start rules. Surviving allies and team resources persist unless the encounter says otherwise.

A boss phase change is not automatically a new wave. It may transform the same unit, replace it while preserving a logical boss link, or run an explicit wave transition. The encounter definition chooses one model and declares HP, Toughness, effects, action gauge, summons, and target-lock carryover.

## Queue order and budgets

Pending work has a total order over reaction priority, phase, side, formation index, spawn sequence, source ID, rule ID, and insertion sequence. No comparison ends without a fixed-width tie-breaker.

Rules-revision constants bound events per command, trigger depth, queued reactions, extra actions, hit/bounce count, active effects, and linked actors. Exceeding a bound emits a stable budget fault. Limits are not silently raised for a particular character or boss.

## Required tests

- rejected commands preserve the complete pre-command hash;
- numeric, invariant, and budget faults follow their declared journal policy;
- death prevention, revival, defeat credit, and owner-linked actor teardown use the stated order;
- multi-hit actions do not enter the next wave under `AfterAction`;
- each nondefault wave policy has a golden fixture;
- target death exercises every retarget policy;
- simultaneous reactions are stable under different insertion/container layouts;
- terminal states accept no commands and produce no RNG draws.
