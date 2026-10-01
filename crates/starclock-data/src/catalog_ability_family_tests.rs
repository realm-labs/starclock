//! Current production curves lower to explicit immutable family roots.
use crate::catalog::{load, tests::PRODUCTION_BUNDLE};

#[test]
fn production_effective_ability_variants_keep_explicit_family_roots() {
    let catalog = load(PRODUCTION_BUNDLE).unwrap();
    let combat = catalog.combat_catalog();
    let builds = catalog.build_catalog();
    let mut variants = 0;
    for id in builds.character_ids() {
        let character = builds.character(id).unwrap();
        for table in character.ability_levels() {
            for row in table.rows() {
                let ability = combat.ability(row.resolved_ability()).unwrap();
                assert_eq!(ability.family(), table.family());
                assert_eq!(
                    combat.ability(ability.family()).unwrap().family(),
                    table.family()
                );
                variants += usize::from(ability.id() != ability.family());
            }
        }
    }
    assert!(
        variants > 0,
        "real production must exercise effective variants"
    );
}
