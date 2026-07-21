use super::*;

const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const REPRESENTATIVE_BUNDLE: &[u8] =
    include_bytes!("../../../config/catalog-fixtures/representative/config.sora");

#[path = "catalog_character_partition_tests.rs"]
mod character_partition_tests;
#[path = "catalog_light_cone_tests.rs"]
mod light_cone_tests;

#[test]
fn production_bundle_builds_standard_v1_and_representative_characters() {
    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    assert_eq!(catalog.manifest().game_version, "4.4");
    assert_eq!(
        catalog.manifest().coverage_manifest_sha256,
        "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19"
    );
    assert_eq!(
        catalog.summary(),
        CatalogSummary {
            identity_count: 4889,
            enabled_identity_count: 4820,
            ability_count: 651,
            hit_plan_count: 354,
            character_count: 88,
            light_cone_count: 96,
            effect_count: 4,
            ai_graph_count: 17,
            enemy_count: 17,
            encounter_count: 6,
            standard_profile_count: 1,
            standard_scenario_count: 6,
        }
    );
    for (id, abilities, resources, parameters, traces) in [
        (2, 7, 1, 287, 20),
        (8, 6, 1, 101, 18),
        (18, 6, 1, 162, 18),
        (27, 8, 1, 318, 21),
        (45, 6, 1, 178, 20),
        (68, 12, 2, 361, 19),
    ] {
        let character = catalog
            .character(starclock_combat::UnitDefinitionId::new(id).unwrap())
            .expect("frozen representative character");
        assert_eq!(character.stat_row_count(), 86);
        assert_eq!(character.ability_count(), abilities);
        assert_eq!(character.resource_count(), resources);
        assert_eq!(character.ability_parameter_count(), parameters);
        assert_eq!(character.trace_count(), traces);
        assert_eq!(character.eidolon_count(), 6);
    }
    assert_eq!(
        catalog
            .enemy(starclock_combat::EnemyDefinitionId::new(98).unwrap())
            .expect("frozen Cocolia variant")
            .phases()
            .len(),
        2
    );
    assert_eq!(
        catalog
            .enemy(starclock_combat::EnemyDefinitionId::new(105).unwrap())
            .expect("frozen Great Septimus variant")
            .phases()
            .len(),
        3
    );
    assert_eq!(
        catalog
            .enemy(starclock_combat::EnemyDefinitionId::new(96).unwrap())
            .unwrap()
            .links()
            .len(),
        1
    );
    for (id, waves) in [(89, 1), (90, 1), (91, 1), (92, 3), (93, 1), (94, 3)] {
        assert_eq!(
            catalog
                .encounter(starclock_combat::EncounterId::new(id).unwrap())
                .expect("frozen encounter")
                .waves()
                .len(),
            waves
        );
    }
    for id in 278..=283 {
        assert!(
            catalog
                .standard_scenario(starclock_mode_standard::StandardScenarioId::new(id).unwrap())
                .is_some()
        );
    }
    let combat = catalog.combat_catalog();
    let linked = combat
        .linked_unit(starclock_combat::UnitDefinitionId::new(24_505).unwrap())
        .expect("Aglaea linked-unit definition");
    assert_eq!(linked.abilities()[0].get(), 24_610);
    let countdown = combat
        .countdown(24_304)
        .expect("Firefly countdown definition");
    assert_eq!(countdown.definition().ability().get(), 24_611);
    assert!(countdown.definition().ends_transformation());
    let asta = combat
        .effect(starclock_combat::EffectDefinitionId::new(24_006).unwrap())
        .expect("Asta SPD effect");
    assert_eq!(
        asta.modifiers(),
        [starclock_combat::ModifierDefinitionId::new(24_004).unwrap()]
    );
    let clara = combat
        .unit(starclock_combat::UnitDefinitionId::new(18).unwrap())
        .expect("Clara unit definition");
    assert_eq!(
        clara.resources()[0].stable_key(),
        "enhanced-counter-charges"
    );
    let base = starclock_combat::AbilityId::new(20_006).unwrap();
    let maximum = starclock_combat::AbilityId::new(1_000_640_202).unwrap();
    assert_eq!(
        combat.ability_parameter(base, "parameter.01"),
        Some(&starclock_combat::rule::model::RuleValue::Scalar(
            starclock_combat::Scalar::from_scaled(1_000_000)
        ))
    );
    assert_eq!(
        combat.ability_parameter(maximum, "parameter.01"),
        Some(&starclock_combat::rule::model::RuleValue::Scalar(
            starclock_combat::Scalar::from_scaled(2_800_000)
        ))
    );
    let formula = starclock_combat::rule::model::ValueExpr::Multiply {
        lhs: Box::new(starclock_combat::rule::model::ValueExpr::AbilityParameter {
            key: "parameter.01".into(),
            kind: starclock_combat::rule::model::RuleValueKind::Scalar,
        }),
        rhs: Box::new(starclock_combat::rule::model::ValueExpr::Literal(
            starclock_combat::rule::model::RuleValue::Scalar(
                starclock_combat::Scalar::from_scaled(2_000_000),
            ),
        )),
        rounding: starclock_combat::Rounding::NearestTiesEven,
    };
    let mut input = starclock_combat::rule::model::RuleEvaluationInput {
        event_kind: starclock_combat::rule::model::RuleEventKind::Action,
        event_facts: &starclock_combat::rule::model::RuleEventFacts {
            point: Some(starclock_combat::rule::model::RuleEventPoint::ActionResolved),
            ..starclock_combat::rule::model::RuleEventFacts::default()
        },
        cause: starclock_combat::rule::model::RuleCause {
            owner: None,
            actor: None,
            applier: None,
            target: None,
            source: None,
        },
        occurrence: starclock_combat::rule::model::RuleOccurrence {
            rule_instance: starclock_combat::RuleInstanceId::new(1).unwrap(),
            event: starclock_combat::EventId::new(1).unwrap(),
            hit: None,
            target: None,
            ability: Some(base),
            action: starclock_combat::ActionId::new(1),
            turn_event: None,
            wave: starclock_combat::WaveInstanceId::new(1).unwrap(),
        },
        source_tags: &[],
        slots: &[],
        selectors: &[],
        stat_reader: None,
        ability_parameter_reader: Some(combat),
        resource_reader: None,
        battle_query_reader: None,
    };
    assert_eq!(
        starclock_combat::rule::evaluate::evaluate_value(&formula, input, None).unwrap(),
        starclock_combat::rule::model::RuleValue::Scalar(starclock_combat::Scalar::from_scaled(
            2_000_000
        ))
    );
    input.occurrence.ability = Some(maximum);
    assert_eq!(
        starclock_combat::rule::evaluate::evaluate_value(&formula, input, None).unwrap(),
        starclock_combat::rule::model::RuleValue::Scalar(starclock_combat::Scalar::from_scaled(
            5_600_000
        ))
    );
    assert!(Arc::ptr_eq(&catalog, &Arc::clone(&catalog)));
}

#[test]
fn production_asta_basic_executes_minimum_and_maximum_hit_formulas() {
    use starclock_combat::{
        catalog::action::HitOperationDefinition,
        formula::model::{CombatElement, DamageClass},
        modifier::model::StatKind,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();
    for (raw, coefficient, damage) in [(20_013, 500_000, 1_000), (1_000_640_426, 1_400_000, 2_800)]
    {
        let ability = combat
            .ability(starclock_combat::AbilityId::new(raw).unwrap())
            .expect("Asta Basic effective-level variant");
        let hit = &ability.action().expect("compiled action").hits()[0];
        assert_eq!(hit.operations().len(), 2);
        let HitOperationDefinition::ScalingDamage(definition) = hit.operations()[0] else {
            panic!("first operation must be live-stat damage");
        };
        assert_eq!(definition.scaling_stat(), StatKind::Atk);
        assert_eq!(definition.coefficient().scaled(), coefficient);
        assert_eq!(definition.class(), DamageClass::Direct);
        assert_eq!(definition.element(), CombatElement::Fire);
        let formula = definition
            .resolve(starclock_combat::Scalar::checked_from_integer(2_000).unwrap())
            .unwrap();
        assert_eq!(
            formula
                .base_damage()
                .rounded_integer(starclock_combat::Rounding::Floor)
                .unwrap(),
            damage
        );

        let HitOperationDefinition::ReduceToughness(definition) = hit.operations()[1] else {
            panic!("second operation must reduce Toughness");
        };
        assert_eq!(definition.element, CombatElement::Fire);
        assert_eq!(definition.reduction.base.get(), 30);
        assert_eq!(
            starclock_combat::formula::toughness::reduction(definition.reduction)
                .unwrap()
                .attempted
                .get(),
            30
        );
    }
}

#[test]
fn production_bounce_and_blast_hits_use_exact_payload_overrides() {
    use starclock_combat::catalog::action::HitOperationDefinition;

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();
    for (raw, coefficient) in [(20_011, 250_000), (1_000_640_367, 625_000)] {
        let ability = combat
            .ability(starclock_combat::AbilityId::new(raw).unwrap())
            .expect("Asta Skill effective-level variant");
        let hits = ability.action().unwrap().hits();
        assert_eq!(hits.len(), 5);
        for hit in hits {
            let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
                panic!("bounce hit must own scaling damage");
            };
            assert_eq!(damage.coefficient().scaled(), coefficient);
            let HitOperationDefinition::ReduceToughness(toughness) = hit.operations()[1] else {
                panic!("bounce hit must own Toughness reduction");
            };
            assert_eq!(toughness.reduction.base.get(), 30);
        }
    }

    for (raw, expected) in [
        (20_029, [(800_000, 60), (300_000, 30)]),
        (1_000_640_943, [(2_000_000, 60), (750_000, 30)]),
    ] {
        let ability = combat
            .ability(starclock_combat::AbilityId::new(raw).unwrap())
            .expect("Kafka Skill effective-level variant");
        for (hit, (coefficient, toughness)) in ability.action().unwrap().hits().iter().zip(expected)
        {
            let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
                panic!("Blast hit must own scaling damage");
            };
            assert_eq!(damage.coefficient().scaled(), coefficient);
            let HitOperationDefinition::ReduceToughness(reduction) = hit.operations()[1] else {
                panic!("Blast hit must own Toughness reduction");
            };
            assert_eq!(reduction.reduction.base.get(), toughness);
        }
    }
}

#[test]
fn production_silver_wolf_executes_released_elation_envelopes() {
    use starclock_combat::{
        catalog::action::{
            AbilityProgramTiming, AbilityTag, HitOperationDefinition, HitTargetGroup,
        },
        formula::model::DamageClass,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();
    let enhanced = combat
        .ability(starclock_combat::AbilityId::new(20_036).unwrap())
        .expect("Silver Wolf enhanced Basic");
    let action = enhanced.action().expect("enhanced Basic action");
    assert_eq!(action.hit_count(), 101);
    assert_eq!(action.resources().skill_point_gain(), 0);
    assert_eq!(
        combat
            .ability(starclock_combat::AbilityId::new(20_042).unwrap())
            .unwrap()
            .action()
            .unwrap()
            .resources()
            .skill_point_gain(),
        1
    );
    assert_eq!(
        combat
            .ability(starclock_combat::AbilityId::new(20_045).unwrap())
            .unwrap()
            .action()
            .unwrap()
            .resources()
            .skill_point_cost(),
        1
    );
    assert_eq!(action.hits()[0].target_group(), HitTargetGroup::Primary);
    assert!(
        action.hits()[1..100]
            .iter()
            .all(|hit| hit.target_group() == HitTargetGroup::BounceDraw)
    );
    assert_eq!(action.hits()[100].target_group(), HitTargetGroup::All);

    let mut toughness = 0;
    for hit in &action.hits()[..100] {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("bounce must execute scaling damage");
        };
        assert_eq!(damage.coefficient().scaled(), 12_000);
        for operation in hit.operations() {
            if let HitOperationDefinition::ReduceToughness(reduction) = operation {
                toughness += reduction.reduction.base.get();
            }
        }
    }
    let final_hit = action.hits().last().unwrap();
    let HitOperationDefinition::ScalingDamage(damage) = final_hit.operations()[0] else {
        panic!("final hit must execute scaling damage");
    };
    assert_eq!(damage.coefficient().scaled(), 500_000);
    let HitOperationDefinition::ReduceToughness(reduction) = final_hit.operations()[1] else {
        panic!("final hit must execute Toughness reduction");
    };
    toughness += reduction.reduction.base.get();
    assert_eq!(toughness, 60);

    let elation_skill = combat
        .ability(starclock_combat::AbilityId::new(20_039).unwrap())
        .expect("Silver Wolf enhanced Elation Skill");
    assert!(
        elation_skill
            .action()
            .unwrap()
            .tags()
            .contains(AbilityTag::ElationSkill)
    );
    assert_eq!(elation_skill.action().unwrap().hit_count(), 6);
    for hit in elation_skill.action().unwrap().hits() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Elation bounce must execute scaling damage");
        };
        assert_eq!(damage.coefficient().scaled(), 450_000);
        assert_eq!(damage.class(), DamageClass::Elation);
    }

    let pro_gamer = combat
        .ability(starclock_combat::AbilityId::new(20_043).unwrap())
        .expect("Silver Wolf Elation Skill");
    assert_eq!(pro_gamer.programs().len(), 1);
    assert_eq!(
        pro_gamer.programs()[0].timing(),
        AbilityProgramTiming::Resolved
    );
    assert_eq!(pro_gamer.programs()[0].program().get(), 24_623);

    let silver = combat
        .unit(starclock_combat::UnitDefinitionId::new(68).unwrap())
        .expect("Silver Wolf form");
    let mmr = silver
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "hidden-mmr")
        .expect("Hidden MMR resource");
    assert_eq!(mmr.maximum().scaled(), 300_000_000);
}

#[test]
fn production_c01_executes_exact_bounce_and_hp_skill_envelopes() {
    use starclock_combat::{
        catalog::action::{HitOperationDefinition, HitTargetGroup},
        formula::model::CombatElement,
        modifier::model::StatKind,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let anaxa = combat
        .ability(starclock_combat::AbilityId::new(30_010).unwrap())
        .expect("Anaxa Skill");
    let hits = anaxa.action().expect("Anaxa damage action").hits();
    assert_eq!(hits.len(), 5);
    assert_eq!(hits[0].target_group(), HitTargetGroup::Primary);
    assert!(
        hits[1..]
            .iter()
            .all(|hit| hit.target_group() == HitTargetGroup::BounceDraw)
    );
    for (index, hit) in hits.iter().enumerate() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Anaxa hit must execute scaling damage");
        };
        assert_eq!(damage.scaling_stat(), StatKind::Atk);
        assert_eq!(damage.element(), CombatElement::Wind);
        assert_eq!(
            damage.coefficient().scaled(),
            if index == 0 { 350_000 } else { 200_000 }
        );
        let HitOperationDefinition::ReduceToughness(toughness) = hit.operations()[1] else {
            panic!("Anaxa hit must execute Toughness reduction");
        };
        assert_eq!(toughness.reduction.base.get(), 30);
    }

    let aventurine = combat
        .ability(starclock_combat::AbilityId::new(30_044).unwrap())
        .expect("Aventurine follow-up");
    let hits = aventurine
        .action()
        .expect("Aventurine damage action")
        .hits();
    assert_eq!(hits.len(), 7);
    for hit in hits {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Aventurine hit must execute scaling damage");
        };
        assert_eq!(damage.scaling_stat(), StatKind::Def);
        assert_eq!(damage.element(), CombatElement::Imaginary);
        assert_eq!(damage.coefficient().scaled(), 125_000);
    }

    let arlan_skill = combat
        .ability(starclock_combat::AbilityId::new(30_033).unwrap())
        .expect("Arlan HP-funded Skill");
    assert_eq!(
        arlan_skill.action().unwrap().resources().skill_point_cost(),
        0
    );
    assert_eq!(
        combat
            .ability(starclock_combat::AbilityId::new(30_031).unwrap())
            .unwrap()
            .action()
            .unwrap()
            .resources()
            .skill_point_gain(),
        1
    );
}

#[test]
fn production_c02_executes_hp_scaling_and_named_ultimate_cost_envelopes() {
    use starclock_combat::{
        Energy,
        catalog::action::{AbilityKind, HitOperationDefinition},
        modifier::model::StatKind,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let blade = combat
        .ability(starclock_combat::AbilityId::new(40_009).unwrap())
        .expect("Blade enhanced Basic");
    let action = blade.action().expect("Blade damage action");
    assert_eq!(action.kind(), AbilityKind::Basic);
    assert_eq!(action.resources().skill_point_gain(), 0);
    assert_eq!(action.hits().len(), 2);
    for hit in action.hits() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Blade hit must execute scaling damage");
        };
        assert_eq!(damage.scaling_stat(), StatKind::Hp);
    }

    let castorice = combat
        .ability(starclock_combat::AbilityId::new(40_030).unwrap())
        .expect("Castorice Ultimate");
    let resources = castorice.action().expect("Castorice action").resources();
    assert_eq!(resources.energy_cost(), Energy::ZERO);
    assert_eq!(resources.character_resource_costs().len(), 1);
    assert_eq!(
        resources.character_resource_costs()[0].stable_key(),
        "newbud"
    );
    assert_eq!(
        resources.character_resource_costs()[0].amount().scaled(),
        100_000_000
    );
    let form = combat
        .unit(starclock_combat::UnitDefinitionId::new(15).unwrap())
        .expect("Castorice form");
    let newbud = form
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "newbud")
        .expect("Castorice Newbud");
    assert_eq!(newbud.maximum().scaled(), 100_000_000);
}

#[test]
fn production_c03_executes_enhanced_elation_and_flying_aureus_envelopes() {
    use starclock_combat::{
        Energy, catalog::action::HitOperationDefinition, formula::model::DamageClass,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let fulgurant = combat
        .ability(starclock_combat::AbilityId::new(50_007).unwrap())
        .expect("Imbibitor Lunae enhanced Basic");
    let action = fulgurant.action().expect("Fulgurant Leap action");
    assert_eq!(action.resources().skill_point_cost(), 3);
    assert_eq!(action.resources().skill_point_gain(), 0);
    assert_eq!(action.hits().len(), 2);

    let evanescia = combat
        .ability(starclock_combat::AbilityId::new(50_032).unwrap())
        .expect("Evanescia Elation damage");
    let HitOperationDefinition::ScalingDamage(elation) =
        evanescia.action().unwrap().hits()[0].operations()[0]
    else {
        panic!("Evanescia hit must execute scaling damage");
    };
    assert_eq!(elation.class(), DamageClass::Elation);

    let feixiao = combat
        .ability(starclock_combat::AbilityId::new(50_046).unwrap())
        .expect("Feixiao Ultimate");
    let resources = feixiao.action().expect("Feixiao action").resources();
    assert_eq!(resources.energy_cost(), Energy::ZERO);
    assert_eq!(resources.character_resource_costs().len(), 1);
    assert_eq!(
        resources.character_resource_costs()[0].stable_key(),
        "flying-aureus"
    );
    assert_eq!(
        resources.character_resource_costs()[0].amount().scaled(),
        6_000_000
    );
}

#[test]
fn production_c04_executes_assist_and_source_energy_envelopes() {
    use starclock_combat::{
        AbilityId, EffectDefinitionId, ProgramId,
        catalog::action::{AbilityTag, HitOperationDefinition},
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let assist = combat
        .ability(AbilityId::new(60_047).unwrap())
        .expect("Himeko Nova Assist");
    let action = assist.action().expect("Assist damage action");
    assert!(action.tags().contains(AbilityTag::Assist));
    assert!(action.tags().supports_forced_skill());
    assert_eq!(action.resources().team_resource_costs().len(), 1);
    assert_eq!(
        action.resources().team_resource_costs()[0].stable_key(),
        "assist-use"
    );
    assert_eq!(action.resources().team_resource_costs()[0].amount(), 1);
    assert_eq!(action.hits().len(), 4);
    for hit in action.hits() {
        let toughness = hit
            .operations()
            .iter()
            .find_map(|operation| match operation {
                HitOperationDefinition::ReduceToughness(reduction) => Some(reduction),
                _ => None,
            })
            .expect("every Assist hit reduces Toughness");
        assert!(toughness.ignores_weakness);
    }

    let effect = combat
        .effect(EffectDefinitionId::new(67_001).unwrap())
        .expect("Starblazer Assist protocol effect");
    assert_eq!(
        effect.granted_abilities(),
        &[
            AbilityId::new(60_040).unwrap(),
            AbilityId::new(60_041).unwrap(),
            AbilityId::new(60_047).unwrap(),
        ]
    );
    let upraise = combat
        .ability(AbilityId::new(60_048).unwrap())
        .expect("Upraise the Vanward Cresset");
    assert_eq!(upraise.programs().len(), 1);
    assert_eq!(
        upraise.programs()[0].program(),
        ProgramId::new(67_002).unwrap()
    );
    let hyperluminal = combat
        .ability(AbilityId::new(60_043).unwrap())
        .expect("Hyperluminal Particle Beam");
    assert_eq!(hyperluminal.programs().len(), 1);
    assert_eq!(
        hyperluminal.programs()[0].program(),
        ProgramId::new(67_010).unwrap()
    );

    let form = combat
        .unit(starclock_combat::UnitDefinitionId::new(36).unwrap())
        .expect("Himeko Nova form");
    let source_energy = form
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "source-energy")
        .expect("Himeko Nova Source Energy");
    assert_eq!(source_energy.maximum().scaled(), 3_000_000);
    let orbital = combat
        .ability(AbilityId::new(60_045).unwrap())
        .expect("Orbital Annihilation Pulse");
    let costs = orbital
        .action()
        .expect("Orbital action")
        .resources()
        .character_resource_costs();
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0].stable_key(), "source-energy");
    assert_eq!(costs[0].amount().scaled(), 3_000_000);
}

#[test]
fn production_c05_executes_follow_up_summon_and_syzygy_envelopes() {
    use starclock_combat::{
        AbilityId, UnitDefinitionId,
        catalog::action::{AbilityKind, AbilityTag, HitOperationDefinition},
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let jade = combat
        .ability(AbilityId::new(70_027).unwrap())
        .expect("Jade follow-up");
    let jade_action = jade.action().expect("Jade follow-up action");
    assert_eq!(jade_action.kind(), AbilityKind::FollowUp);
    assert!(jade_action.tags().contains(AbilityTag::FollowUp));
    assert_eq!(jade_action.hits().len(), 1);
    let HitOperationDefinition::ScalingDamage(jade_damage) = jade_action.hits()[0].operations()[0]
    else {
        panic!("Jade follow-up must execute scaling damage");
    };
    assert_eq!(jade_damage.coefficient().scaled(), 600_000);
    let jade_form = combat
        .unit(UnitDefinitionId::new(41).unwrap())
        .expect("Jade form");
    let charge = jade_form
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "charge")
        .expect("Jade Charge");
    assert_eq!(charge.maximum().scaled(), 8_000_000);

    let lightning_lord = combat
        .ability(AbilityId::new(70_040).unwrap())
        .expect("Lightning-Lord summon action");
    let lightning_lord_action = lightning_lord.action().expect("Lightning-Lord action");
    assert_eq!(lightning_lord_action.kind(), AbilityKind::Summon);
    assert!(lightning_lord_action.tags().contains(AbilityTag::Summon));
    assert_eq!(lightning_lord_action.hits().len(), 3);
    let jing_yuan = combat
        .unit(UnitDefinitionId::new(43).unwrap())
        .expect("Jing Yuan form");
    let lightning_lord_hits = jing_yuan
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "lightning-lord-hits")
        .expect("Lightning-Lord hit counter");
    assert_eq!(lightning_lord_hits.initial().scaled(), 3_000_000);
    assert_eq!(lightning_lord_hits.maximum().scaled(), 10_000_000);

    let moon = combat
        .ability(AbilityId::new(70_047).unwrap())
        .expect("Jingliu enhanced Skill");
    let moon_action = moon.action().expect("Jingliu enhanced Skill action");
    assert_eq!(moon_action.kind(), AbilityKind::Skill);
    assert_eq!(moon_action.resources().skill_point_cost(), 0);
    assert_eq!(moon_action.resources().skill_point_gain(), 0);
    let costs = moon_action.resources().character_resource_costs();
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0].stable_key(), "syzygy");
    assert_eq!(costs[0].amount().scaled(), 1_000_000);
    let jingliu = combat
        .unit(UnitDefinitionId::new(44).unwrap())
        .expect("Jingliu form");
    let syzygy = jingliu
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "syzygy")
        .expect("Jingliu Syzygy");
    assert_eq!(syzygy.maximum().scaled(), 3_000_000);
}

#[test]
fn production_c06_executes_summon_counter_and_bounded_resource_envelopes() {
    use starclock_combat::{
        AbilityId, UnitDefinitionId,
        catalog::action::{AbilityKind, AbilityTag, HitOperationDefinition},
        modifier::model::StatKind,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let fuyuan = combat
        .ability(AbilityId::new(80_003).unwrap())
        .expect("Fuyuan summon action");
    let action = fuyuan.action().expect("Fuyuan action");
    assert_eq!(action.kind(), AbilityKind::Summon);
    assert!(action.tags().contains(AbilityTag::Summon));
    assert!(action.tags().contains(AbilityTag::FollowUp));
    assert_eq!(action.hits().len(), 2);
    let lingsha = combat
        .unit(UnitDefinitionId::new(46).unwrap())
        .expect("Lingsha form");
    let fuyuan_actions = lingsha
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "fuyuan-actions")
        .expect("Fuyuan action count");
    assert_eq!(fuyuan_actions.maximum().scaled(), 5_000_000);

    let luka = combat
        .ability(AbilityId::new(80_013).unwrap())
        .expect("Luka enhanced Basic");
    let action = luka.action().expect("Luka enhanced Basic action");
    assert_eq!(action.kind(), AbilityKind::Basic);
    assert_eq!(action.hits().len(), 4);
    assert_eq!(action.resources().skill_point_gain(), 0);
    let costs = action.resources().character_resource_costs();
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0].stable_key(), "fighting-will");
    assert_eq!(costs[0].amount().scaled(), 2_000_000);
    let luka_form = combat
        .unit(UnitDefinitionId::new(47).unwrap())
        .expect("Luka form");
    let fighting_will = luka_form
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "fighting-will")
        .expect("Luka Fighting Will");
    assert_eq!(fighting_will.initial().scaled(), 1_000_000);
    assert_eq!(fighting_will.maximum().scaled(), 4_000_000);

    let counter = combat
        .ability(AbilityId::new(80_029).unwrap())
        .expect("March 7th counter");
    let action = counter.action().expect("March 7th counter action");
    assert_eq!(action.kind(), AbilityKind::Counter);
    assert!(action.tags().contains(AbilityTag::Counter));
    assert_eq!(action.hits().len(), 1);
    let HitOperationDefinition::ScalingDamage(counter_damage) = action.hits()[0].operations()[0]
    else {
        panic!("March 7th counter must execute scaling damage");
    };
    assert_eq!(counter_damage.coefficient().scaled(), 500_000);

    let march = combat
        .ability(AbilityId::new(80_033).unwrap())
        .expect("March 7th enhanced Basic");
    let action = march.action().expect("March 7th enhanced Basic action");
    assert_eq!(action.kind(), AbilityKind::Basic);
    assert_eq!(action.hits().len(), 3);
    assert_eq!(action.resources().skill_point_gain(), 0);
    assert_eq!(
        action.resources().character_resource_costs()[0].stable_key(),
        "charge"
    );
    assert_eq!(
        action.resources().character_resource_costs()[0]
            .amount()
            .scaled(),
        7_000_000
    );

    let misha = combat
        .ability(AbilityId::new(80_041).unwrap())
        .expect("Misha Ultimate");
    assert_eq!(misha.action().unwrap().hits().len(), 3);
    let misha_form = combat
        .unit(UnitDefinitionId::new(52).unwrap())
        .expect("Misha form");
    let hits_per_action = misha_form
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "hits-per-action")
        .expect("Misha Hits Per Action");
    assert_eq!(hits_per_action.initial().scaled(), 3_000_000);
    assert_eq!(hits_per_action.maximum().scaled(), 10_000_000);

    let mortenax = combat
        .ability(AbilityId::new(80_046).unwrap())
        .expect("Mortenax Blade Skill");
    let action = mortenax.action().expect("Mortenax Blade Skill action");
    assert_eq!(action.kind(), AbilityKind::Skill);
    assert!(action.tags().contains(AbilityTag::FollowUp));
    assert_eq!(action.resources().skill_point_cost(), 0);
    assert_eq!(action.hits().len(), 5);
    for hit in action.hits() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Mortenax Blade Skill must execute scaling damage");
        };
        assert_eq!(damage.scaling_stat(), StatKind::Hp);
    }
}

#[test]
fn production_c07_executes_enhanced_actions_and_named_resource_envelopes() {
    use starclock_combat::{
        AbilityId, UnitDefinitionId,
        catalog::action::{AbilityKind, AbilityTag, HitOperationDefinition},
        modifier::model::StatKind,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let moze = combat
        .ability(AbilityId::new(90_003).unwrap())
        .expect("Moze follow-up");
    let action = moze.action().expect("Moze follow-up action");
    assert_eq!(action.kind(), AbilityKind::FollowUp);
    assert!(action.tags().contains(AbilityTag::FollowUp));
    assert!(action.tags().contains(AbilityTag::AdditionalDamage));
    assert_eq!(action.hits().len(), 1);

    let godslayer = combat
        .ability(AbilityId::new(90_011).unwrap())
        .expect("Mydei enhanced Skill");
    let action = godslayer.action().expect("Mydei enhanced Skill action");
    assert_eq!(action.kind(), AbilityKind::Skill);
    assert_eq!(action.hits().len(), 2);
    for hit in action.hits() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Mydei enhanced Skill must execute scaling damage");
        };
        assert_eq!(damage.scaling_stat(), StatKind::Hp);
    }
    let costs = action.resources().character_resource_costs();
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0].stable_key(), "charge");
    assert_eq!(costs[0].amount().scaled(), 150_000_000);
    let mydei = combat
        .unit(UnitDefinitionId::new(55).unwrap())
        .expect("Mydei form");
    assert_eq!(mydei.resources()[0].maximum().scaled(), 200_000_000);

    let foundation = combat
        .ability(AbilityId::new(90_032).unwrap())
        .expect("Phainon enhanced Skill");
    let action = foundation.action().expect("Phainon enhanced Skill action");
    assert_eq!(action.kind(), AbilityKind::Skill);
    assert_eq!(action.hits().len(), 17);
    let phainon = combat
        .unit(UnitDefinitionId::new(58).unwrap())
        .expect("Phainon form");
    let coreflame = phainon
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "coreflame")
        .expect("Phainon Coreflame");
    assert_eq!(coreflame.maximum().scaled(), 15_000_000);
    let ultimate = combat
        .ability(AbilityId::new(90_033).unwrap())
        .expect("Phainon Ultimate")
        .action()
        .expect("Phainon Ultimate action");
    assert_eq!(ultimate.resources().energy_cost().scaled(), 0);
    assert_eq!(
        ultimate.resources().character_resource_costs()[0]
            .amount()
            .scaled(),
        12_000_000
    );

    let qingque = combat
        .ability(AbilityId::new(90_041).unwrap())
        .expect("Qingque enhanced Basic")
        .action()
        .expect("Qingque enhanced Basic action");
    assert_eq!(qingque.kind(), AbilityKind::Basic);
    assert_eq!(qingque.hits().len(), 2);
    assert_eq!(qingque.resources().skill_point_gain(), 0);
    assert_eq!(
        qingque.resources().character_resource_costs()[0]
            .amount()
            .scaled(),
        4_000_000
    );

    let rappa = combat
        .ability(AbilityId::new(90_046).unwrap())
        .expect("Rappa enhanced Basic")
        .action()
        .expect("Rappa enhanced Basic action");
    assert_eq!(rappa.kind(), AbilityKind::Basic);
    assert!(rappa.tags().contains(AbilityTag::Attack));
    assert_eq!(rappa.hits().len(), 5);
    assert_eq!(rappa.resources().skill_point_gain(), 0);
    assert_eq!(
        rappa.resources().character_resource_costs()[0].stable_key(),
        "chroma-ink"
    );

    let robin = combat
        .ability(AbilityId::new(90_055).unwrap())
        .expect("Robin Ultimate")
        .action()
        .expect("Robin Ultimate action");
    assert!(robin.tags().contains(AbilityTag::AdditionalDamage));
    assert_eq!(robin.hits().len(), 1);
    assert!(robin.hits()[0].operations().is_empty());
}

#[test]
fn production_c08_executes_bounce_elation_and_support_resource_envelopes() {
    use starclock_combat::{
        AbilityId, UnitDefinitionId,
        catalog::action::{AbilityKind, AbilityTag, HitOperationDefinition},
        formula::model::DamageClass,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let excalibur = combat
        .ability(AbilityId::new(100_010).unwrap())
        .expect("Saber Ultimate")
        .action()
        .expect("Saber Ultimate action");
    assert_eq!(excalibur.kind(), AbilityKind::Ultimate);
    assert_eq!(excalibur.hits().len(), 11);
    let saber = combat
        .unit(UnitDefinitionId::new(63).unwrap())
        .expect("Saber form");
    let core = saber
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "core-resonance")
        .expect("Saber Core Resonance");
    assert_eq!(core.initial().scaled(), 1_000_000);
    assert_eq!(core.maximum().scaled(), 99_000_000);

    let sampo = combat
        .ability(AbilityId::new(100_016).unwrap())
        .expect("Sampo bounce Skill")
        .action()
        .expect("Sampo bounce Skill action");
    assert_eq!(sampo.kind(), AbilityKind::Skill);
    assert_eq!(sampo.hits().len(), 5);

    let serval = combat
        .ability(AbilityId::new(100_027).unwrap())
        .expect("Serval additional damage")
        .action()
        .expect("Serval additional-damage action");
    assert_eq!(serval.kind(), AbilityKind::ExtraAction);
    assert!(serval.tags().contains(AbilityTag::AdditionalDamage));
    assert_eq!(serval.hits().len(), 1);

    let silver_wolf = combat
        .ability(AbilityId::new(100_035).unwrap())
        .expect("Silver Wolf Technique")
        .action()
        .expect("Silver Wolf Technique action");
    assert_eq!(silver_wolf.hits().len(), 1);
    let reduction = silver_wolf.hits()[0]
        .operations()
        .iter()
        .find_map(|operation| match operation {
            HitOperationDefinition::ReduceToughness(reduction) => Some(reduction),
            _ => None,
        })
        .expect("Silver Wolf Technique Toughness operation");
    assert!(reduction.ignores_weakness);

    let sparkle_ultimate = combat
        .ability(AbilityId::new(100_042).unwrap())
        .expect("Sparkle Ultimate")
        .action()
        .expect("Sparkle Ultimate action");
    assert_eq!(sparkle_ultimate.resources().skill_point_gain(), 4);
    assert!(!sparkle_ultimate.tags().contains(AbilityTag::Attack));
    let sparkle_technique = combat
        .ability(AbilityId::new(100_043).unwrap())
        .expect("Sparkle Technique")
        .action()
        .expect("Sparkle Technique action");
    assert_eq!(sparkle_technique.resources().skill_point_gain(), 3);

    let bloom = combat
        .ability(AbilityId::new(100_045).unwrap())
        .expect("Sparxie enhanced Basic")
        .action()
        .expect("Sparxie enhanced Basic action");
    assert_eq!(bloom.kind(), AbilityKind::Basic);
    assert_eq!(bloom.hits().len(), 2);
    assert_eq!(bloom.resources().skill_point_gain(), 1);

    let encore = combat
        .ability(AbilityId::new(100_051).unwrap())
        .expect("Sparxie Elation Skill")
        .action()
        .expect("Sparxie Elation Skill action");
    assert_eq!(encore.kind(), AbilityKind::ExtraAction);
    assert!(encore.tags().contains(AbilityTag::ElationSkill));
    assert_eq!(encore.hits().len(), 21);
    for hit in encore.hits() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Sparxie Elation Skill must execute scaling damage");
        };
        assert_eq!(damage.class(), DamageClass::Elation);
    }
}

#[test]
fn production_characters_compile_at_e0_and_complete_e6() {
    use starclock_build::{
        ability::AbilityInvestment,
        compiler::LoadoutCompiler,
        patch::BuildPatch,
        spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
    };
    use starclock_combat::UnitLevel;

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    for raw in 1..=88 {
        let form = starclock_combat::UnitDefinitionId::new(raw).unwrap();
        let character = catalog
            .build_catalog()
            .character(form)
            .expect("production character build definition");
        let investments = character
            .ability_levels()
            .iter()
            .map(|table| AbilityInvestment::new(table.family(), table.invested_cap()))
            .collect::<Vec<_>>();
        let base = CombatantBuildSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            PromotionStage::new(6).unwrap(),
        )
        .with_ability_levels(investments)
        .unwrap();
        let e0 = LoadoutCompiler
            .compile(catalog.build_catalog(), catalog.combat_catalog(), &base)
            .expect("production E0 build");
        assert!(e0.combatant().modifiers().is_empty());

        let graph = character.trace_graph().expect("complete Trace graph");
        let trace_modifiers = graph
            .nodes()
            .iter()
            .flat_map(|node| node.patches())
            .filter(|patch| matches!(patch, BuildPatch::AddModifier(_)))
            .count();
        assert!(trace_modifiers > 0);
        let e6_spec = base
            .with_traces(graph.canonical_order().to_vec())
            .unwrap()
            .with_eidolon(EidolonLevel::new(6).unwrap());
        let e6 = LoadoutCompiler
            .compile(catalog.build_catalog(), catalog.combat_catalog(), &e6_spec)
            .expect("production E6 build");
        assert_eq!(e6.combatant().modifiers().len(), trace_modifiers);
    }
}

#[test]
fn real_fixture_bundle_builds_representative_private_definitions() {
    let catalog = load_with_mode(REPRESENTATIVE_BUNDLE, LoadMode::Fixture)
        .expect("representative catalog must load");
    assert_eq!(
        catalog.manifest().data_revision,
        "catalog-representative-v1"
    );
    assert_eq!(
        catalog.summary(),
        CatalogSummary {
            identity_count: 3,
            enabled_identity_count: 0,
            ability_count: 1,
            hit_plan_count: 1,
            character_count: 1,
            light_cone_count: 0,
            effect_count: 0,
            ai_graph_count: 0,
            enemy_count: 0,
            encounter_count: 0,
            standard_profile_count: 0,
            standard_scenario_count: 0,
        }
    );
    let ability = &catalog.combat.abilities[0];
    assert_eq!(ability.id.get(), 2);
    assert_eq!(ability.level_cap, 6);
    assert_eq!(ability.phases[0].sequence, 1);
    let hit = &catalog.combat.hit_plans[0].hits[0];
    assert_eq!(hit.damage_ratio.scaled(), 1_000_000);
    assert_eq!(hit.toughness_ratio.scaled(), 1_000_000);
    let character = &catalog.builds.characters[0];
    assert_eq!(character.id.get(), 1);
    assert_eq!(character.base_energy.scaled(), 120_000_000);
    assert_eq!(character.base_aggro.scaled(), 100_000_000);
    assert_eq!(character.stats.len(), 2);
    assert_eq!(character.abilities[0].ability.get(), 2);
}

#[test]
fn production_loader_rejects_fixture_labels() {
    let error = load(REPRESENTATIVE_BUNDLE).expect_err("fixture cannot enter production");
    assert_eq!(error.kind(), CatalogLoadErrorKind::Metadata);
    assert!(error.to_string().contains("synthetic") || error.to_string().contains("fixture"));
}

#[test]
fn canonical_decimal_parser_has_no_floating_point_path() {
    for (source, expected) in [
        ("0", 0),
        ("1", 1_000_000),
        ("0.000001", 1),
        ("-12.345678", -12_345_678),
        ("9223372036854.775807", i64::MAX),
        ("-9223372036854.775808", i64::MIN),
    ] {
        assert_eq!(parse_decimal(source), Ok(expected));
    }
    for invalid in ["", "+1", "01", "-0", "1.", "1.0", "1e2", "0.0000001"] {
        assert!(parse_decimal(invalid).is_err(), "accepted {invalid}");
    }
}
