# Goal 07 Service Partition S04

`G07-P4-M14-S04` closes 16 service records and 16 mechanic-rule bindings.
The slice contains Standard Trailblaze Bonus ID `5` plus Divergent Universe
IDs `419`–`432` and `501`.

## Standard enhanced entry bonus

ID `5`, `Blessing Universe`, is the second enhanced entry option. It selects
one eligible, unowned one- or two-star Blessing from the complete Standard
pool. Candidates are canonically ordered and the selection consumes one
labeled Reward RNG draw. An empty candidate set fails without committing
Activity state or RNG.

The option is offered only when the Ability Tree projects the enhanced
Trailblaze Bonus capability. Selection grants the Blessing through the same
checked Activity inventory operation used by ordinary rewards, records the
entry choice in replay v3 and leaves Cosmic Fragments unchanged.

## Expansion profile boundary

The other 15 rows are formally owned by `DivergentUniverse`, retained as
`EvidenceOnly`, omitted from Standard offer pools and authored without a
Standard effect payload. They lower to `ProfileExcluded` and return
`ProfileUnavailable` before payload validation, RNG or authoritative
mutation when presented to the Standard runtime.

Definition and compiled-interaction tests enumerate the complete expansion
set and verify formal ownership plus fail-closed behavior. The boundary is
generic and table-driven; no Trailblaze Bonus content ID enters shared
Activity or combat resolvers.

## Runtime disposition

ID `5` and its rule finish as `ExecutableRuleIr` with `ExactPublic`
accuracy. The remaining records and rules finish as `ProfileExcluded` with
their exact `DivergentUniverse` owner. No native handler or numeric
approximation is admitted.
