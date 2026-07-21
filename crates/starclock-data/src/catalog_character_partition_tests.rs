use super::{PRODUCTION_BUNDLE, load};

#[test]
fn production_c09_executes_follow_up_inspiration_and_elation_envelopes() {
    use starclock_combat::{
        AbilityId, UnitDefinitionId,
        catalog::action::{AbilityKind, AbilityTag, HitOperationDefinition},
        formula::model::DamageClass,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let dahlia = combat
        .ability(AbilityId::new(110_018).unwrap())
        .expect("Dahlia follow-up")
        .action()
        .expect("Dahlia follow-up action");
    assert_eq!(dahlia.kind(), AbilityKind::FollowUp);
    assert!(dahlia.tags().contains(AbilityTag::FollowUp));
    assert_eq!(dahlia.hits().len(), 5);

    let herta = combat
        .unit(UnitDefinitionId::new(74).unwrap())
        .expect("The Herta form");
    let inspiration = herta
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "inspiration")
        .expect("The Herta Inspiration");
    assert_eq!(inspiration.maximum().scaled(), 4_000_000);
    let hear_me_out = combat
        .ability(AbilityId::new(110_023).unwrap())
        .expect("The Herta enhanced Skill")
        .action()
        .expect("The Herta enhanced Skill action");
    assert_eq!(hear_me_out.kind(), AbilityKind::Skill);
    assert_eq!(hear_me_out.hits().len(), 6);
    assert_eq!(
        hear_me_out.resources().character_resource_costs()[0].stable_key(),
        "inspiration"
    );

    let tingyun = combat
        .ability(AbilityId::new(110_029).unwrap())
        .expect("Tingyun Technique")
        .action()
        .expect("Tingyun Technique action");
    assert_eq!(tingyun.resources().energy_gain().scaled(), 50_000_000);

    let topaz_skill = combat
        .ability(AbilityId::new(110_034).unwrap())
        .expect("Topaz Skill")
        .action()
        .expect("Topaz Skill action");
    assert_eq!(topaz_skill.kind(), AbilityKind::Skill);
    assert!(topaz_skill.tags().contains(AbilityTag::FollowUp));
    let numby = combat
        .ability(AbilityId::new(110_036).unwrap())
        .expect("Numby action")
        .action()
        .expect("Numby summon action");
    assert_eq!(numby.kind(), AbilityKind::Summon);
    assert!(numby.tags().contains(AbilityTag::Summon));
    assert!(numby.tags().contains(AbilityTag::FollowUp));

    for ability in [110_039, 110_040] {
        let finisher = combat
            .ability(AbilityId::new(ability).unwrap())
            .expect("Trailblazer Destruction finisher")
            .action()
            .expect("Trailblazer Destruction finisher action");
        assert_eq!(finisher.kind(), AbilityKind::ExtraAction);
        assert!(finisher.tags().contains(AbilityTag::Ultimate));
        assert_eq!(finisher.resources().energy_cost().scaled(), 0);
    }

    let elation = combat
        .ability(AbilityId::new(110_047).unwrap())
        .expect("Trailblazer Elation Skill")
        .action()
        .expect("Trailblazer Elation Skill action");
    assert_eq!(elation.kind(), AbilityKind::ExtraAction);
    assert!(elation.tags().contains(AbilityTag::ElationSkill));
    assert_eq!(elation.hits().len(), 9);
    for hit in elation.hits() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Trailblazer Elation Skill must execute scaling damage");
        };
        assert_eq!(damage.class(), DamageClass::Elation);
    }
}

#[test]
fn production_c10_executes_joint_hp_karma_and_elation_envelopes() {
    use starclock_combat::{
        AbilityId, UnitDefinitionId,
        catalog::action::{AbilityKind, AbilityTag, HitOperationDefinition},
        formula::model::DamageClass,
        modifier::model::StatKind,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let harmony = combat
        .ability(AbilityId::new(120_004).unwrap())
        .expect("Harmony Trailblazer Skill")
        .action()
        .expect("Harmony Trailblazer Skill action");
    assert_eq!(harmony.kind(), AbilityKind::Skill);
    assert_eq!(harmony.hits().len(), 5);

    let preservation = combat
        .ability(AbilityId::new(120_012).unwrap())
        .expect("Preservation Trailblazer Ultimate")
        .action()
        .expect("Preservation Trailblazer Ultimate action");
    let HitOperationDefinition::ScalingDamage(preservation_damage) =
        preservation.hits()[0].operations()[0]
    else {
        panic!("Preservation Trailblazer Ultimate must execute scaling damage");
    };
    assert_eq!(preservation_damage.scaling_stat(), StatKind::Def);

    let joint = combat
        .ability(AbilityId::new(120_019).unwrap())
        .expect("Remembrance Trailblazer Joint ATK")
        .action()
        .expect("Remembrance Trailblazer Joint ATK action");
    assert_eq!(joint.kind(), AbilityKind::Basic);
    assert!(joint.tags().contains(AbilityTag::Joint));
    assert_eq!(joint.hits().len(), 2);

    let tribbie = combat
        .ability(AbilityId::new(120_021).unwrap())
        .expect("Tribbie follow-up")
        .action()
        .expect("Tribbie follow-up action");
    assert_eq!(tribbie.kind(), AbilityKind::FollowUp);
    let HitOperationDefinition::ScalingDamage(tribbie_damage) = tribbie.hits()[0].operations()[0]
    else {
        panic!("Tribbie follow-up must execute scaling damage");
    };
    assert_eq!(tribbie_damage.scaling_stat(), StatKind::Hp);

    let welt = combat
        .ability(AbilityId::new(120_027).unwrap())
        .expect("Welt bounce Skill")
        .action()
        .expect("Welt bounce Skill action");
    assert_eq!(welt.hits().len(), 3);

    let xueyi = combat
        .unit(UnitDefinitionId::new(84).unwrap())
        .expect("Xueyi form");
    let karma = xueyi
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "karma")
        .expect("Xueyi Karma");
    assert_eq!(karma.maximum().scaled(), 8_000_000);
    let karmic = combat
        .ability(AbilityId::new(120_035).unwrap())
        .expect("Xueyi Karma follow-up")
        .action()
        .expect("Xueyi Karma follow-up action");
    assert_eq!(karmic.kind(), AbilityKind::FollowUp);
    assert_eq!(karmic.hits().len(), 3);
    assert_eq!(
        karmic.resources().character_resource_costs()[0].stable_key(),
        "karma"
    );
    let xueyi_ultimate = combat
        .ability(AbilityId::new(120_033).unwrap())
        .expect("Xueyi Ultimate")
        .action()
        .expect("Xueyi Ultimate action");
    let ignores_weakness = xueyi_ultimate.hits()[0]
        .operations()
        .iter()
        .find_map(|operation| match operation {
            HitOperationDefinition::ReduceToughness(reduction) => Some(reduction.ignores_weakness),
            _ => None,
        })
        .expect("Xueyi Ultimate Toughness operation");
    assert!(ignores_weakness);

    let yanqing = combat
        .ability(AbilityId::new(120_042).unwrap())
        .expect("Yanqing follow-up")
        .action()
        .expect("Yanqing follow-up action");
    assert_eq!(yanqing.kind(), AbilityKind::FollowUp);

    let boon = combat
        .ability(AbilityId::new(120_044).unwrap())
        .expect("Yao Guang Great Boon")
        .action()
        .expect("Yao Guang Great Boon action");
    assert_eq!(boon.kind(), AbilityKind::ExtraAction);
    assert!(boon.tags().contains(AbilityTag::AdditionalDamage));
    assert!(!boon.tags().contains(AbilityTag::Attack));
    let HitOperationDefinition::ScalingDamage(boon_damage) = boon.hits()[0].operations()[0] else {
        panic!("Great Boon must execute scaling damage");
    };
    assert_eq!(boon_damage.class(), DamageClass::Elation);

    let yao_guang = combat
        .ability(AbilityId::new(120_047).unwrap())
        .expect("Yao Guang Elation Skill")
        .action()
        .expect("Yao Guang Elation Skill action");
    assert!(yao_guang.tags().contains(AbilityTag::ElationSkill));
    assert_eq!(yao_guang.hits().len(), 6);
    for hit in yao_guang.hits() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Yao Guang Elation Skill must execute scaling damage");
        };
        assert_eq!(damage.class(), DamageClass::Elation);
    }
}

#[test]
fn production_c11_executes_bowstrings_counter_and_partial_energy_envelopes() {
    use starclock_combat::{
        AbilityId, UnitDefinitionId,
        catalog::action::{AbilityKind, AbilityTag},
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let yukong = combat
        .unit(UnitDefinitionId::new(87).unwrap())
        .expect("Yukong form");
    let bowstrings = yukong
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "roaring-bowstrings")
        .expect("Yukong Roaring Bowstrings");
    assert_eq!(bowstrings.maximum().scaled(), 2_000_000);
    let salvo = combat
        .ability(AbilityId::new(130_004).unwrap())
        .expect("Yukong Skill")
        .action()
        .expect("Yukong Skill action");
    assert!(!salvo.tags().contains(AbilityTag::Attack));
    let arrow = combat
        .ability(AbilityId::new(130_005).unwrap())
        .expect("Yukong additional damage")
        .action()
        .expect("Yukong additional-damage action");
    assert_eq!(arrow.kind(), AbilityKind::ExtraAction);
    assert!(arrow.tags().contains(AbilityTag::AdditionalDamage));
    assert_eq!(arrow.hits().len(), 1);

    let ultimate = combat
        .ability(AbilityId::new(130_009).unwrap())
        .expect("Yunli Ultimate")
        .action()
        .expect("Yunli Ultimate action");
    assert_eq!(ultimate.resources().energy_cost().scaled(), 120_000_000);
    assert!(!ultimate.tags().contains(AbilityTag::Attack));
    assert!(
        ultimate
            .hits()
            .iter()
            .all(|hit| hit.operations().is_empty())
    );
    let counter = combat
        .ability(AbilityId::new(130_010).unwrap())
        .expect("Yunli Counter")
        .action()
        .expect("Yunli Counter action");
    assert_eq!(counter.kind(), AbilityKind::Counter);
    assert!(counter.tags().contains(AbilityTag::Counter));
    assert_eq!(counter.hits().len(), 2);
}
