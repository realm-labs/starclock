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
