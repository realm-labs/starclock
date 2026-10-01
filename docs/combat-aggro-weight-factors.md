# Authored Aggro weight factors

`StatKind::Aggro` is a normalized target-weight factor. Its neutral combat base
is exactly one, not a guessed character or Path-specific threat value. Shared
stat resolution applies declared percent-of-base, flat and final modifiers,
their purpose, source/subject ownership, grouping, bounds and snapshot policy.
An Aggro-purpose modifier and a generic Stat-purpose modifier are both readable
by an Aggro-purpose stat query under the existing resolver contract.

## Explicit targeting contract

An authored `RngWeighted` selector explicitly reads candidate
`QueryStat(CurrentTarget, Aggro, Aggro)` and multiplies it by the content-owned
base weight, with named checked rounding. The neutral base weight may be one
only when the author declares uniform baseline weights. No shared resolver
infers character membership, Path weights or a mode's enemy behavior.

For example, a candidate with a percent-of-base Aggro modifier `0.3` has factor
`1.3`. With equal authored base weights, it is sampled against the other
candidate's factor `1`, not against an unchanged or missing stat. This is a
generic formula contract, not evidence that a particular game's hidden Aggro
parameter uses this mapping.

The existing battle-owned integer sampler executes the authored pool in stable
order on its declared `aggro-target` stream. Eligible zero-weight candidates
are never selected; all-zero pools consume no draw and follow their authored
empty-pool behavior. Negative final weights fault; there is no implicit clamp.
Accepted faults and rejected commands retain the shared atomicity contract.

Fixed targets, manual commitments, uniform selectors and weighted expressions
that do not query Aggro remain unchanged. Merely attaching an Aggro modifier
does not implement targeting for an enemy whose authored selector ignores it.

## Query and snapshot boundary

The neutral base is provided consistently to:

- live program/rule and selector queries;
- initial modifier-value capture; and
- immutable event/action selector projections.

Historical selectors continue to read their captured modifiers rather than
substituting current values. These bases remain deterministic derived query
inputs, not an additional serialized stat or a second targeting engine.

## Verification and scope

Native accepted-command tests in
`crates/starclock-test-kit/tests/suites/core/combat/aggro_weight.rs` execute
real weighted selections and damage, verify effective factors in live/event
queries and application-time capture, cover zero and invalid weights, and bind
64-seed current selection goldens. Rejected command and fresh reconstruction
checks compare events, state hashes and RNG counts. Existing pure formula and
integer RNG golden tests remain applicable.

This shared prerequisite requires no Sora opcode or mode handler. The consuming
[Self-Amusement overlay](divergent-universe-weighted-curio-retaliation.md) now
authors elemental eligibility, explicit baselines, enemy-selector bindings and
nonlethal retaliation. Seven of seventeen Weighted Curio effects execute, without
additional original obligation, program, family, gap or policy terminal credit.
