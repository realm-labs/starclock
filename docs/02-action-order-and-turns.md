# Action Order and Turns

## Canonical timeline representation

Use Action Gauge (AG), not rounded displayed Action Value, as canonical state.

```text
final_spd = base_spd * (1 + total_spd_percent) + total_flat_spd
base_action_gauge = 10_000
base_action_value = 10_000 / final_spd
current_action_value = current_action_gauge / final_spd
```

These formulas are consistently documented by the [Star Rail Wiki Speed page](https://honkai-star-rail.fandom.com/wiki/Speed) and [KQM Speed guide](https://hsr.keqingmains.com/misc/speed-guide/). The UI rounds displayed AV upward, but the simulation should retain full precision.

## Advancing time

At battle start, normal actors begin with `current_ag = 10_000`, after battle-entry modifiers. To find the next actor:

1. compute `delta_av = min(current_ag / current_spd)` among eligible actors;
2. subtract `current_spd * delta_av` from every eligible actor's AG;
3. the selected actor reaches zero and starts its turn;
4. after its normal action, reset its AG to 10,000;
5. continue after interrupts and turn-end processing.

This is algebraically equivalent to subtracting the lowest current AV from all actors. Storing AG makes an in-flight Speed change straightforward because distance already traveled does not change.

## Speed changes

When Speed changes, AG is unchanged and current AV is prorated:

```text
new_av = old_av * old_spd / new_spd
```

This is a consequence of `AG = AV * SPD`, not an independent action advance.

## Advance and delay

Advance/delay modifies the current gauge by a fraction of a full 10,000-point gauge:

```text
new_ag = max(0, current_ag - 10_000 * (advance - delay))
```

Equivalently:

```text
new_av = max(0, old_av - base_av * (advance - delay))
```

Percentages are decimal values: 25% is `0.25`. Action delay may push AG beyond 10,000. A 100% advance only subtracts 10,000, so it does not necessarily reach zero after a sufficiently large delay.

An authored **immediately take action** effect is different: it sets current AG to zero regardless of its previous value. The distinction and formulas are documented on the [Speed page](https://honkai-star-rail.fandom.com/wiki/Speed#Action_Value_Modification).

## Weakness Break timing modifiers

The universal Weakness Break reaction delays the enemy by 25%, adding 2,500 AG. This portion is not increased by Break Effect. Quantum Entanglement and Imaginary Imprisonment add their own delays, covered in [Toughness and Break](04-toughness-and-break.md).

## Turn lifecycle

The observed ordinary turn lifecycle is:

1. select the actor with the lowest AV and advance global time;
2. emit turn start;
3. trigger start-of-turn DoT and other start effects;
4. open the pre-action interrupt window;
5. select and resolve the normal action;
6. reset the actor's AG after the action is committed/resolved;
7. drain post-action queued Ultimates and higher-priority reactions;
8. tick eligible effect durations at turn end;
9. emit turn end and select the next actor.

This ordering follows the current community [Speed turn-order description](https://honkai-star-rail.fandom.com/wiki/Speed#Speed). Keep the interrupt windows as domain phases even in a CLI or AI-only build.

## Interrupt and reaction priority

The useful baseline priority is:

1. currently resolving atomic hit/effect;
2. forced follow-up action/counter queue;
3. queued Ultimates and general extra actions, in queue order;
4. extra turns;
5. zero-AG normal turns;
6. future timeline actors.

The wiki documents that follow-up actions ignore normal order and have higher priority than Ultimates/extra actions, while extra turns have higher priority than normal turns but lower priority than follow-ups: [Follow-Up Attack](https://honkai-star-rail.fandom.com/wiki/Follow-Up_Attack#Follow-Up_Actions), [Extra Turn](https://honkai-star-rail.fandom.com/wiki/Extra_Turn).

Implement priority as explicit queue metadata, not nested function calls. Reactions triggered during a reaction are enqueued with a monotonically increasing sequence number.

## Ultimate actions

Ultimates are out-of-order actions normally enabled when their resource reaches the required threshold. They do not consume the user's normal timeline turn. They can be requested in allowed interrupt windows, including at combat start.

The engine should expose an `AwaitingInterrupts` decision point to a human controller, AI, or replay command stream. A timeout or animation is a presentation concern and does not belong in the combat core.

## Follow-up actions and extra turns

Follow-up actions are automatically triggered actions that ignore normal order. Counters are follow-up attacks. A follow-up action generally does not move the source's normal AG.

An extra turn also leaves normal AG unchanged and, according to the [Extra Turn reference](https://honkai-star-rail.fandom.com/wiki/Extra_Turn), status durations do not tick down during it. Do not represent a 100% advance as an extra turn: action advance produces a real normal turn with normal duration processing.

Use different enums such as:

```rust
enum ActionOrigin {
    NormalTurn,
    UltimateInterrupt,
    FollowUp,
    Counter,
    ExtraTurn,
    Forced,
}
```

## Effect duration timing

Effects require an explicit duration owner and tick phase. Common forms include:

- ticks at the end of the affected unit's normal turn;
- ticks at the start of the affected unit's turn;
- ticks at the source's turn boundary;
- lasts for action/hit counts rather than turns;
- lasts until a named phase or condition.

Some effects only decrement if present when the turn began; Speed modifiers are a documented example. Record `present_at_turn_start` per relevant effect instance or use an eligibility epoch. Do not decrement every duration in a single generic loop.

## Cycles are encounter clocks

Challenge cycles are not unit turns. A commonly documented challenge-clock preset gives a first wave window of 150 AV and later windows of 100 AV, but ownership, wave/node resets, expiry, and score effects are mode/stage data. Implement cycles as deterministic encounter/challenge clocks; they must not alter the base action algorithm. Standard battles have no implicit cycle clock. See [Standard battle and challenge modes](18-standard-and-challenge-modes.md) and the community [Speed cycles reference](https://honkai-star-rail.fandom.com/wiki/Speed#Cycles_and_Breakpoints).

## Enemy Speed scaling

Current community data gives these level-based multipliers over an enemy's authored base Speed:

| Enemy level | Multiplier |
|---:|---:|
| 1–64 | 1.00 |
| 65–77 | 1.10 |
| 78–85 | 1.20 |
| 86+ | 1.32 |

Keep this as configurable encounter data because balance patches or specific enemies may override it.

## Unknown and project-defined ordering

Public references do not provide a universal, version-stable rule for every exact-AV tie, simultaneous interrupt requested by multiple controllers, or all multi-phase boss transitions. The project must use a stable tie key such as `(priority, formation_side, formation_index, spawn_sequence, action_id)` and record it in replays. Replace this policy only after a reproducible observation requires it.
