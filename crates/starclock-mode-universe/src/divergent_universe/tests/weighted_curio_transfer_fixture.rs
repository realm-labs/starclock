//! Controlled grants and damage retain the production special-shield rules.
use crate::divergent_universe::DivergentUniverseAssembledBattle;
use starclock_combat::{
    AbilityId, ActionEventData, ActionGauge, AssemblyDigest, Battle, BattleEvent, BattleEventKind,
    BattleSeed, BattleSpec, CombatantSpecDigest, Command, ConcedePolicy, DispelCategory,
    DurationClock, EffectCategory, EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase,
    Energy, FormationIndex, Hp, LifeState, LinkedEntityKind, LinkedUnitCatalogDefinition,
    LinkedUnitDefinition, OwnerLinkPolicy, ParticipantInitialState, ParticipantSource,
    ParticipantSpec, PresenceState, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    Scalar, Speed, TeamResourceSpec, TeamSide, UnitId, WaveLinkPolicy,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, HitOperationDefinition,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, TargetInvalidationPolicy,
            TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{
        ProgramStep, RuleEffectChancePolicy, RuleOperationTemplate, RuleValue, ValueExpr,
    },
};

pub(super) const SPECIAL: u32 = 0x7ef8_0001;
pub(super) const ORDINARY: u32 = 0x7d96_0001;
pub(super) const GRANT: u32 = 0x7d90_0001;
pub(super) const REMOVE: u32 = 0x7d90_0002;
pub(super) const ABSORB: u32 = 0x7d90_0003;
pub(super) const SHRINK_HP: u32 = 0x7d90_0004;
pub(super) const IDLE: u32 = 0x7d90_0005;
pub(super) const SUMMON: u32 = 0x7d90_0006;
const LINKED_IDLE: u32 = 0x7d90_0007;

pub(super) fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
fn scalar(amount: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(amount).unwrap(),
    ))
}
fn op(operation: RuleOperationTemplate) -> ProgramStep {
    ProgramStep::Operation(operation)
}

#[derive(Clone, Copy)]
pub(super) struct Probe {
    pub(super) hp: [i64; 4],
    pub(super) grant: i64,
    pub(super) repetitions: u16,
    pub(super) initial_hp: i64,
    pub(super) absent: Option<u8>,
    pub(super) defeated: Option<u8>,
    pub(super) linked_kind: Option<LinkedEntityKind>,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            hp: [1000; 4],
            grant: 400,
            repetitions: 1,
            initial_hp: 1,
            absent: None,
            defeated: None,
            linked_kind: None,
        }
    }
}

pub(super) fn scenario(source: &DivergentUniverseAssembledBattle, input: Probe) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(source.combat_catalog(), [0x96; 32]);
    let recipient = id(0x7d91_0001);
    builder.add_selector(
        SelectorDefinition::new(recipient).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::Formation,
                0,
                1,
                RuleEmptyPoolPolicy::NoOp,
                RuleSelectorChoice::First,
                None,
                false,
            )
            .unwrap(),
        ),
    );
    builder.add_effect(
        EffectDefinition::new(id(ORDINARY), vec![], vec![]).with_runtime(
            EffectRuntimeDefinition::new(
                EffectCategory::Shield,
                DispelCategory::NonDispellable,
                1,
                None,
                DurationClock::Permanent,
                EffectTickPhase::None,
                EffectStackPolicy::Replace,
            )
            .unwrap(),
        ),
    );
    if source.combat_catalog().effect(id(SPECIAL)).is_none() {
        // A dormant removal target keeps the synthetic actions well-formed;
        // it supplies no producer, marker binding or execution for control parties.
        builder.add_effect(EffectDefinition::new(id(SPECIAL), vec![], vec![]));
    }
    let abilities = [GRANT, REMOVE, ABSORB, SHRINK_HP, IDLE, SUMMON]
        .map(id::<AbilityId>)
        .to_vec();
    for (offset, raw) in [GRANT, REMOVE, ABSORB, SHRINK_HP, IDLE, SUMMON, LINKED_IDLE]
        .into_iter()
        .enumerate()
    {
        let ordinal = u32::try_from(offset).unwrap() + 1;
        let selector = id(0x7d92_0000 + ordinal);
        let program = id(0x7d93_0000 + ordinal);
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(
                    if matches!(raw, IDLE | LINKED_IDLE) {
                        TargetRelation::SelfUnit
                    } else {
                        TargetRelation::Allied
                    },
                    TargetPattern::Single,
                )
                .unwrap(),
            ),
        );
        let steps = match raw {
            GRANT => {
                let mut steps = vec![op(RuleOperationTemplate::ApplyEffect {
                    selector: recipient,
                    effect: id(ORDINARY),
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                })];
                for _ in 0..input.repetitions {
                    steps.push(op(RuleOperationTemplate::Shield {
                        selector: recipient,
                        effect: id(ORDINARY),
                        amount: scalar(input.grant),
                    }));
                }
                steps
            }
            REMOVE => vec![op(RuleOperationTemplate::RemoveEffect {
                selector: recipient,
                effect: id(SPECIAL),
            })],
            ABSORB => vec![op(RuleOperationTemplate::TrueDamage {
                selector: recipient,
                amount: scalar(100),
            })],
            SHRINK_HP => vec![op(RuleOperationTemplate::ReduceMaximumHp {
                selector: recipient,
                amount: scalar(100),
                minimum_ratio: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(100_000))),
            })],
            SUMMON if input.linked_kind.is_some() => vec![op(RuleOperationTemplate::Summon {
                owner_selector: recipient,
                unit_definition: id(0x7d94_0002),
            })],
            _ => vec![],
        };
        builder.add_program(
            ProgramDefinition::new(
                program,
                vec![],
                vec![recipient],
                vec![id(ORDINARY), id(SPECIAL)],
                vec![],
            )
            .with_steps(steps),
        );
        let action = AbilityActionDefinition::new(
            if raw == LINKED_IDLE {
                match input.linked_kind {
                    Some(LinkedEntityKind::Summon) => AbilityKind::Summon,
                    Some(LinkedEntityKind::Memosprite) => AbilityKind::Memosprite,
                    _ => AbilityKind::Skill,
                }
            } else {
                AbilityKind::Basic
            },
            1,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .unwrap()
        .with_hits(vec![ActionHitDefinition::new(vec![
            HitOperationDefinition::Damage(
                OrdinaryDamageDefinition::new(
                    Scalar::ZERO,
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                )
                .unwrap(),
            ),
        ])])
        .unwrap();
        builder.add_ability(
            AbilityDefinition::new(id(raw), program, selector, vec![])
                .with_action(action)
                .with_programs(vec![
                    AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, program).unwrap(),
                ]),
        );
    }
    let mut participants = Vec::new();
    if let Some(kind) = input.linked_kind {
        let base = source
            .battle_spec()
            .participants()
            .iter()
            .find(|p| p.side() == TeamSide::Player && p.formation().get() == 0)
            .unwrap()
            .combatant();
        let form = id(0x7d94_0002);
        builder.add_unit(UnitDefinition::new(form, vec![id(LINKED_IDLE)], vec![]));
        let spec = ResolvedCombatantSpec::new(
            form,
            base.level(),
            Hp::new(1000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                vec![id(LINKED_IDLE)],
                base.rule_bundles().to_vec(),
                base.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([0x96; 32]).unwrap(),
        )
        .unwrap()
        .with_sources(base.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(base.modifier_bindings().to_vec())
        .unwrap();
        let linked = LinkedUnitDefinition::new(
            spec,
            id(0x7d98_0001),
            FormationIndex::new(4).unwrap(),
            kind,
            PresenceState::Present,
            if kind == LinkedEntityKind::SharedActor {
                None
            } else {
                Some(id(LINKED_IDLE))
            },
            ActionGauge::from_scaled(100_000_000).unwrap(),
            OwnerLinkPolicy::Depart,
            OwnerLinkPolicy::Depart,
            WaveLinkPolicy::Depart,
        )
        .unwrap();
        builder.add_linked_unit(LinkedUnitCatalogDefinition::new(form, linked).unwrap());
    }
    for original in source
        .battle_spec()
        .participants()
        .iter()
        .filter(|p| p.side() == TeamSide::Player)
    {
        let formation = original.formation().get();
        let base = original.combatant();
        let form = id(0x7d94_0010 + u32::from(formation));
        builder.add_unit(UnitDefinition::new(form, abilities.clone(), vec![]));
        let maximum = Hp::new(input.hp[usize::from(formation)]).unwrap();
        let spec = ResolvedCombatantSpec::new(
            form,
            base.level(),
            maximum,
            Speed::from_scaled(200_000_000 - i64::from(formation) * 10_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                abilities.clone(),
                base.rule_bundles().to_vec(),
                base.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([0x97; 32]).unwrap(),
        )
        .unwrap()
        .with_sources(base.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(base.modifier_bindings().to_vec())
        .unwrap();
        participants.push(
            ParticipantSpec::new(
                original.side(),
                original.formation(),
                original.source(),
                spec,
            )
            .with_initial_state(
                ParticipantInitialState::new(
                    Hp::new(if input.defeated == Some(formation) {
                        0
                    } else {
                        input.initial_hp.min(maximum.get())
                    })
                    .unwrap(),
                    maximum,
                    Energy::ZERO,
                    Energy::ZERO,
                    if input.defeated == Some(formation) {
                        LifeState::Defeated
                    } else {
                        LifeState::Alive
                    },
                    if input.absent == Some(formation) {
                        PresenceState::Reserved
                    } else {
                        PresenceState::Present
                    },
                )
                .unwrap(),
            )
            .unwrap(),
        );
    }
    let form = id(0x7d94_0001);
    let enemy = id(0x7d95_0001);
    let encounter = id(0x7d97_0001);
    builder.add_unit(UnitDefinition::new(form, vec![id(IDLE)], vec![]));
    builder.add_enemy(EnemyDefinition::new(enemy, form, vec![id(IDLE)]));
    let base = &participants[0];
    let spec = ResolvedCombatantSpec::new(
        form,
        base.combatant().level(),
        Hp::new(100_000).unwrap(),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![id(IDLE)], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([0x98; 32]).unwrap(),
    )
    .unwrap();
    participants.push(ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::EncounterEnemy(enemy),
        spec,
    ));
    builder.add_encounter(EncounterDefinition::new(encounter, vec![enemy], vec![]));
    let spec = BattleSpec::new(
        AssemblyDigest::new([0x99; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(builder.build().unwrap(), spec, BattleSeed::new([0x9a; 32])).unwrap()
}

pub(super) fn unit(battle: &Battle, formation: u8) -> UnitId {
    battle
        .view()
        .units_by_id()
        .find(|u| u.side() == TeamSide::Player && u.formation().get() == formation)
        .unwrap()
        .id()
}
pub(super) fn accept(battle: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let result = battle.apply(command).unwrap();
    assert!(result.fault().is_none(), "{:?}", result.fault());
    result.events().to_vec()
}
pub(super) fn idle(battle: &mut Battle) -> Vec<BattleEvent> {
    let command = battle
        .decision()
        .and_then(|d| {
            d.legal_commands()
                .iter()
                .find(|c| {
                    matches!(c, Command::UseAbility {ability,..} if ability.get() == IDLE)
                        || matches!(c, Command::StartBattle { .. })
                })
                .cloned()
        })
        .unwrap_or_else(|| battle.advance_command().unwrap());
    accept(battle, command)
}
pub(super) fn cast(
    battle: &mut Battle,
    formation: u8,
    raw: u32,
    recipient: UnitId,
) -> Vec<BattleEvent> {
    let actor = unit(battle, formation);
    let mut events = Vec::new();
    let mut started = false;
    for _ in 0..256 {
        let command = if !started {
            battle.decision().and_then(|d| {
                d.legal_commands()
                    .iter()
                    .find(|c| {
                        matches!(c, Command::UseAbility {actor: offered,ability,primary_target,..}
                if *offered == actor && ability.get() == raw && *primary_target == Some(recipient))
                    })
                    .cloned()
            })
        } else {
            None
        };
        let applied = if let Some(command) = command {
            started = true;
            accept(battle, command)
        } else {
            idle(battle)
        };
        let finished = applied.iter().any(|event| matches!(event.kind(), BattleEventKind::Action(ActionEventData::Resolved {ability,..}) if ability.get() == raw));
        events.extend(applied);
        if finished {
            return events;
        }
    }
    panic!("controlled action did not resolve");
}
