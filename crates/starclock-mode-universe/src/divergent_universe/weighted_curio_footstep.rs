//! Explicit, replaceable HP-loss contribution; production equipment is not admitted.
use crate::{
    digest::Encoder,
    divergent_universe::{
        DivergentUniverseBattleAssemblyError,
        battle_passive_bindings::{PassiveBindings, bind_passives},
    },
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    ParticipantSpec, ProgramId, Rounding, RuleBundleId, RuleId, Scalar, SelectorId,
    SourceDefinitionId, StateSlotDefinitionId, TeamSide, TriggerId,
    catalog::{
        builder::CombatCatalogBuilder,
        definition::{ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    modifier::model::StatQuerySubject,
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, Comparison, ConditionExpr, EventFilter,
        EventValueProperty, OnceScope, ProgramStep, ReactionPriority, ResourceUpdateKind,
        RuleEventPoint, RuleOperationTemplate, RuleResourceKind, RuleSource, RuleValue,
        RuleValueKind, SourceClass, StateSlotDef, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::catalog::SimulationCatalog;

/// Invalid immutable constructor inputs, rejected before catalog mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HpLossPointPolicyError {
    InvalidLossFraction,
}
impl core::fmt::Display for HpLossPointPolicyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "HP-loss point policy error: {self:?}")
    }
}
impl std::error::Error for HpLossPointPolicyError {}

/// Caller-bound immutable operands, not evidence of source parity or equipment.
#[derive(Clone, Debug)]
pub struct HpLossPointPolicy {
    loss_fraction: Scalar,
    identity: [u8; 32],
}
impl HpLossPointPolicy {
    /// A threshold is live maximum HP times a fraction in `(0, 1]`.
    /// The caller binds provenance, policy notes and other external inputs in
    /// identity. The shared checked backend and explicit floor rules govern
    /// every expression; no float, default fraction or live Activity query exists.
    pub fn new(loss_fraction: Scalar, identity: [u8; 32]) -> Result<Self, HpLossPointPolicyError> {
        if loss_fraction <= Scalar::ZERO || loss_fraction > Scalar::ONE {
            return Err(HpLossPointPolicyError::InvalidLossFraction);
        }
        Ok(Self {
            loss_fraction,
            identity,
        })
    }
}

/// Compile the HP-loss clause for one original player (formation 0..=3).
/// Entry Destruction/Remembrance Path proof comes from the supplied build
/// catalog; other Paths return the unchanged participant without definitions.
/// Original identity survives transformation, but owned linked units cannot
/// borrow the contribution through inherited forms, slots, rules or sources.
///
/// VersionedProjectPolicy: observe negative effective DamageApplied (ordinary,
/// True, DoT and Break) and HpChanged (explicit consumption) at AfterEvent,
/// priority zero, once per event. Healing, shields and maximum-HP clamps do not
/// add loss. Track a battle-local absolute HP remainder, not a health percentage.
/// On each loss, floor live-capacity times fraction to six places, then floor
/// (remainder + effective loss) / threshold to integral points. Consume all
/// crossed thresholds before capped gain, including discarded overflow.
/// Healing and waves retain remainder; terminal win/loss explicitly clears it.
/// Lethal loss counts while the original is still Present/Transformed.
///
/// Programs read one immutable trigger snapshot: the remainder write cannot
/// affect the same proposal's gain expression. Arithmetic and out-of-u16 point
/// requests enter the shared deterministic rollback fault path, never saturate.
/// Discard the builder on construction errors (definitions may be appended).
/// This explicit API is not normal equipment/Forge admission, the Skill-damage
/// clause or a new command processor; no live battle/Activity state is mutated.
pub fn bind_hp_loss_point_policy(
    builder: &mut CombatCatalogBuilder,
    core: &SimulationCatalog,
    player: &ParticipantSpec,
    policy: &HpLossPointPolicy,
    assembly_digest: [u8; 32],
) -> Result<ParticipantSpec, DivergentUniverseBattleAssemblyError> {
    if player.side() != TeamSide::Player || player.formation().get() > 3 {
        return Err(invalid());
    }
    let character = core
        .build_catalog()
        .character(player.combatant().form())
        .ok_or_else(invalid)?;
    if !matches!(
        character.path(),
        CombatPath::Destruction | CombatPath::Remembrance
    ) {
        return Ok(player.clone());
    }
    let ordinal = u32::from(player.formation().get()) + 1;
    let raw = |base: u32, offset: u32| {
        ordinal
            .checked_mul(16)
            .and_then(|value| value.checked_add(base))
            .and_then(|value| value.checked_add(offset))
            .ok_or_else(invalid)
    };
    let selector = |offset| SelectorId::new(raw(0x7f20_0000, offset)?).ok_or_else(invalid);
    let owner = selector(0)?;
    let team = selector(1)?;
    let linked = selector(2)?;
    let present = selector(3)?;
    let transformed = selector(4)?;
    let collect = ProgramId::new(raw(0x7f21_0000, 0)?).ok_or_else(invalid)?;
    let reset = ProgramId::new(raw(0x7f21_0000, 1)?).ok_or_else(invalid)?;
    let rule = RuleId::new(raw(0x7f22_0000, 0)?).ok_or_else(invalid)?;
    let bundle = RuleBundleId::new(raw(0x7f23_0000, 0)?).ok_or_else(invalid)?;
    let source_id = SourceDefinitionId::new(raw(0x7f24_0000, 0)?).ok_or_else(invalid)?;
    let remainder = StateSlotDefinitionId::new(raw(0x7f25_0000, 0)?).ok_or_else(invalid)?;
    let all = select(
        RuleSelectorOrigin::Team,
        RulePresencePredicate::Any,
        u16::MAX,
    )?;
    builder.add_selector(SelectorDefinition::new(team).with_rule_units(all.clone()));
    builder.add_selector(
        SelectorDefinition::new(linked)
            .with_rule_units(all.with_predicates(vec![RuleSelectorPredicate::OwnedBy(team)])),
    );
    for (id, presence) in [
        (present, RulePresencePredicate::Present),
        (transformed, RulePresencePredicate::Transformed),
    ] {
        builder.add_selector(SelectorDefinition::new(id).with_rule_units(select(
            RuleSelectorOrigin::Owner,
            presence,
            1,
        )?));
    }
    builder.add_selector(
        SelectorDefinition::new(owner).with_rule_units(
            select(RuleSelectorOrigin::Owner, RulePresencePredicate::Any, 1)?
                .with_candidate_union(vec![present, transformed])
                .ok_or_else(invalid)?
                .with_predicates(vec![
                    RuleSelectorPredicate::FormationRange {
                        minimum: player.formation().get(),
                        maximum: player.formation().get(),
                    },
                    RuleSelectorPredicate::Excludes(linked),
                ]),
        ),
    );
    let selectors = vec![owner, team, linked, present, transformed];
    let total = ValueExpr::Subtract(Box::new(ValueExpr::Slot(remainder)), Box::new(delta()));
    let threshold = multiply(
        ValueExpr::QueryMaximumHp(StatQuerySubject::Owner),
        literal(policy.loss_fraction),
    );
    let points = ValueExpr::Convert {
        value: Box::new(ValueExpr::Divide {
            lhs: Box::new(total.clone()),
            rhs: Box::new(threshold.clone()),
            rounding: Rounding::Floor,
        }),
        target: RuleValueKind::Integer,
        rounding: Rounding::Floor,
    };
    let scalar_points = ValueExpr::Convert {
        value: Box::new(points),
        target: RuleValueKind::Scalar,
        rounding: Rounding::Floor,
    };
    let residue = ValueExpr::Subtract(
        Box::new(total),
        Box::new(multiply(scalar_points.clone(), threshold)),
    );
    builder.add_program(
        ProgramDefinition::new(collect, vec![], selectors.clone(), vec![], vec![]).with_steps(
            vec![
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: remainder,
                    value: residue,
                }),
                ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: RuleResourceKind::SkillPoints,
                    update: ResourceUpdateKind::Gain,
                    amount: scalar_points,
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                }),
            ],
        ),
    );
    builder.add_program(
        ProgramDefinition::new(reset, vec![], vec![], vec![], vec![]).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot: remainder,
                value: literal(Scalar::ZERO),
            }),
        ]),
    );
    let mut digest =
        Encoder::new(b"starclock.divergent-universe.footstep.hp-loss-current-capacity-policy");
    digest.digest(assembly_digest);
    digest.digest(policy.identity);
    digest.i64(policy.loss_fraction.scaled());
    digest.digest(
        core.build_catalog()
            .character_digest(player.combatant().form())
            .ok_or_else(invalid)?
            .bytes(),
    );
    digest.u8(player.formation().get());
    let identity = digest.finish();
    let source = RuleSource::new(source_id, SourceClass::Mode, vec![], identity);
    let mut triggers = vec![];
    for (offset, point, program) in [
        (0, RuleEventPoint::DamageApplied, collect),
        (1, RuleEventPoint::HpChanged, collect),
        (2, RuleEventPoint::BattleWon, reset),
        (3, RuleEventPoint::BattleLost, reset),
    ] {
        triggers.push(TriggerDef {
            id: TriggerId::new(raw(0x7f26_0000, offset)?).ok_or_else(invalid)?,
            event: point.kind(),
            event_point: point,
            phase: TriggerPhase::AfterEvent,
            filter: if program == collect {
                EventFilter {
                    target_selector: Some(owner),
                    ..EventFilter::default()
                }
            } else {
                EventFilter::default()
            },
            condition: if program == collect {
                ConditionExpr::All(
                    vec![
                        ConditionExpr::SelectorCardinality {
                            selector: owner,
                            operator: Comparison::Equal,
                            count: 1,
                        },
                        ConditionExpr::Compare {
                            lhs: Box::new(delta()),
                            operator: Comparison::Less,
                            rhs: Box::new(literal(Scalar::ZERO)),
                        },
                    ]
                    .into_boxed_slice(),
                )
            } else {
                // Reset also applies to absent/defeated/inherited instances.
                ConditionExpr::Literal(true)
            },
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program,
        });
    }
    let slot = StateSlotDef::new(
        remainder,
        RuleValueKind::Scalar,
        BattleRuleScope::Battle,
        RuleValue::Scalar(Scalar::ZERO),
    )
    .with_bounds(
        RuleValue::Scalar(Scalar::ZERO),
        RuleValue::Scalar(Scalar::from_scaled(i64::MAX)),
    );
    builder.add_rule(
        RuleDefinition::new(rule, vec![collect, reset], selectors).with_runtime(
            BattleRuleDefinition::new(source.clone(), vec![slot], triggers, None),
        ),
    );
    builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
    bind_passives(
        player,
        &PassiveBindings {
            rule_bundles: vec![bundle],
            sources: vec![source],
            ..PassiveBindings::default()
        },
        identity,
    )
}

fn select(
    origin: RuleSelectorOrigin,
    presence: RulePresencePredicate,
    maximum: u16,
) -> Result<RuleUnitSelector, DivergentUniverseBattleAssemblyError> {
    RuleUnitSelector::new(
        origin,
        RuleSelectorSide::Same,
        RuleLifePredicate::Any,
        presence,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        if origin == RuleSelectorOrigin::Owner {
            RuleSelectorChoice::First
        } else {
            RuleSelectorChoice::All
        },
        None,
        false,
    )
    .ok_or_else(invalid)
}
fn literal(value: Scalar) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(value))
}
fn delta() -> ValueExpr {
    ValueExpr::ReadEventProperty(EventValueProperty::HpChangeAmount)
}
fn multiply(lhs: ValueExpr, rhs: ValueExpr) -> ValueExpr {
    ValueExpr::Multiply {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        rounding: Rounding::Floor,
    }
}
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
