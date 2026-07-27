# Goal 07 Service Partition S03

`G07-P4-M14-S03` closes the next frozen Trailblaze Bonus slice. It owns
16 service records and 16 mechanic-rule bindings: Divergent Universe source
IDs `403`–`418`.

These rows remain in Goal 07's exact-once denominator because the 2,201
records and 786 rules were frozen before profile ownership was corrected.
They are not Standard Simulated Universe content and do not become selectable
or executable effects in a Standard run.

## Profile boundary

Every assigned row is authored in Excel/Sora with:

- `profile_owner = DivergentUniverse`;
- `mode_owner = EvidenceOnly`;
- its exact `source_event_id`;
- no Standard offer-pool membership; and
- no Standard effect payload.

The service compiler lowers each row to `ProfileExcluded`. Attempting to
execute one through the Standard service runtime returns
`ProfileUnavailable` before payload validation, RNG use, currency mutation,
inventory mutation or Activity event emission.

The boundary is table-driven. Runtime tests enumerate every expansion-owned
Trailblaze Bonus from Swarm Disaster, Gold and Gears and Divergent Universe,
verify its formal owner, and prove that all of them fail closed through both
the definition and compiled-interaction paths. This prevents a later catalog
regeneration from accidentally exposing an expansion row through Standard
eligibility.

## Runtime disposition

All 16 records and all 16 rules finish as `ProfileExcluded` with
`ExactPublic` accuracy and explicit `DivergentUniverse` ownership. No native
handler, numeric approximation or guessed expansion behavior is introduced.
The scoped golden binds only the S03 Excel/Sora rows, while the completion
receipt records the exact production artifacts and fail-closed runtime
evidence accepted for this partition.
