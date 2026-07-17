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

The active normal turn and interrupt window are authoritative state. Start
opens a pre-action interrupt window and offers only `PassInterruptWindow`.
Passing closes that decision and offers the exact executable normal-action
commands for its owner. Catalog/spec composition rejects a participant with no
currently executable ability, so an accepted battle cannot open an empty normal
decision.

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
7. turn ended;
8. next turn started;
9. next interrupt decision offered.

Every fact retains the root command and immediate parent. Action, phase and hit
identities are added as soon as they exist. Stable fixed vectors cover the
initial, start, interrupt-pass, concede and completed structural-action states.

## Interrupt ordering

The private queue freezes a total ordering before it carries executable
interrupts: forced follow-ups, then Ultimates, then extra actions, followed by
owner side/formation and insertion ordinal. The queue state and pending count
are canonically encoded. B4 will construct executable entries through this
ordering; no character-ID branch or alternative queue is permitted.
