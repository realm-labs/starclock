//! Original-entry Fire admission and source-owned Burn over shared Rule IR.
use crate::{
    digest::Encoder,
    divergent_universe::{
        DivergentUniverseBattleAssemblyError,
        battle_participant_element::basic_element,
        battle_passive_bindings::{PassiveBindings, bind_passives},
        contribution_snapshot::DivergentUniverseDifficultyProtocolSnapshot,
        weighted_curio::deflagration::DeflagrationBaseDamagePolicy,
    },
};
use starclock_combat::{
    CauseActorKind, DamageKind, DispelCategory, DotDetonationFilter, DotDetonationScope,
    DotDetonationSelection, DotFamily, DurationClock, EffectCategory, EffectDefinitionId,
    EffectRuntimeTemplate, EffectSnapshotPolicy, EffectStackPolicy, EffectTickPhase,
    ParticipantSource, ParticipantSpec, ProgramId, Rounding, RuleBundleId, RuleId, Scalar,
    SelectorId, SourceDefinitionId, StateSlotDefinitionId, TeamSide, TriggerId,
    catalog::{
        action::AbilityTag,
        builder::CombatCatalogBuilder,
        definition::{
            EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    formula::model::CombatElement,
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, Comparison, ConditionExpr, EventFilter, OnceScope,
        ProgramStep, ReactionPriority, RuleDamageClass, RuleDotSelection, RuleEffectChancePolicy,
        RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue, RuleValueKind, SourceClass,
        StateSlotDef, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_deflagrations::{
        WeightedCurioDeflagrationDefinition, WeightedCurioDeflagrationPolicy,
    },
};

const EFFECT: u32 = 0x7f75_0001;

struct OriginalFireRoster {
    formations: Vec<u8>,
    digest: [u8; 32],
}

/// Construct the authored original-Fire policy from immutable mapped originals.
/// Does not admit Activity equipment, Forge services or reference dispositions.
///
/// Proves entry membership through canonical Basic scaling-damage elements.
/// Formations must be unique originals in 0..=3; linked actors cannot borrow
/// these rules. Captures alive/present original Fire count at BattleStarted,
/// once for the battle, then retains it across roster and presence changes.
/// Attack ActionResolved applies one Guaranteed two-turn Fire Burn per distinct
/// living/present opposing target. Replacement is deliberately cross-caster,
/// not claimed as released ReplaceByCaster parity. Only own-source natural
/// DotTick detonates other ordinary/Break Burns; external detonation never chains.
///
/// Returns new participant specs and preserves existing passives. Never mutates
/// live battle or Activity state. Discard the builder if construction fails:
/// partial definitions can have been appended. Normal equipment remains gated.
pub fn bind_mapped_deflagration_policy(
    builder: &mut CombatCatalogBuilder,
    definition: &WeightedCurioDeflagrationDefinition,
    core: &SimulationCatalog,
    players: &[ParticipantSpec],
    protocol: &DivergentUniverseDifficultyProtocolSnapshot,
    assembly_digest: [u8; 32],
) -> Result<Vec<ParticipantSpec>, DivergentUniverseBattleAssemblyError> {
    if players.is_empty()
        || players.len() > 4
        || definition.burn_fractions_millionths != [500_000, 1_000_000, 1_500_000, 2_000_000]
        || definition.damage_multiplier_millionths != 2_000_000
        || definition.duration_turns != 2
        || !definition
            .runtime_policy_note
            .starts_with("VersionedProjectPolicy:")
        || definition.runtime_replacement_condition.trim().is_empty()
    {
        return Err(invalid());
    }
    let WeightedCurioDeflagrationPolicy::VersionedProjectPolicyOriginalFireAfterActionNaturalTickBurns = definition.policy;
    let base =
        DeflagrationBaseDamagePolicy::from_authored(definition, protocol).map_err(|_| invalid())?;
    // Validate the complete captured DoT magnitude, not merely base damage.
    let factor = Scalar::ONE
        .checked_add(Scalar::from_scaled(
            protocol.maximum_hp_increase_millionths(),
        ))
        .map_err(|_| invalid())?;
    for ratio in &definition.hp_ratios_millionths {
        Scalar::from_scaled(definition.fixed_base_damage_millionths)
            .checked_mul(Scalar::from_scaled(*ratio), Rounding::Floor)
            .and_then(|value| value.checked_mul(factor, Rounding::Floor))
            .and_then(|value| {
                value.checked_mul(
                    Scalar::from_scaled(definition.damage_multiplier_millionths),
                    Rounding::Floor,
                )
            })
            .map_err(|_| invalid())?;
    }
    let mut originals = players.iter().collect::<Vec<_>>();
    originals.sort_by_key(|player| player.formation());
    if originals.iter().any(|player| {
        player.side() != TeamSide::Player
            || player.source() != ParticipantSource::Player
            || player.formation().get() > 3
    }) || originals
        .windows(2)
        .any(|pair| pair[0].formation() == pair[1].formation())
    {
        return Err(invalid());
    }
    let mut membership =
        Encoder::new(b"starclock.divergent-universe.deflagration.original-fire-membership");
    let mut fire = Vec::new();
    for player in originals {
        let eligible = basic_element(core, player)? == CombatElement::Fire;
        membership.u32(u32::from(player.formation().get()));
        membership.digest(player.combatant().digest().bytes());
        membership.u32(u32::from(eligible));
        if eligible {
            fire.push(player.formation().get());
        }
    }
    if fire.is_empty() {
        return Ok(players.to_vec());
    }
    let roster = OriginalFireRoster {
        formations: fire,
        digest: membership.finish(),
    };
    let effect = EffectDefinitionId::new(EFFECT).ok_or_else(invalid)?;
    builder.add_effect(
        EffectDefinition::new(effect, vec![], vec![])
            .with_runtime_template(
                EffectRuntimeTemplate::new(
                    EffectCategory::Dot,
                    DispelCategory::DispellableDebuff,
                    1,
                    Some(ValueExpr::Literal(RuleValue::Integer(i64::from(
                        definition.duration_turns,
                    )))),
                    DurationClock::TargetTurnEnd,
                    EffectTickPhase::TurnStart,
                    EffectStackPolicy::Replace,
                )
                .ok_or_else(invalid)?
                .with_comparison(
                    Some(ValueExpr::Multiply {
                        lhs: Box::new(base.expression().clone()),
                        rhs: Box::new(value(definition.damage_multiplier_millionths)),
                        rounding: Rounding::Floor,
                    }),
                    0,
                )
                .with_snapshot(EffectSnapshotPolicy::SourceSnapshotTargetDynamic)
                .with_dot(CombatElement::Fire, None)
                .ok_or_else(invalid)?,
            )
            .with_dot_family(DotFamily::Burn),
    );
    players
        .iter()
        .map(|player| {
            if !roster.formations.contains(&player.formation().get()) {
                return Ok(player.clone());
            }
            bind_owner(
                builder,
                definition,
                player,
                &roster,
                effect,
                base.identity(),
                assembly_digest,
            )
        })
        .collect()
}

fn bind_owner(
    builder: &mut CombatCatalogBuilder,
    definition: &WeightedCurioDeflagrationDefinition,
    player: &ParticipantSpec,
    roster: &OriginalFireRoster,
    effect: EffectDefinitionId,
    base_digest: [u8; 32],
    assembly_digest: [u8; 32],
) -> Result<ParticipantSpec, DivergentUniverseBattleAssemblyError> {
    let ordinal = u32::from(player.formation().get()) + 1;
    let raw = |base: u32, offset: u32| {
        ordinal
            .checked_mul(32)
            .and_then(|n| n.checked_add(base))
            .and_then(|n| n.checked_add(offset))
            .ok_or_else(invalid)
    };
    let selector = |offset| SelectorId::new(raw(0x7f70_0000, offset)?).ok_or_else(invalid);
    let owner = selector(0)?;
    let present = selector(1)?;
    let transformed = selector(2)?;
    let acting = selector(3)?;
    let team = selector(4)?;
    let linked = selector(5)?;
    let members = selector(6)?;
    let targets = selector(7)?;
    let cleanup = selector(8)?;
    let count = StateSlotDefinitionId::new(raw(0x7f76_0000, 0)?).ok_or_else(invalid)?;
    let all_team = select(
        RuleSelectorOrigin::Team,
        RuleSelectorSide::Same,
        RuleLifePredicate::Any,
        RulePresencePredicate::Any,
        u16::MAX,
    )?;
    builder.add_selector(SelectorDefinition::new(team).with_rule_units(all_team.clone()));
    builder.add_selector(
        SelectorDefinition::new(linked)
            .with_rule_units(all_team.with_predicates(vec![RuleSelectorPredicate::OwnedBy(team)])),
    );
    let original = vec![
        RuleSelectorPredicate::FormationRange {
            minimum: player.formation().get(),
            maximum: player.formation().get(),
        },
        RuleSelectorPredicate::Excludes(linked),
    ];
    builder.add_selector(
        SelectorDefinition::new(owner).with_rule_units(
            select(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleLifePredicate::Any,
                RulePresencePredicate::Any,
                1,
            )?
            .with_predicates(original.clone()),
        ),
    );
    for (id, presence) in [
        (present, RulePresencePredicate::Present),
        (transformed, RulePresencePredicate::Transformed),
    ] {
        builder.add_selector(SelectorDefinition::new(id).with_rule_units(select(
            RuleSelectorOrigin::Owner,
            RuleSelectorSide::Same,
            RuleLifePredicate::Alive,
            presence,
            1,
        )?));
    }
    builder.add_selector(
        SelectorDefinition::new(acting).with_rule_units(
            select(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Any,
                1,
            )?
            .with_candidate_union(vec![present, transformed])
            .ok_or_else(invalid)?
            .with_predicates(original),
        ),
    );
    let mut selectors = vec![
        owner,
        present,
        transformed,
        acting,
        team,
        linked,
        members,
        targets,
        cleanup,
    ];
    let mut union = Vec::new();
    for (index, formation) in roster.formations.iter().enumerate() {
        let id = selector(
            u32::try_from(index)
                .map_err(|_| invalid())?
                .checked_add(9)
                .ok_or_else(invalid)?,
        )?;
        builder.add_selector(
            SelectorDefinition::new(id).with_rule_units(
                select(
                    RuleSelectorOrigin::Team,
                    RuleSelectorSide::Same,
                    RuleLifePredicate::Alive,
                    RulePresencePredicate::Present,
                    1,
                )?
                .with_predicates(vec![
                    RuleSelectorPredicate::FormationRange {
                        minimum: *formation,
                        maximum: *formation,
                    },
                    RuleSelectorPredicate::Excludes(linked),
                ]),
            ),
        );
        union.push(id);
        selectors.push(id);
    }
    builder.add_selector(
        SelectorDefinition::new(members).with_rule_units(
            select(
                RuleSelectorOrigin::Team,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                4,
            )?
            .with_candidate_union(union)
            .ok_or_else(invalid)?,
        ),
    );
    for (id, origin, life, presence, maximum) in [
        (
            targets,
            RuleSelectorOrigin::EventTargets,
            RuleLifePredicate::Alive,
            RulePresencePredicate::Present,
            u16::MAX,
        ),
        (
            cleanup,
            RuleSelectorOrigin::Encounter,
            RuleLifePredicate::Any,
            RulePresencePredicate::Any,
            u16::MAX,
        ),
    ] {
        builder.add_selector(SelectorDefinition::new(id).with_rule_units(select(
            origin,
            RuleSelectorSide::Opposing,
            life,
            presence,
            maximum,
        )?));
    }
    selectors.sort_unstable();
    let capture = ProgramId::new(raw(0x7f71_0000, 0)?).ok_or_else(invalid)?;
    let apply = ProgramId::new(raw(0x7f71_0000, 1)?).ok_or_else(invalid)?;
    let detonate = ProgramId::new(raw(0x7f71_0000, 2)?).ok_or_else(invalid)?;
    let clear = ProgramId::new(raw(0x7f71_0000, 3)?).ok_or_else(invalid)?;
    let mut fraction = value(0);
    for (index, scaled) in definition
        .burn_fractions_millionths
        .iter()
        .enumerate()
        .rev()
    {
        fraction = ValueExpr::Choose {
            condition: Box::new(ConditionExpr::Compare {
                operator: Comparison::Equal,
                lhs: Box::new(ValueExpr::Slot(count)),
                rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(
                    i64::try_from(index + 1).map_err(|_| invalid())?,
                ))),
            }),
            when_true: Box::new(value(*scaled)),
            when_false: Box::new(fraction),
        };
    }
    for (id, references, effects, steps) in [
        (
            capture,
            vec![members],
            vec![],
            vec![op(RuleOperationTemplate::SetSlot {
                slot: count,
                value: ValueExpr::SelectorCount(members),
            })],
        ),
        (
            apply,
            vec![acting, targets],
            vec![effect],
            vec![op(RuleOperationTemplate::ApplyEffect {
                selector: targets,
                effect,
                stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            })],
        ),
        (
            detonate,
            vec![owner, targets],
            vec![effect],
            vec![op(RuleOperationTemplate::DetonateDot {
                selector: targets,
                fraction,
                required_tag: None,
                selection: RuleDotSelection::Filtered {
                    filter: DotDetonationFilter::family(DotFamily::Burn).excluding(effect),
                    selection: DotDetonationSelection::All,
                    scope: DotDetonationScope::OrdinaryAndBreakEffects,
                },
            })],
        ),
        (
            clear,
            vec![cleanup],
            vec![effect],
            vec![
                op(RuleOperationTemplate::SetSlot {
                    slot: count,
                    value: ValueExpr::Literal(RuleValue::Integer(0)),
                }),
                op(RuleOperationTemplate::RemoveEffect {
                    selector: cleanup,
                    effect,
                }),
            ],
        ),
    ] {
        builder.add_program(
            ProgramDefinition::new(id, vec![], references, effects, vec![]).with_steps(steps),
        );
    }
    let mut identity = Encoder::new(
        b"starclock.divergent-universe.deflagration.original-fire-after-action-natural-tick-policy",
    );
    identity.digest(assembly_digest);
    identity.digest(base_digest);
    identity.digest(roster.digest);
    identity.text(&definition.key);
    identity.u32(u32::from(player.formation().get()));
    for fraction in definition.burn_fractions_millionths {
        identity.i64(fraction);
    }
    identity.i64(definition.damage_multiplier_millionths);
    identity.u32(u32::from(definition.duration_turns));
    identity.text(&definition.runtime_policy_note);
    identity.text(&definition.runtime_replacement_condition);
    let identity = identity.finish();
    let source_id = SourceDefinitionId::new(raw(0x7f74_0000, 0)?).ok_or_else(invalid)?;
    let source = RuleSource::new(source_id, SourceClass::Mode, vec![], identity);
    let mut triggers = Vec::new();
    for (offset, point, phase, filter, once_scope, program) in [
        (
            0,
            RuleEventPoint::BattleStarted,
            TriggerPhase::AfterEvent,
            EventFilter::default(),
            OnceScope::Battle,
            capture,
        ),
        (
            1,
            RuleEventPoint::ActionResolved,
            TriggerPhase::AfterAction,
            EventFilter {
                // Complete-action causes have no per-hit applier/source.
                // The raw actor and aggregate validated tags anchor admission;
                // the shared reaction emitter supplies its original applier.
                actor_kind: Some(CauseActorKind::Unit),
                actor_selector: Some(acting),
                ability_tag: Some(AbilityTag::Attack),
                has_action: Some(true),
                ..EventFilter::default()
            },
            OnceScope::Action,
            apply,
        ),
        (
            2,
            RuleEventPoint::DamageApplied,
            TriggerPhase::AfterEvent,
            EventFilter {
                source: Some(source_id),
                applier_selector: Some(owner),
                element: Some(CombatElement::Fire),
                damage_class: Some(RuleDamageClass::Dot),
                damage_kind: Some(DamageKind::DotTick),
                ..EventFilter::default()
            },
            OnceScope::Event,
            detonate,
        ),
        (
            3,
            RuleEventPoint::BattleWon,
            TriggerPhase::AfterEvent,
            EventFilter::default(),
            OnceScope::Event,
            clear,
        ),
        (
            4,
            RuleEventPoint::BattleLost,
            TriggerPhase::AfterEvent,
            EventFilter::default(),
            OnceScope::Event,
            clear,
        ),
    ] {
        triggers.push(TriggerDef {
            id: TriggerId::new(raw(0x7f77_0000, offset)?).ok_or_else(invalid)?,
            event: point.kind(),
            event_point: point,
            phase,
            filter,
            condition: ConditionExpr::Literal(true),
            once_scope,
            priority: ReactionPriority::new(0),
            program,
        });
    }
    let rule = RuleId::new(raw(0x7f72_0000, 0)?).ok_or_else(invalid)?;
    let bundle = RuleBundleId::new(raw(0x7f73_0000, 0)?).ok_or_else(invalid)?;
    builder.add_rule(
        RuleDefinition::new(rule, vec![capture, apply, detonate, clear], selectors).with_runtime(
            BattleRuleDefinition::new(
                source.clone(),
                vec![StateSlotDef::new(
                    count,
                    RuleValueKind::Integer,
                    BattleRuleScope::Battle,
                    RuleValue::Integer(0),
                )],
                triggers,
                None,
            ),
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
    side: RuleSelectorSide,
    life: RuleLifePredicate,
    presence: RulePresencePredicate,
    maximum: u16,
) -> Result<RuleUnitSelector, DivergentUniverseBattleAssemblyError> {
    RuleUnitSelector::new(
        origin,
        side,
        life,
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
fn value(scaled: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(scaled)))
}
fn op(operation: RuleOperationTemplate) -> ProgramStep {
    ProgramStep::Operation(operation)
}
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
