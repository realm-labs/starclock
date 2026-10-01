use super::{PRODUCTION_BUNDLE, load};
use crate::catalog::{CatalogLoadErrorKind, effect_bindings};
use starclock_combat::{DotFamily, EffectCategory, EffectDefinitionId};

#[test]
fn production_dot_family_tags_survive_real_sora_and_domain_lowering() {
    let catalog = load(PRODUCTION_BUNDLE).unwrap();
    let expected = [
        (1_010_501, DotFamily::Burn),
        (1_125_004, DotFamily::Burn),
        (1_135_004, DotFamily::Burn),
        (1_095_005, DotFamily::Bleed),
        (1_105_008, DotFamily::Bleed),
        (1_135_010, DotFamily::Bleed),
        (1_070_501, DotFamily::Shock),
        (1_145_001, DotFamily::Shock),
        (1_125_005, DotFamily::WindShear),
        (1_135_014, DotFamily::WindShear),
        (1_155_003, DotFamily::WindShear),
    ];
    for effect in &catalog.combat.effects {
        let family = expected
            .iter()
            .find(|(id, _)| *id == effect.id().get())
            .map(|(_, family)| *family);
        assert_eq!(effect.dot_family(), family, "effect {}", effect.id().get());
        assert_eq!(
            catalog
                .combat_catalog()
                .effect(effect.id())
                .unwrap()
                .dot_family(),
            family
        );
    }
    for (id, family) in expected {
        assert_eq!(
            catalog
                .combat_catalog()
                .effect(EffectDefinitionId::new(id).unwrap())
                .unwrap()
                .dot_family(),
            Some(family)
        );
    }
}

#[test]
fn dot_family_lowering_rejects_conflicting_duplicate_and_non_damaging_metadata() {
    for (category, damaging, tags) in [
        (EffectCategory::Dot, true, vec!["burn", "shock"]),
        (EffectCategory::Dot, true, vec!["burn", "burn"]),
        (EffectCategory::Dot, false, vec!["burn"]),
        (EffectCategory::Buff, false, vec!["burn"]),
        (EffectCategory::Control, true, vec!["burn"]),
    ] {
        let error = effect_bindings::dot_family(1, category, damaging, &tags).unwrap_err();
        assert_eq!(error.kind(), CatalogLoadErrorKind::Domain);
    }
    assert_eq!(
        effect_bindings::dot_family(1, EffectCategory::Dot, true, &["necrosis", "fire"]),
        Ok(None)
    );
    assert_eq!(
        effect_bindings::dot_family(1, EffectCategory::Dot, true, &["burn", "enkindle"]),
        Ok(Some(DotFamily::Burn))
    );
}
