# Assist Skill and companion-protocol boundary

Goal 01 batch `G01-P7-M01` freezes a generic, character-agnostic envelope for released-text Assist mechanics. It is an explicit deterministic project policy for the ten Himeko Nova approximation cases, not a claim that unrecorded game timing was observed.

## Generic model

- `AbilityTag::Assist` identifies an Assist Skill independently from `AbilityKind::Skill` and ordinary attack/skill tags.
- An effect may grant a canonical set of abilities to its target. Removing or expiring that effect removes the grant from subsequent legality checks.
- The unit selecting targets and consuming a turn is the action actor. The effect applier is the provider and action owner; provider stats scale the granted ability and provider attribution is retained on its events.
- If more than one active effect grants the same ability, the lowest canonical effect-instance ID selects the provider. This is deterministic and remains replaceable by stronger source-bound evidence.
- Side-wide Assist uses are named keyed team resources. A normal Assist requires and spends its authored amount at action start.
- A forced action whose envelope selects `SkillPointPaymentPolicy::Suppressed` suppresses Energy, character-resource and named team-resource costs as well as effective Skill Point payment. Authored gains remain active and the attempted Skill Point amount remains available as telemetry.
- An Assist hit may explicitly ignore elemental weakness for Toughness eligibility. This does not add a weakness or alter damage resistance.
- Thresholds remain ordinary typed rule slots and observations. A qualifying threshold queues a tagged forced Assist with explicit actor, owner, target, boundary, priority and payment policy.
- A companion is a generic linked unit with `LinkedEntityKind::SharedActor`. Its unit, provider owner and optional timeline identity remain distinct.

## Released-text approximation policy

The prepared Version 4.4 records keep their exact numeric fields and text hashes. Where those records do not expose target ordering, operation ordering, invalidation behavior or hidden timing, C04 must author a finite deterministic program inside the envelope above. It must not borrow behavior from a similarly named ability or introduce a form ID into combat core.

The initial policy uses stable formation order, authored hit order, canonical effect-instance/provider selection, exact released thresholds and an explicitly authored forced no-cost action. Observation requirements remain attached to every research case. A later source-bound observation or stronger official operation record must replace conflicting policy through a new decision and regression update before the affected content is treated as observed fact.

## Executable evidence

- `crates/starclock-combat/tests/assist_skill_subsystem.rs` covers effect grant legality, provider/actor attribution, provider-stat scaling, shared-use payment, no-cost forced use and weakness-independent Toughness reduction.
- `crates/starclock-combat/tests/elation_subsystem.rs` covers the existing generic shared linked actor and explicit queued owner/payment envelope.
- `crates/starclock-combat/tests/catalog_contract.rs` rejects missing and non-canonical granted ability references.
- The production Sora schema and data bridge carry Assist tags, effect-granted abilities, named team-resource costs and per-hit `ignores_weakness` authoring.

Validation is performed with the focused combat tests, full combat/data test suites, production catalog verification, the research-register verifier and the repository gate.
