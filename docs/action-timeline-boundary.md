# Action and timeline boundary

Goal 01 batch `G01-P3-B3` establishes the first deterministic normal-turn
action envelope inside `starclock-combat`. It extends the transaction boundary
without exposing fixed-point storage, mutable battle state or engine types.

## Timeline selection

Every present actor owns a private Speed and Action Gauge. Turn selection
compares exact rational action values by promoted integer cross-products, then
uses side, formation, spawn ordinal, unit ID and actor ID as the complete stable
tie key. Advancing all other eligible gauges uses one checked calculation and
floors elapsed distance to six decimal places; ineligible actors retain their
gauge and the selected actor is set to zero explicitly. Completing its normal
action resets that actor to the full 10,000 gauge before the next selection.

The active normal turn and a stable action boundary are authoritative state. A
boundary records its identity, suspended turn and typed continuation: continue
the selected action or complete the action-owning turn. The resolver returns at
these boundaries independently of Ultimate readiness. Core callers may submit a
ready Ultimate request or `Advance`; adapters normally auto-advance when no
ready Ultimate needs external input.

A stable boundary may coexist with an already offered normal `DecisionPoint`.
In that state, `Advance` closes only the current Ultimate-insertion opportunity
and preserves the normal decision unchanged. When no normal decision is open,
`Advance` resumes the boundary's typed continuation. It therefore never skips
an actor's pending normal action.

An Ultimate request records only actor and ability. It closes the current
boundary, creates `PreparedActionState`, and offers exact target commitments plus
cancel. Neither request nor cancellation declares the action or pays resources.
Committing a prepared target normally executes the action and restores the
suspended continuation at a newly identified action boundary. A bounded
segmented Ultimate instead declares and pays once, persists an `ActionFrame`
only between complete segments, and accepts exact offered `CommitActionFrame`
inputs until its automatic finisher resolves the same action identity. The two
confirmed segmented families and their input shapes are audited in the
[character action-flow matrix](characters/action-flow-matrix.md).

## Structural action lowering

B3 initially proved a deliberately narrow structural envelope. B4 replaces
that staging flag with typed finite action definitions, target commitments,
resources and multi-hit plans as documented in the
[target and action-resource boundary](target-and-resource-boundary.md).
Lowering still allocates monotonic `ActionId`, `PhaseId` and `HitId` values only
after the offered command has passed exact decision-membership validation. B5
adds HP operations and does not create a second action language.

The synchronous fact chain is:

1. decision closed;
2. action declared and started;
3. phase started;
4. hit started and ended;
5. phase ended;
6. action resolved;
7. higher-priority reactions drained;
8. after-action boundary opened;
9. turn ended after `Advance` resumes the saved continuation;
10. next turn started and its before-action boundary opened.

Every fact retains the root command and immediate parent. Action, phase and hit
identities are added as soon as they exist. Stable fixed vectors cover the
initial, start, boundary-advance, concede and completed structural-action states.

## Action-boundary ordering

The private reaction queue freezes a total ordering: forced follow-ups, then
extra actions, followed by owner side/formation and insertion ordinal. Manual
Ultimates enter only through stable action-boundary commands and never split an
atomic hit or reaction. Queue state and pending counts are canonically encoded;
no character-ID branch or alternative queue is permitted.
