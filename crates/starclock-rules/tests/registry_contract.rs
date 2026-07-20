use starclock_combat::{
    AbilityId, ActionId, EventId, NativeHandlerId, RuleInstanceId, SourceDefinitionId,
    WaveInstanceId,
    rule::{
        evaluate::{EvaluationBudget, ProgramLookup, evaluate_program},
        model::{
            ProgramStep, RuleCause, RuleEmission, RuleEvaluationInput, RuleEventKind,
            RuleOccurrence, RuleOperationTemplate, RuleValue, ValueExpr,
        },
    },
};
use starclock_rules::{
    model::{
        BattleHandlerInput, BattleHandlerOutput, BattleHandlerRegistration, HandlerDomain,
        NativeHandlerFault, NativeHandlerRequirement, RegistryErrorKind,
    },
    registry::NativeHandlerRegistry,
};

const HANDLER_ID: NativeHandlerId = match NativeHandlerId::new(1) {
    Some(value) => value,
    None => panic!("one is non-zero"),
};
const SCHEMA: [u8; 32] = [7; 32];

fn synthetic_handler(
    input: BattleHandlerInput<'_>,
) -> Result<BattleHandlerOutput, NativeHandlerFault> {
    Ok(BattleHandlerOutput::new(vec![
        RuleEmission::Informational {
            code: 41,
            value: input.arguments.first().cloned(),
            current_target: None,
        },
    ]))
}

static REGISTRATIONS: [BattleHandlerRegistration; 1] = [BattleHandlerRegistration {
    id: HANDLER_ID,
    stable_key: "synthetic.echo",
    version: "1",
    argument_schema_digest: SCHEMA,
    determinism_note: "pure echo fixture with no RNG",
    owner: "G01-P4-B1 synthetic test",
    ir_insufficiency: "test-only equivalent shape; no content admission",
    removal_condition: "remove with the synthetic registry fixture",
    handler: synthetic_handler,
}];

#[test]
fn registry_audits_version_schema_and_written_decision() {
    let registry = NativeHandlerRegistry::new("native-registry-v1", &REGISTRATIONS).unwrap();
    let requirement = NativeHandlerRequirement {
        id: HANDLER_ID,
        stable_key: "synthetic.echo",
        domain: HandlerDomain::Battle,
        version: "1",
        argument_schema_digest: SCHEMA,
        determinism_note: "pure echo fixture with no RNG",
        owner: "G01-P4-B1 synthetic test",
        ir_insufficiency: "test-only equivalent shape; no content admission",
        removal_condition: "remove with the synthetic registry fixture",
        enabled: true,
    };
    registry.audit(&[requirement]).unwrap();
    let error = registry
        .audit(&[NativeHandlerRequirement {
            ir_insufficiency: " ",
            ..requirement
        }])
        .unwrap_err();
    assert_eq!(
        error.kind(),
        RegistryErrorKind::MissingIrInsufficiencyDecision
    );

    for (changed, expected) in [
        (
            NativeHandlerRequirement {
                stable_key: "synthetic.other",
                ..requirement
            },
            RegistryErrorKind::StableKeyMismatch,
        ),
        (
            NativeHandlerRequirement {
                version: "2",
                ..requirement
            },
            RegistryErrorKind::VersionMismatch,
        ),
        (
            NativeHandlerRequirement {
                argument_schema_digest: [8; 32],
                ..requirement
            },
            RegistryErrorKind::ArgumentSchemaMismatch,
        ),
        (
            NativeHandlerRequirement {
                determinism_note: "different",
                ..requirement
            },
            RegistryErrorKind::DeterminismNoteMismatch,
        ),
        (
            NativeHandlerRequirement {
                owner: "different",
                ..requirement
            },
            RegistryErrorKind::OwnerMismatch,
        ),
        (
            NativeHandlerRequirement {
                ir_insufficiency: "different",
                ..requirement
            },
            RegistryErrorKind::IrInsufficiencyMismatch,
        ),
        (
            NativeHandlerRequirement {
                removal_condition: "different",
                ..requirement
            },
            RegistryErrorKind::RemovalConditionMismatch,
        ),
    ] {
        assert_eq!(registry.audit(&[changed]).unwrap_err().kind(), expected);
    }

    let disabled_unregistered = NativeHandlerRequirement {
        id: NativeHandlerId::new(99).unwrap(),
        enabled: false,
        ..requirement
    };
    registry.audit(&[disabled_unregistered]).unwrap();

    let missing = NativeHandlerRequirement {
        id: NativeHandlerId::new(99).unwrap(),
        ..requirement
    };
    assert_eq!(
        registry.audit(&[missing]).unwrap_err().kind(),
        RegistryErrorKind::MissingRegistration
    );
    assert_eq!(
        registry
            .audit(&[NativeHandlerRequirement {
                domain: HandlerDomain::Activity,
                ..requirement
            }])
            .unwrap_err()
            .kind(),
        RegistryErrorKind::UnsupportedDomain
    );
    assert_eq!(
        registry
            .audit(&[NativeHandlerRequirement {
                version: " ",
                ..requirement
            }])
            .unwrap_err()
            .kind(),
        RegistryErrorKind::InvalidRequirement
    );
}

#[test]
fn registry_rejects_noncanonical_and_incomplete_static_metadata() {
    let mut incomplete = REGISTRATIONS[0];
    incomplete.owner = " ";
    let incomplete = Box::leak(vec![incomplete].into_boxed_slice());
    assert_eq!(
        NativeHandlerRegistry::new("native-registry-v1", incomplete)
            .unwrap_err()
            .kind(),
        RegistryErrorKind::InvalidRegistration
    );
    let duplicate = Box::leak(vec![REGISTRATIONS[0], REGISTRATIONS[0]].into_boxed_slice());
    assert_eq!(
        NativeHandlerRegistry::new("native-registry-v1", duplicate)
            .unwrap_err()
            .kind(),
        RegistryErrorKind::NonCanonicalRegistration
    );
}

#[test]
fn production_registry_is_explicitly_empty_after_the_v1a_review() {
    let registry = starclock_rules::registry::production();
    assert_eq!(registry.revision(), "native-registry-v1");
    assert!(registry.battle(HANDLER_ID).is_none());
    registry.audit(&[]).unwrap();
}

struct ProgramFixture {
    id: starclock_combat::ProgramId,
    steps: Vec<ProgramStep>,
}

impl ProgramLookup for ProgramFixture {
    fn program_steps(&self, id: starclock_combat::ProgramId) -> Option<&[ProgramStep]> {
        (id == self.id).then_some(&self.steps)
    }
}

#[test]
fn synthetic_native_and_equivalent_ir_emit_the_same_shape() {
    let program = starclock_combat::ProgramId::new(1).unwrap();
    let expected_value = RuleValue::Integer(9);
    let fixture = ProgramFixture {
        id: program,
        steps: vec![ProgramStep::Operation(
            RuleOperationTemplate::EmitRuleEvent {
                code: 41,
                value: Some(ValueExpr::Literal(expected_value.clone())),
            },
        )],
    };
    let input = evaluation_input();
    let ir = evaluate_program(&fixture, program, input, EvaluationBudget::STANDARD).unwrap();
    let native = (REGISTRATIONS[0].handler)(BattleHandlerInput {
        rule: input,
        arguments: &[expected_value],
    })
    .unwrap();
    assert_eq!(native.emissions(), ir);
}

fn evaluation_input<'a>() -> RuleEvaluationInput<'a> {
    RuleEvaluationInput {
        event_kind: RuleEventKind::Action,
        cause: RuleCause {
            owner: None,
            actor: None,
            applier: None,
            target: None,
            source: Some(SourceDefinitionId::new(1).unwrap()),
        },
        occurrence: RuleOccurrence {
            rule_instance: RuleInstanceId::new(1).unwrap(),
            event: EventId::new(1).unwrap(),
            hit: None,
            target: None,
            ability: Some(AbilityId::new(1).unwrap()),
            action: Some(ActionId::new(1).unwrap()),
            turn_event: Some(EventId::new(1).unwrap()),
            wave: WaveInstanceId::new(1).unwrap(),
        },
        source_tags: &[],
        slots: &[],
        selectors: &[],
        stat_reader: None,
    }
}
