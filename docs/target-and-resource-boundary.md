# Target and action-resource boundary

Goal 01 batch `G01-P3-B4` replaces the B3 structural action flag with typed,
finite catalog action definitions. An executable ability now declares its
Basic, Skill or Ultimate family, one to 64 authored hits, target selector,
target-invalidation policy and explicit Skill Point/Energy costs and gains.

## Offered targets and commitments

The baseline selector vocabulary covers self, allied and opposing
relationships plus Single, Blast and All patterns. Every pool is built from
alive, present units in canonical formation order. Single and Blast produce one
offered command for every legal primary; self and All use no controller-supplied
primary. Blast commits the primary and valid numeric formation neighbors at
action lowering. Each hit carries the resulting ordered runtime unit IDs in its
public event and its cause retains the primary target.

An `ActionPlan` owns that commitment for its complete multi-hit execution. Each
hit revalidates it through exactly one authored policy:

- `CancelRemainingForTarget` removes an illegal target;
- `KeepIfPresent` retains downed or ordinarily untargetable battlefield state;
- `RetargetSamePool` draws from the current stable pool;
- `RetargetPrimaryThenRebuildPattern` replaces the primary and rebuilds the
  authored pattern;
- `FailAction` follows the transaction's rollback fault path.

Retarget draws use the pinned battle RNG purpose and enter the forward journal.
Selectors declare whether repeated identities are permitted; the baseline is
no repetition. Catalog construction rejects executable abilities whose selector
has no target semantics.

## Resource legality and timing

Skill Points remain team-owned integral values. Energy is a non-negative
six-decimal personal domain value stored with its authored cap. Both enter the
canonical state codec and immutable views. Entry Energy above its maximum is a
typed combatant-spec error.

Legal-command construction checks life, presence and every payable cost before
offering an action. A battle participant must retain at least one zero-cost
normal action, preventing an empty normal decision. The current resource model
also rejects a zero-cost Ultimate definition. This does not hard-code ordinary
game defaults: Basic generation, Skill cost, Ultimate cost and Energy gain are
all explicit per ability.

Costs commit after `ActionDeclared` and before `ActionStarted`. Gains commit
after the authored hits and before `ActionResolved`. Every mutation emits a
typed resource fact containing before/after values and discarded ordinary-gain
overflow. An offered Ultimate executes in the current interrupt window, does
not reset Action Gauge or end the normal turn, and reopens a freshly enumerated
interrupt decision after its costs settle.

Golden fixtures retain the B3 command hashes under the expanded canonical
state and add fixed Ultimate and three-hit Skill hashes. Focused fixtures prove
Basic cap overflow, exact SP/Energy affordability, stable Blast locks across
hits, all invalidation policies, and invalid catalog/spec shapes. Damage,
defeat and target invalidation caused by real HP mutations remain owned by
`G01-P3-B5` and use this same commitment path.
