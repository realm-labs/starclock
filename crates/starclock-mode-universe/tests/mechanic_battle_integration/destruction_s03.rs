use super::*;
use starclock_combat::{
    EffectDamageGuard, ParticipantInitialState,
    modifier::model::{FormulaStage, StatKind},
    rule::model::{ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue, ValueExpr},
};

const ATTACK: (&str, u32) = ("universe.blessing.612550", 2);
const PREVENTION: (&str, u32) = ("universe.blessing.612551", 2);
const MAXIMUM_HP: (&str, u32) = ("universe.blessing.612552", 2);
const HIT_ENERGY: (&str, u32) = ("universe.blessing.612553", 2);
const ENTRY_SHIELD: (&str, u32) = ("universe.blessing.612554", 2);
const LOW_HP_SHIELD: (&str, u32) = ("universe.blessing.612555", 2);

#[test]
fn goal07_p2_m07_s03_materializes_every_assigned_level_as_generic_rule_ir() {
    let catalog = catalog();
    let contributions = s03_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    for key in [
        "StageAbility_61255001",
        "StageAbility_61255101",
        "StageAbility_61255201",
        "StageAbility_61255301",
        "StageAbility_61255401",
        "StageAbility_61255501",
    ] {
        let rule = combat
            .rule(binding(&contributions, key).rule())
            .unwrap_or_else(|| panic!("{key} is executable"));
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }

    let guard = combat
        .effect(effect_id(binding(&contributions, "StageAbility_61255101")))
        .unwrap();
    assert_eq!(
        guard.runtime_template().unwrap().damage_guard(),
        EffectDamageGuard::TeamDefeatOnce
    );
}

#[test]
fn exact_static_stats_energy_and_shields_preserve_typed_boundaries() {
    let catalog = catalog();
    let contributions = s03_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let attack = single_modifier(combat, binding(&contributions, "StageAbility_61255001"));
    assert_eq!(attack.stat, StatKind::Atk);
    assert_eq!(attack.stage, FormulaStage::PercentOfBase);
    assert_eq!(literal_scalar(&attack.value), Some(420_000));

    let hp = single_modifier(combat, binding(&contributions, "StageAbility_61255201"));
    assert_eq!(hp.stat, StatKind::Hp);
    assert_eq!(hp.stage, FormulaStage::PercentOfBase);
    assert_eq!(literal_scalar(&hp.value), Some(240_000));

    let energy = combat
        .rule(binding(&contributions, "StageAbility_61255301").rule())
        .unwrap()
        .runtime()
        .unwrap();
    assert!(energy.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::DamageApplied
            && combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                            resource: starclock_combat::rule::model::RuleResourceKind::Energy,
                            amount: ValueExpr::Literal(RuleValue::Scalar(value)),
                            ..
                        }) if value.scaled() == 6_000_000
                    )
                })
    }));

    for key in ["StageAbility_61255401", "StageAbility_61255501"] {
        let runtime = combat
            .rule(binding(&contributions, key).rule())
            .unwrap()
            .runtime()
            .unwrap();
        assert!(runtime.triggers().iter().any(|trigger| {
            combat
                .program(trigger.program)
                .unwrap()
                .steps()
                .iter()
                .any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::Shield { .. })
                    )
                })
        }));
    }
}

#[test]
fn defeat_guard_signals_and_heals_the_actual_lethal_target() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.destruction",
        &[PREVENTION],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let spec = player_duel_spec(&materialization, 1, 0xb2);
    let (mut battle, started) = start(&materialization, spec, 0xb3);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    let guarded_target = started
        .events()
        .iter()
        .chain(resolution.events())
        .find_map(|event| match event.kind() {
            BattleEventKind::RuleSignal(signal)
                if signal.code == starclock_combat::TEAM_DEFEAT_GUARDED_SIGNAL =>
            {
                event.cause().primary_target()
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "team defeat guard emits its actual target: start={:#?}; next={:#?}",
                started.events(),
                resolution.events()
            )
        });
    let guarded = battle
        .view()
        .units_by_id()
        .find(|unit| unit.id() == guarded_target)
        .unwrap();
    assert_eq!(guarded.life(), starclock_combat::LifeState::Alive);
    assert!(
        guarded.current_hp().get() >= 30_001,
        "enhanced Reflection restores 30% MaxHP after the one-HP clamp"
    );
}

#[test]
fn hit_energy_and_low_hp_shield_execute_without_fault_in_production() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.destruction",
        &[HIT_ENERGY, LOW_HP_SHIELD],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let spec = player_duel_spec(&materialization, 49_000, 0xb5);
    let (mut battle, started) = start(&materialization, spec, 0xb6);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:?}",
        resolution.fault(),
        resolution.events()
    );
    let events = started
        .events()
        .iter()
        .chain(resolution.events())
        .collect::<Vec<_>>();
    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Shield(_))),
        "incoming damage below 50% HP creates the once-per-battle shield: start={:#?}; next={:#?}",
        started.events(),
        resolution.events()
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Resource(_))),
        "the same action restores enhanced hit Energy"
    );
}

fn s03_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.destruction",
        &[
            ATTACK,
            PREVENTION,
            MAXIMUM_HP,
            HIT_ENERGY,
            ENTRY_SHIELD,
            LOW_HP_SHIELD,
        ],
        None,
        false,
    )
}

fn binding<'a>(
    contributions: &'a UniverseBattleContributionSet,
    key: &str,
) -> &'a starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding {
    contributions
        .rules()
        .iter()
        .find(|binding| binding.source_binding_key() == Some(key))
        .unwrap_or_else(|| panic!("{key} selected"))
}

fn effect_id(
    binding: &starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding,
) -> starclock_combat::EffectDefinitionId {
    starclock_combat::EffectDefinitionId::new(0x7660_0000 + binding.rule().get()).unwrap()
}

fn single_modifier<'a>(
    combat: &'a starclock_combat::catalog::CombatCatalog,
    binding: &starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding,
) -> &'a starclock_combat::modifier::model::ModifierDefinition {
    let effect = combat.effect(effect_id(binding)).unwrap();
    assert_eq!(effect.modifiers().len(), 1);
    combat.modifier(effect.modifiers()[0]).unwrap()
}

fn literal_scalar(value: &ValueExpr) -> Option<i64> {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => Some(value.scaled()),
        _ => None,
    }
}

fn player_duel_spec(
    materialization: &UniverseBattleMaterialization,
    protected_hp: i64,
    marker: u8,
) -> BattleSpec {
    let original = durable_spec(materialization, marker, false);
    let players = original
        .participants()
        .iter()
        .filter(|participant| participant.side() == TeamSide::Player)
        .take(2)
        .collect::<Vec<_>>();
    let protected = players[0];
    let attacker = players[1];
    let enemy = original
        .participants()
        .iter()
        .find(|participant| participant.side() == TeamSide::Enemy)
        .expect("production encounter enemy");
    let enemy_source = match enemy.source() {
        ParticipantSource::EncounterEnemy(source) => source,
        _ => panic!("production encounter enemy source"),
    };
    let attacker_combatant = clone_combatant(
        attacker.combatant(),
        Speed::from_scaled(500_000_000).unwrap(),
        marker,
    );
    let protected_combatant = clone_enemy_with_rules(
        enemy.combatant(),
        protected.combatant(),
        Speed::from_scaled(100_000_000).unwrap(),
        marker.wrapping_add(1),
    );
    let attacker = ParticipantSpec::new(
        TeamSide::Player,
        starclock_combat::FormationIndex::new(0).unwrap(),
        ParticipantSource::Player,
        attacker_combatant,
    );
    let protected = ParticipantSpec::new(
        TeamSide::Enemy,
        enemy.formation(),
        ParticipantSource::EncounterEnemy(enemy_source),
        protected_combatant,
    )
    .with_wave(enemy.wave())
    .unwrap()
    .with_initial_state(
        ParticipantInitialState::new(
            Hp::new(protected_hp).unwrap(),
            protected.combatant().maximum_hp(),
            protected.combatant().current_energy(),
            protected.combatant().maximum_energy(),
            starclock_combat::LifeState::Alive,
            starclock_combat::PresenceState::Present,
        )
        .unwrap(),
    )
    .unwrap();
    BattleSpec::new(
        original.rules_revision(),
        AssemblyDigest::new([marker; 32]).unwrap(),
        original.encounter(),
        vec![attacker, protected],
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}

fn clone_combatant(
    base: &ResolvedCombatantSpec,
    speed: Speed,
    marker: u8,
) -> ResolvedCombatantSpec {
    let mut combatant = ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        base.maximum_hp(),
        speed,
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            base.rule_bundles().to_vec(),
            base.modifiers().to_vec(),
        )
        .unwrap(),
        CombatantSpecDigest::new([marker; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_energy(base.current_energy(), base.maximum_energy())
    .unwrap()
    .with_sources(base.sources().to_vec())
    .unwrap()
    .with_modifier_bindings(base.modifier_bindings().to_vec())
    .unwrap();
    if !base.toughness_layers().is_empty() {
        combatant = combatant
            .with_toughness(
                base.rank(),
                base.weaknesses().to_vec(),
                base.toughness_layers().to_vec(),
            )
            .unwrap();
    }
    combatant
}

fn clone_enemy_with_rules(
    enemy: &ResolvedCombatantSpec,
    rules: &ResolvedCombatantSpec,
    speed: Speed,
    marker: u8,
) -> ResolvedCombatantSpec {
    let mut rule_bundles = enemy.rule_bundles().to_vec();
    rule_bundles.extend_from_slice(rules.rule_bundles());
    rule_bundles.sort_unstable();
    rule_bundles.dedup();
    let mut modifiers = enemy.modifiers().to_vec();
    modifiers.extend_from_slice(rules.modifiers());
    modifiers.sort_unstable();
    modifiers.dedup();
    let mut sources = enemy.sources().to_vec();
    sources.extend_from_slice(rules.sources());
    sources.sort_unstable_by_key(|source| source.definition());
    sources.dedup_by_key(|source| source.definition());
    let mut modifier_bindings = enemy.modifier_bindings().to_vec();
    modifier_bindings.extend_from_slice(rules.modifier_bindings());
    modifier_bindings.sort_unstable();
    modifier_bindings.dedup();
    let mut combatant = ResolvedCombatantSpec::new(
        enemy.form(),
        enemy.level(),
        enemy.maximum_hp(),
        speed,
        ResolvedDefinitionBindings::new(enemy.abilities().to_vec(), rule_bundles, modifiers)
            .unwrap(),
        CombatantSpecDigest::new([marker; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(enemy.base_attack(), enemy.base_defense())
    .with_energy(enemy.current_energy(), enemy.maximum_energy())
    .unwrap()
    .with_sources(sources)
    .unwrap()
    .with_modifier_bindings(modifier_bindings)
    .unwrap();
    if !enemy.toughness_layers().is_empty() {
        combatant = combatant
            .with_toughness(
                enemy.rank(),
                enemy.weaknesses().to_vec(),
                enemy.toughness_layers().to_vec(),
            )
            .unwrap();
    }
    combatant
}
