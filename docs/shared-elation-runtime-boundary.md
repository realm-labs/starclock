# Shared Elation runtime boundary

Status: implemented by `G01-P4-B8`. This boundary is generic combat-domain
infrastructure. It does not encode a released character ID, resolve the open
Elation observation cases, or grant production content coverage.

## Typed identity and damage

`AbilityTags` is a checked compact set independent from `AbilityKind` and from
the damage formula. It can distinguish Attack, Skill, Ultimate, follow-up,
counter, summon, memosprite, additional, joint and Elation Skill semantics.
`DamageClass::Elation` travels independently through authored ordinary-damage
operations and authoritative damage events. A Skill can therefore be tagged as
an Elation Skill while emitting zero, one or several Elation damage operations;
an ordinary ability can emit Elation damage without acquiring the Skill tag.

The Sora `Ability.semantic_tags_mask` field is validated into this domain set.
Generated `DamageClass::Elation` rows lower into the same combat-domain damage
class. Neither generated reader types nor the numeric bit mask cross into
`starclock-combat` public state.

## Shared resources

`TeamResourceSpec` owns canonically ordered keyed resources. Each definition
has an initial value, maximum and explicit wave policy: persist, reset to its
initial value, or clear. Authoritative state, battle views and canonical state
encoding retain the keyed values.

Mutations use one generic operation and one event envelope. Gain, spend and set
are checked before commit; the event records attempted and effective amounts,
before/after values and overflow. The Rule IR addresses a team resource by an
authored key. Data compilation must eventually bind that key to the battle-local
`SourceDefinitionId`; the prepared probes retain the key without introducing a
runtime JSON path.

This shape is sufficient for a Punchline-like meter, but no core type is named
after Punchline and no unresolved cap, credit, threshold or wave behavior is
selected as a default.

## Holder, provider and forced-use envelope

A Certified-Banger-like state uses the ordinary effect model. The effect holder
is its subject while the provider remains in the effect cause. Grant, replace,
consume and teardown behavior stays authored by effect and rule definitions.

Queued work retains separate actor and attribution owner. The generic envelope
selects the actor from the cause owner, cause applier, primary target, or a
unique linked entity kind; selects the retained target independently; and
declares one payment policy:

- ordinary team Skill Points;
- suppressed cost; or
- a keyed team-resource substitute.

Forced Skills are accepted only when the ability is explicitly tagged as an
Elation Skill. Affordability and target legality are checked before action IDs,
resource mutation or RNG use. An invalid forced action is cancelled without
turn ownership, gauge movement or a hidden fallback target. Skill Point events
retain the attempted cost, actual payer and effective spend so rules need not
infer which cost path occurred.

## Shared actors

A shared subsystem actor is an ordinary linked unit with
`LinkedEntityKind::SharedActor`. It has a stable unit identity, owner link and
presence, but does not receive an automatic timeline action merely by existing.
Authored queued work selects the unique active linked entity of that kind on the
provider's side. This keeps provider, actor, owner and target distinct and uses
the same reaction scheduler as all other forced work.

## Evidence and unresolved observations

The Trailblazer (Elation) and Yao Guang scopes are disabled `ProjectFixture`
workbooks compiled twice with pinned Sora 0.3.0. They export all 80 production
tables and grant zero production coverage:

- `config/probes/v1a/trailblazer-elation/golden.json`
- `config/probes/v1a/yao-guang-elation/golden.json`

`crates/starclock-data/src/probe_tests.rs` proves the typed Rule IR and generated
lowering. `crates/starclock-combat/tests/elation_subsystem.rs` proves the
authoritative provider/actor split, capped team resource, substitute/suppressed
costs, forced Skill execution, independent tags/damage class and generic shared
actor selection.

The eight `G01-R-ELATION-*` cases remain `Researching`. The probes constrain the
implementation but do not replace the required cross-kit live observations for
exact trigger matrices, target/cost policies, concurrent provider behavior or
wave persistence. No released form name or ID appears in combat resolver code.

Validation commands:

```text
node tools/config-probes/verify-elation-probes.mjs
cargo test -p starclock-combat --test elation_subsystem --all-features
cargo test -p starclock-data probe_tests --all-features
rg -n -i "trailblazer|yao.?guang|sparxie|silver.?wolf|punchline|banger|aha" crates/starclock-combat/src
```
