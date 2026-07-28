# Goal 07 Occurrence Partition S04

`G07-P4-M13-S04` owns 32 frozen records spanning the remaining Cosmic
Altruist graph, Societal Dreamscape and the first three Saleo states.

## Exact activity outcomes

The shared Occurrence handler executes every assigned choice without a
deferred-effect counter:

- Cosmic Con Job spends exactly 100 Cosmic Fragments for one normal Curio or
  one Blessing of any rarity;
- Cosmic Altruist spends exactly 10 Cosmic Fragments to enhance three owned
  Blessings or obtain one three-star Blessing;
- Societal Dreamscape grants 300 Cosmic Fragments with one negative Curio, or
  grants 100 Cosmic Fragments;
- Sal grants one normal Curio and removes exactly 20% of every living
  participant's current HP without defeating them;
- Leo discards one owned Curio through the complete lifecycle teardown and
  grants one two-star Blessing;
- each Saleo branch transition and 100-fragment reset is an explicit graph
  transition in the same checked transaction.

Random pools remain stable and catalog-backed. Single Curio acquisitions
compile to explicit external results; the Curio-to-Blessing exchange composes
the complete 61-Curio discard pool with the 63 two-star Blessings under one
bounded ordered random policy.

## Generic transition and pool lowering

`Special` now lowers to the shared transition primitive independently of the
schema placeholder target. This supports multi-operation outcomes without
special-casing Saleo or any content ID. The compact
`universe.blessing-pool.rarity.2` reference is expanded from the authoritative
catalog alongside the existing S03 Blessing pools.

Focused runtime tests cover all 24 assigned choices, exact fragment balances,
Curio acquisition and teardown, two-star Blessing inventory mutation, and a
real participant-carry settlement from 1,000 to 800 HP. Reviewed source facts
and captured digests are in
`evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S04.json`.
