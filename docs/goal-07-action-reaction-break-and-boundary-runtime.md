# Goal 07 action, reaction, Break and boundary runtime

`G07-P1-B5` closes the shared action-scheduling and encounter-boundary
capabilities required before Standard Universe content partitions can be
implemented. The resolver still interprets generic action origins, operations,
resources and boundary programs. It does not branch on character, Path,
Blessing, Resonance, Curio or enemy IDs.

## Action envelope and extra turns

All eleven `ActionOrigin` values use the common declared → started → phase →
hit → resolved envelope. Follow-ups, counters, forced actions, extra actions,
summons, memosprites and countdowns are queued reactions. A granted extra turn
is different: it is persistent timeline state ordered by a monotonic insertion
ID and becomes a controller-selected `ExtraTurn` decision before the next
normal timeline advance.

Completing an extra-turn action does not reset Action Gauge, tick normal
turn-end effects or reset normal turn-scoped slots. It still emits action and
turn facts with `origin = ExtraTurn`, so content can observe it explicitly.
Defeat, victory, concession and fault settlement clear pending extra turns.

Action advance and delay mutate the owning timeline actor in the same
transaction. The amount uses the fixed-point `Ratio` domain, overflow faults,
and advance clamps at zero. Every accepted shift emits the actor, owner,
semantic kind, requested amount and exact before/after gauge values.

## Reaction scheduling

The four authored boundaries are `AfterHit`, `AfterPhase`, `AfterAction` and
`BeforeTimeline`. Ready actions are ordered by:

1. boundary;
2. semantic tier: forced follow-up/counter, Ultimate, extra action, extra-turn
   action;
3. authored signed priority;
4. side, formation, spawn, source, rule, instance, trigger, actor, ability and
   insertion IDs.

This prevents a content-authored numeric priority from inverting the shared
semantic order. A command can drain at most 256 reactions; exhausting the
budget faults through the normal transaction policy.

Path Resonance is not a separate resolver subsystem. Its active use is an
ordinary interruptible Ultimate-like ability paid from a keyed team resource.
Its passive behavior is attached Rule IR. This keeps Resonance ordering,
attribution, replay and AI legality on existing combat paths.

## Break and Super Break

`Break` resolves the currently routable Toughness layer through the ordinary
Toughness reduction and break settlement pipeline. It bypasses weakness but
does not bypass layer locks, state mutation, break attribution, base break
effect application or events.

A `Reduced` event now retains the attack element and effective reduction.
After-event Rule IR can therefore issue `SuperBreak` against the observed
target without relying on private hit scratch. Direct hit-plan Super Break
continues to use the same scratch contract. Both routes use the normative
formula boundary and checked fixed-point arithmetic.

## Wave and enemy-phase boundaries

All four wave transition points remain executable: after action, phase, hit,
or an explicit operation. Wave exit programs run before `WaveEnded`; carry
settlement and unique explicit carry programs run between waves; wave entry
programs run after `WaveStarted`. The owner is the causing action owner, with a
stable living-player fallback for encounter-originated boundaries.

Enemy phase transitions preserve their three data models and execute every
built-in carry family transactionally. Unique `ExplicitProgram` carry hooks
run once in authored field order, and the phase entry program runs after the
transition fact. The current transition request remains an authored
operation; entry/exit conditions are immutable data available to enemy
orchestration and later content validation, not an implicit polling loop.

## Compatibility revision

Pending extra turns and turn origin are authoritative state, so combat state
advances to `SCBS` version 4 / `sha256-v5`. Turn and Toughness event payloads
also gained fields, so the current event payload is version 3. Historical
payload versions 1 and 2 retain their exact encodings and are selected from
the recorded payload prefix during replay verification.

The compatibility revision is authored in `ConfigManifest.xlsx` with Python
`openpyxl`. The Break, delay and extra-turn probes are authored in
`Operation.xlsx`; pinned Sora 0.3.0 produces the authoritative `.sora` bundle
and diagnostic JSON.
