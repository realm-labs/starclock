# Goal 07 Occurrence Partition S03

`G07-P4-M13-S03` owns 32 frozen records spanning Kindling of the
Self-Annihilator, Nomadic Miners and the three Cosmic Merchant graph variants.

## Exact activity outcomes

The shared Occurrence handler executes every assigned choice without a
deferred-effect counter:

- Nomadic Miners enhances two uniformly selected owned Blessings or grants
  one two-star Preservation Blessing;
- Kindling grants one three-star Blessing and one negative Curio in the same
  checked transaction, or grants 100 Cosmic Fragments;
- Cosmic Merchant spends exactly 100 Cosmic Fragments for a one-star
  Blessing or 200 for a negative Curio;
- Cosmic Con Job spends exactly 100 Cosmic Fragments for a normal Curio or
  one Blessing of any rarity;
- Cosmic Altruist spends exactly 10 Cosmic Fragments to enhance three owned
  Blessings or obtain one three-star Blessing.

All costs use checked atomic Activity operations. Unaffordable choices reject
without changing state. Random Blessing and Curio pools compile to stable,
ordered external results, so the submitted handler performs no hidden
Activity RNG draw.

## Generic Blessing pools

The outcome schema now carries four compact stable references for the complete
Blessing pool, one-star pool, three-star pool and two-star Preservation pool.
The runtime expands those references from the authoritative catalog. This
keeps the Sora parameter-reference bound closed while avoiding content-ID
branches or a duplicated list of 162 Blessing keys.

The focused runtime test covers the 162-candidate enhancement pool, seven
two-star Preservation results, 135 Kindling result combinations, exact Curio
pool sizes and real Cosmic Fragment debits. The reviewed source facts and
captured digests are in
`evidence/standard-universe-mechanics-complete-v1/source-reviews/G07-P4-M13-S03.json`.
