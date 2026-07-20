use starclock_combat::catalog::action::{HitCritPolicy, HitTargetGroup};

const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");

#[test]
fn production_hit_plan_metadata_reaches_immutable_combat_actions() {
    let catalog =
        starclock_data::catalog::load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let bounce = catalog
        .combat_catalog()
        .ability(starclock_combat::AbilityId::new(20_011).unwrap())
        .and_then(starclock_combat::catalog::definition::AbilityDefinition::action)
        .expect("Asta Meteor Storm action");

    assert_eq!(bounce.hit_count(), 5);
    assert_eq!(bounce.hits()[0].target_group(), HitTargetGroup::Primary);
    assert!(
        bounce.hits()[1..]
            .iter()
            .all(|hit| hit.target_group() == HitTargetGroup::BounceDraw)
    );
    assert!(bounce.hits().iter().all(|hit| {
        hit.damage_share().scaled() == 200_000
            && hit.toughness_share().scaled() == 200_000
            && hit.crit_policy() == HitCritPolicy::PerTarget
    }));
}
