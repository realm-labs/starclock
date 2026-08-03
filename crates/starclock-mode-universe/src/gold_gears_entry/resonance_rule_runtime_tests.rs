use std::collections::BTreeMap;

use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams,
};
use starclock_combat::rule::model::SourceClass;

use crate::digest::Encoder;

use super::{
    CONUNDRUM_AREA_KEY, GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
    progression_runtime::GoldAndGearsExtrapolationContext,
    resonance_rule_runtime::{
        GoldAndGearsResonanceCombatAttachment, GoldAndGearsResonanceRuleAccuracy,
        GoldAndGearsResonanceRuleBinding, GoldAndGearsResonanceRuleKind,
        GoldAndGearsResonanceRuleOwnership,
    },
    tests::entry,
};
use super::{tests};

#[test]
fn resonance_partition_binds_exactly_90_terminal_rules() {
    let factory = factory();
    let bindings = factory.resonance_rule_bindings();
    assert_eq!(bindings.len(), 90);
    assert!(
        bindings
            .windows(2)
            .all(|pair| pair[0].rule_id() < pair[1].rule_id())
    );
    assert_eq!(
        count_kind(bindings, GoldAndGearsResonanceRuleKind::SharedResonance),
        36
    );
    assert_eq!(
        count_kind(bindings, GoldAndGearsResonanceRuleKind::Interplay),
        18
    );
    assert_eq!(
        count_kind(bindings, GoldAndGearsResonanceRuleKind::Extrapolation),
        36
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| { binding.ownership() == GoldAndGearsResonanceRuleOwnership::Shared })
            .count(),
        36
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.accuracy() == GoldAndGearsResonanceRuleAccuracy::ExactPublic
            })
            .count(),
        54
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.accuracy() == GoldAndGearsResonanceRuleAccuracy::ProjectPolicy
            })
            .count(),
        36
    );
    assert!(bindings.iter().all(|binding| {
        binding.executor()
            == if binding.ownership() == GoldAndGearsResonanceRuleOwnership::Shared {
                "ReleasedSharedExecutor"
            } else {
                "CombatRuleIr"
            }
    }));
}

#[test]
fn all_54_shared_resonance_and_interplay_rules_project_to_combat() {
    let factory = factory();
    let counts = factory
        .unique
        .paths
        .iter()
        .map(|path| (path.identity.stable_key.to_string(), 14))
        .collect::<Vec<_>>();
    let mut projected = Vec::new();
    for path in &factory.unique.paths {
        let path_key = path.identity.stable_key.as_ref();
        let instance = compile(factory, path_key, 0);
        let formations = factory
            .unique
            .resonances
            .iter()
            .filter(|resonance| {
                resonance.path == path.identity.id
                    && resonance.resonance_kind.as_ref() == "Formation"
            })
            .map(|resonance| resonance.identity.stable_key.to_string())
            .collect::<Vec<_>>();
        let additions = instance.resonance_additions(&counts, &formations).unwrap();
        assert_eq!(additions.formations().len(), 3);
        assert_eq!(additions.interplays().len(), 2);
        let combat = instance.compile_resonance_combat_set(&additions).unwrap();
        assert_eq!(combat.bindings().len(), 6);
        for binding in combat.bindings() {
            assert_eq!(
                binding.attachment(),
                GoldAndGearsResonanceCombatAttachment::PlayerOwner
            );
            assert_eq!(binding.source().class(), SourceClass::Mode);
            assert_ne!(
                binding.terminal().kind(),
                GoldAndGearsResonanceRuleKind::Extrapolation
            );
            projected.push((
                binding.terminal().rule_id().to_owned(),
                binding.source().digest(),
            ));
        }
    }
    projected.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(projected.len(), 54);
    assert!(projected.windows(2).all(|pair| pair[0].0 < pair[1].0));
    assert_eq!(
        hex(combat_fixture_digest(
            b"starclock-gold-gears-resonance-shared-fixture-v1",
            &projected
        )),
        "e559973235fd92eb68a0aebb69fbb6655a6cc8c3022404424afb28111f717e2f"
    );
}

#[test]
fn all_36_extrapolation_rules_project_with_seeded_enemy_attachment() {
    let factory = factory();
    let instance = compile(factory, "universe.path.preservation", 1);
    let mut projected = BTreeMap::new();
    let mut selection_calls = 0_u32;
    for offered_path in factory.extrapolation_paths() {
        let mut path_rules = BTreeMap::new();
        for seed in 0..64 {
            let mut rng = activity_rng(&instance, seed);
            let before = rng.snapshots();
            let selection = instance
                .compile_resonance_extrapolation(
                    GoldAndGearsExtrapolationContext::new(3, true, offered_path),
                    &mut rng,
                )
                .unwrap();
            assert_only_encounter_draws(&before, &rng.snapshots(), 2);
            let combat = instance
                .compile_extrapolation_combat_set(&selection)
                .unwrap();
            assert_eq!(combat.bindings().len(), 3);
            for binding in combat.bindings() {
                assert_eq!(
                    binding.attachment(),
                    GoldAndGearsResonanceCombatAttachment::RelativeToEnemyOwner
                );
                assert_eq!(binding.source().class(), SourceClass::Mode);
                assert_eq!(
                    binding.terminal().kind(),
                    GoldAndGearsResonanceRuleKind::Extrapolation
                );
                assert_eq!(
                    binding.terminal().accuracy(),
                    GoldAndGearsResonanceRuleAccuracy::ProjectPolicy
                );
                assert_eq!(binding.terminal().executor(), "CombatRuleIr");
                path_rules
                    .entry(binding.terminal().rule_id().to_owned())
                    .or_insert(binding.source().digest());
            }
            selection_calls += 1;
            if path_rules.len() == 4 {
                break;
            }
        }
        assert_eq!(
            path_rules.len(),
            4,
            "incomplete coverage for {offered_path}"
        );
        projected.extend(path_rules);
    }
    assert_eq!(projected.len(), 36);
    assert!(selection_calls <= 9 * 64);
    let entries = projected.into_iter().collect::<Vec<_>>();
    assert_eq!(selection_calls, 18);
    assert_eq!(
        hex(combat_fixture_digest(
            b"starclock-gold-gears-extrapolation-fixture-v1",
            &entries
        )),
        "afa8c9779868558a8326fa0a371067fb7f392dacd9cfd8d3acb61d486056a93c"
    );
}

#[test]
fn extrapolation_rejections_preserve_rng_and_valid_projection_polarity() {
    let factory = factory();
    let instance = compile(factory, "universe.path.preservation", 1);
    for invalid in [
        GoldAndGearsExtrapolationContext::new(2, true, "universe.path.abundance"),
        GoldAndGearsExtrapolationContext::new(3, false, "universe.path.abundance"),
        GoldAndGearsExtrapolationContext::new(3, true, "universe.path.unknown"),
    ] {
        let mut rng = activity_rng(&instance, 14_509);
        let before = rng.snapshots();
        assert!(
            instance
                .compile_resonance_extrapolation(invalid, &mut rng)
                .is_err()
        );
        assert_eq!(rng.snapshots(), before);
    }

    let mut rng = activity_rng(&instance, 14_509);
    let selection = instance
        .compile_resonance_extrapolation(
            GoldAndGearsExtrapolationContext::new(3, true, "universe.path.abundance"),
            &mut rng,
        )
        .unwrap();
    let combat = instance
        .compile_extrapolation_combat_set(&selection)
        .unwrap();
    assert!(combat.bindings().iter().all(|binding| {
        binding.attachment() == GoldAndGearsResonanceCombatAttachment::RelativeToEnemyOwner
    }));
}

fn factory() -> &'static GoldAndGearsRuntimeFactory {
    tests::shared_factory()
}

fn compile(
    factory: &GoldAndGearsRuntimeFactory,
    path: &str,
    auxiliary: u8,
) -> GoldAndGearsRuntimeInstance {
    let dice = &factory.unique.dice[0];
    factory
        .compile_entry(
            entry(factory, CONUNDRUM_AREA_KEY, path, dice)
                .with_neural_network(
                    factory
                        .unique
                        .neural_nodes
                        .iter()
                        .map(|node| node.identity.stable_key.to_string())
                        .collect(),
                )
                .with_conundrum(0, auxiliary, vec![CONUNDRUM_AREA_KEY.to_owned()]),
        )
        .unwrap()
}

fn activity_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    );
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn assert_only_encounter_draws(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
    count: u64,
) {
    for (old, new) in before.iter().zip(after) {
        let expected = if old.label() == ActivityRngLabel::Encounter {
            count
        } else {
            0
        };
        assert_eq!(new.draw_count(), old.draw_count() + expected);
        assert_eq!(new.seed(), old.seed());
    }
}

fn count_kind(
    bindings: &[GoldAndGearsResonanceRuleBinding],
    kind: GoldAndGearsResonanceRuleKind,
) -> usize {
    bindings
        .iter()
        .filter(|binding| binding.kind() == kind)
        .count()
}

fn combat_fixture_digest(domain: &[u8], entries: &[(String, [u8; 32])]) -> [u8; 32] {
    let mut encoder = Encoder::new(domain);
    encoder.u32(entries.len() as u32);
    for (rule, digest) in entries {
        encoder.text(rule);
        encoder.digest(*digest);
    }
    encoder.finish()
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
