# Goal 07 Occurrence Partition S05

`G07-P4-M13-S05` owns 32 frozen records spanning Saleo, Bounty Hunter,
Implement of Error, both We Are Cowboys variants, History Fictionologists (I)
and Nildis (Lightfish).

## Exact activity outcomes

The shared Occurrence handler executes every assigned choice without a
deferred-effect counter:

- Saleo resets the graph with 100 Cosmic Fragments, or discards one owned
  Curio for one two-star Blessing before its authored transition;
- Bounty Hunter discards one owned Curio through the complete lifecycle and
  grants exactly 200 Cosmic Fragments;
- Implement of Error grants one stable Reward-RNG-selected repairable Error
  Code Curio from the complete six-item pool;
- both We Are Cowboys variants lose exactly 50% of current Cosmic Fragments
  with floor semantics before their battle transition;
- History Fictionologists enhances three one-star, two two-star or one
  three-star Blessing from the Path with the greatest number of currently
  owned Blessings, with stable ordering for tied Paths;
- Nildis implements the complete four-attempt Lightfish probability ladder.

## Bounded progressive state

Nildis uses a dedicated player-visible, hash-committed bounded counter map for
its attempt number. Reward and blank results advance through the authored
56/30/14, 32/60/8 and 8/90/2 ladders and repeat the same content node; the
fourth attempt is guaranteed to enter battle. Battle completion and giving up
both reset the progressive state. The content-node self-edge is generic and
selected only when the interaction emits its repeat marker.

Catalog-backed pools and ordered Reward RNG keep every inventory mutation
replayable. Focused runtime tests cover all 32 assigned choices, real seeded
Blessing and Curio inventories, both fragment-loss variants, all three first
Lightfish outcomes, the guaranteed fourth failure and give-up reset. Reviewed
source facts and captured digests are in
`evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S05.json`.
