//! Raw actor representation is independent of a selector's owner projection.
use super::{definition, input, runtime};
use starclock_combat::{
    CauseActor, CauseActorKind,
    rule::{
        evaluate::matches_filter,
        model::{EventFilter, RuleEventFacts},
    },
};

#[test]
fn combat_rule_ir_contract_actor_kind_requires_exact_fact_without_owner_inference() {
    let unit = runtime(1);
    let actors = [
        None,
        Some(CauseActor::Unit(unit)),
        Some(CauseActor::TimelineActor(runtime(1))),
    ];
    for actor in actors {
        let facts = RuleEventFacts {
            actor_kind: actor.map(CauseActor::kind),
            ..RuleEventFacts::default()
        };
        for required in [
            None,
            Some(CauseActorKind::Unit),
            Some(CauseActorKind::TimelineActor),
        ] {
            let filter = EventFilter {
                actor_kind: required,
                actor: Some(unit),
                ..EventFilter::default()
            };
            let mut context = input(&[], definition(1), &[]);
            context.event_facts = &facts;
            // Retain the same legacy resolved actor/owner identity for every
            // frame, including the missing raw-actor frame. It is not proof.
            context.cause.actor = Some(unit);
            context.cause.owner = Some(unit);
            assert_eq!(
                matches_filter(&filter, context),
                required.is_none() || required == facts.actor_kind
            );
            context.cause.actor = Some(runtime(2));
            assert!(
                !matches_filter(&filter, context),
                "kind and actor filters are conjunctive"
            );
        }
    }
}
