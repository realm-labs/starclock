//! Exact settlement kind is orthogonal to class and action ownership.
use super::{definition, input};
use starclock_combat::{
    DamageKind,
    rule::{
        evaluate::matches_filter,
        model::{EventFilter, RuleDamageClass, RuleEventFacts},
    },
};

#[test]
fn combat_rule_ir_contract_damage_kind_requires_exact_fact_and_conjunctive_class() {
    let kinds = [
        None,
        Some(DamageKind::Direct),
        Some(DamageKind::DotTick),
        Some(DamageKind::DotDetonation),
    ];
    let classes = [
        None,
        Some(RuleDamageClass::Ordinary),
        Some(RuleDamageClass::Dot),
        Some(RuleDamageClass::Additional),
        Some(RuleDamageClass::Elation),
        Some(RuleDamageClass::Break),
        Some(RuleDamageClass::SuperBreak),
    ];
    for kind in kinds {
        for class in classes {
            for has_action in [false, true] {
                let facts = RuleEventFacts {
                    damage_kind: kind,
                    damage_class: class,
                    has_action,
                    ..RuleEventFacts::default()
                };
                let mut context = input(&[], definition(1), &[]);
                context.event_facts = &facts;
                for required_kind in kinds {
                    for required_class in classes {
                        let filter = EventFilter {
                            damage_kind: required_kind,
                            damage_class: required_class,
                            ..EventFilter::default()
                        };
                        let expected = required_kind.is_none_or(|value| kind == Some(value))
                            && required_class.is_none_or(|value| class == Some(value));
                        assert_eq!(matches_filter(&filter, context), expected);
                        assert!(!matches_filter(
                            &EventFilter {
                                has_action: Some(!has_action),
                                ..filter
                            },
                            context
                        ));
                    }
                }
            }
        }
    }
}
