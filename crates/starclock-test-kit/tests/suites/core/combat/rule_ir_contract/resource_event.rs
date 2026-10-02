//! Resource mutation-axis filters are orthogonal to addresses and signed values.

use super::{definition, input};
use starclock_combat::{
    Scalar,
    rule::{
        evaluate::matches_filter,
        model::{EventFilter, RuleEventFacts, RuleResourceEventKind, RuleResourceKind},
    },
};

#[test]
fn resource_event_filters_require_the_exact_axis_without_inferring_it_from_delta() {
    for resource in [
        None,
        Some(RuleResourceKind::Energy),
        Some(RuleResourceKind::SkillPoints),
        Some(RuleResourceKind::Character("charge".into())),
        Some(RuleResourceKind::Team("meter".into())),
    ] {
        for event in [
            None,
            Some(RuleResourceEventKind::BalanceChanged),
            Some(RuleResourceEventKind::MaximumChanged),
        ] {
            for delta in [-1, 0, 1] {
                let facts = RuleEventFacts {
                    resource: resource.clone(),
                    resource_event: event,
                    resource_delta: Some(Scalar::checked_from_integer(delta).unwrap()),
                    resource_overflow: Some(Scalar::ONE),
                    ..RuleEventFacts::default()
                };
                let mut context = input(&[], definition(1), &[]);
                context.event_facts = &facts;
                assert!(matches_filter(&EventFilter::default(), context));
                for required in [
                    RuleResourceEventKind::BalanceChanged,
                    RuleResourceEventKind::MaximumChanged,
                ] {
                    let filter = EventFilter {
                        resource_event: Some(required),
                        ..EventFilter::default()
                    };
                    assert_eq!(matches_filter(&filter, context), event == Some(required));
                    let addressed = EventFilter {
                        resource: Some(RuleResourceKind::SkillPoints),
                        ..filter
                    };
                    assert_eq!(
                        matches_filter(&addressed, context),
                        event == Some(required) && resource == Some(RuleResourceKind::SkillPoints)
                    );
                }
            }
        }
    }
    let empty = RuleEventFacts::default();
    let mut context = input(&[], definition(1), &[]);
    context.event_facts = &empty;
    assert!(!matches_filter(
        &EventFilter {
            resource_event: Some(RuleResourceEventKind::BalanceChanged),
            ..EventFilter::default()
        },
        context
    ));
}
