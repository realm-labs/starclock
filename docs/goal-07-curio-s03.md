# Goal 07 Curio Partition S03

`G07-P3-M11-S03` executes eight positive and special Curios from the
authoritative Standard Simulated Universe workbooks:

- Faith Bond (`universe.curio.13`);
- Robe of The Beauty (`universe.curio.14`);
- Gold Coin of Discord (`universe.curio.15`);
- Record from Beyond the Sky (`universe.curio.19`);
- Chaos Trametes (`universe.curio.2`);
- Entropic Die (`universe.curio.20`);
- Sealing Wax of Erudition (`universe.curio.211`);
- Sealing Wax of Preservation (`universe.curio.22`).

The partition owns 16 records, 16 rules and three frozen semantic fixtures.
Excel remains authoritative. Curio definitions, states, parameters, rules,
fixtures and provenance are authored in `Universe.xlsx`,
`UniverseBindings.xlsx` and `UniverseEvidence.xlsx`. The pinned openpyxl
authoring command verifies these rows against the committed Sora 0.3.0 bundle
and emits only derived golden evidence.

## Activity services and domain settlement

Faith Bond reduces the fragment price of Blessing enhancement, Blessing-offer
reset and participant revival by exactly 30%. It does not alter shops, Curio
purchases or other respite services. The public evidence does not specify
fractional-fragment rounding, so runtime revision v1 explicitly uses checked
integer floor on the retained 70% price. The undiscounted catalog price remains
available as an upper-bound quote; authoritative settlement calculates the
owned-Curio discount inside the transaction.

Gold Coin of Discord grants 6% of the current Cosmic Fragment balance whenever
a new Domain is entered. Integer evaluation and credit use the shared checked
Activity expression and fragment-gain pipeline, so Gossip composes normally.
If Rubert Empire Mechanical Cogwheel is also owned, its fixed grant settles
first and Gold Coin derives its percentage from the resulting balance. This
ordering is frozen and covered by the topology revision.

Chaos Trametes contributes one additional free postcombat Blessing-offer
reset. It composes additively with the Ability Tree reset authorization while
the generic offer boundary retains a bounded two-use counter. The public
runtime facade, rather than topology alone, calculates the actual owned
authorization and rejects disabled or exhausted rerolls without mutation.

## Authoritative random acquisition effects

Entropic Die immediately enhances two randomly selected owned, unenhanced
Blessings. Selection is canonically ordered, without replacement and driven by
the authoritative Reward RNG stream. If fewer than two eligible Blessings
exist, every eligible Blessing is enhanced; an empty pool consumes the pending
acquisition token without an RNG draw. Enhancement uses the ordinary Blessing
inventory representation, where the enhanced level is stored as value two.

Each Sealing Wax immediately grants one unowned Blessing of its authored Path
through the same generic acquisition boundary. Later postcombat offers give
eligible Erudition or Preservation options increased weight. Public evidence
does not publish either multiplier, so runtime revision v1 freezes both as
replaceable `x2` project-policy approximations. Neither the selection nor the
weight policy requires a native handler.

## Executable battle rules

Robe of The Beauty snapshots the Cosmic Fragment balance at battle assembly
and grants exactly 16% all-damage boost for every complete 100 fragments.
Incomplete hundreds do not contribute. The immutable contribution is lowered
to the ordinary, DoT, additional, Elation and joint damage-purpose modifier
pipeline; combat does not query or mutate Activity state.

Record from Beyond the Sky applies two independent effects to every allied
unit at battle start. The first nullifies ordinary, Break, Super Break,
additional, joint, Elation and true damage, but intentionally excludes DoT.
The first action-backed ordinary hit removes that unit's nullification after
damage resolution. The second grants 100% Effect RES for three target-turn-end
clocks and is not consumed by the first hit. Both effects use typed Rule IR
templates, selectors, durations and removal operations.

No S03 mechanic adds a native handler or a Curio-ID branch to the combat
resolver.

## Revision and executable evidence

`standard-universe-entry-v9` and `standard-universe-topology-v9` identify the
new service payload, owned-Curio reroll policy, acquisition boundaries,
conditional offer weights and deterministic Domain-entry ordering. Older
replays cannot silently adopt these semantics.

Production battle tests verify 32% Robe damage boost at 250 fragments and all
Record mitigation, Effect RES, duration and removal triggers. Activity and
service tests verify exact 70% authored-service prices, Gold Coin composition,
zero-selection atomic random boundaries, two-Blessing Entropic enhancement and
Path-correct Sealing Wax acquisition. Topology tests freeze the two-use reroll
counter and all three conditional Path-offer policies.
