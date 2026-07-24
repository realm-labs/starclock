use std::sync::{Arc, OnceLock};

use starclock_combat::modifier::registry::ModifierRegistry;
use starclock_mode_universe::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
        AbilityTarget,
    },
    battle_contribution::{
        UNIVERSE_BATTLE_CONTRIBUTION_REVISION, UniverseBattleContributionCompiler,
        UniverseBattleRuleRole,
    },
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    path_runtime::PathRuntimeCatalog,
    progression::AbilityEffectClass,
    run_runtime::RunRuntimeCatalog,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
    }))
}

#[test]
fn complete_snapshot_compiles_to_canonical_rule_modifier_and_boundary_bindings() {
    let catalog = catalog();
    let path_definition = &catalog.paths()[0];
    let selected_path = path_definition.id();
    let mut owned_blessings = path_definition
        .blessings()
        .iter()
        .take(14)
        .map(|id| (*id, 1))
        .collect::<Vec<_>>();
    owned_blessings.sort_unstable_by_key(|entry| entry.0);
    let blessing_runtime = BlessingRuntimeCatalog::compile(&catalog).unwrap();
    let blessings = blessing_runtime
        .contributions_from_owned(&owned_blessings)
        .unwrap();
    let path_runtime = PathRuntimeCatalog::compile(&catalog).unwrap();
    let formations = path_definition
        .formations()
        .iter()
        .map(|id| (*id, 1))
        .collect::<Vec<_>>();
    let path = path_runtime
        .contributions(selected_path, &blessings, &formations)
        .unwrap();

    let curio_runtime = CurioRuntimeCatalog::compile(&catalog).unwrap();
    let selected_curios = curio_runtime
        .definitions()
        .iter()
        .take(2)
        .collect::<Vec<_>>();
    let inventory = selected_curios
        .iter()
        .map(|definition| (definition.curio(), 1))
        .collect::<Vec<_>>();
    let states = selected_curios
        .iter()
        .map(|definition| (definition.curio(), definition.initial_state()))
        .collect::<Vec<_>>();
    let charges = selected_curios
        .iter()
        .map(|definition| {
            let state = definition
                .states()
                .iter()
                .find(|state| state.id() == definition.initial_state())
                .unwrap();
            (definition.curio(), state.maximum_charges().unwrap_or(0))
        })
        .collect::<Vec<_>>();
    let curios = curio_runtime
        .contributions_from_owned(&inventory, &states, &charges)
        .unwrap();

    let selected_abilities = catalog
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let run_runtime = RunRuntimeCatalog::compile(&catalog).unwrap();
    let abilities = run_runtime
        .ability_contributions(&selected_abilities)
        .unwrap();
    let ability_runtime = AbilityRuntimeCatalog::compile(&catalog).unwrap();
    let projection = ability_runtime
        .project(
            &selected_abilities,
            AbilityExecutionContext::new(
                AbilityProjectionScope::Battle,
                AbilityBoundary::BattleStart,
                14,
                false,
            ),
        )
        .unwrap();

    let compiler = UniverseBattleContributionCompiler::compile(Arc::clone(&catalog)).unwrap();
    assert_eq!(
        compiler.digest(),
        [
            93, 87, 109, 65, 189, 231, 199, 152, 93, 17, 213, 118, 175, 160, 227, 113, 136, 148,
            162, 188, 103, 8, 58, 134, 46, 100, 223, 191, 20, 154, 91, 209,
        ]
    );
    let contributions = compiler
        .compile_snapshot(&path, &blessings, &curios, &abilities, &projection)
        .unwrap();
    assert_eq!(
        UNIVERSE_BATTLE_CONTRIBUTION_REVISION,
        "standard-universe-battle-contribution-v1"
    );
    assert_eq!(contributions.selected_path(), selected_path);
    assert_eq!(contributions.selected_path_blessings(), 14);
    let battle_abilities = abilities
        .entries()
        .iter()
        .filter(|entry| {
            matches!(
                entry.effect_class(),
                AbilityEffectClass::Battle | AbilityEffectClass::RunAndBattle
            )
        })
        .count();
    assert_eq!(
        contributions.rules().len(),
        owned_blessings.len() * 2 + 1 + formations.len() + inventory.len() * 2 + battle_abilities
    );
    assert!(
        contributions
            .rules()
            .windows(2)
            .all(|pair| pair[0].bundle() < pair[1].bundle())
    );
    assert!(
        contributions.rules().iter().all(|binding| binding
            .source()
            .digest()
            .iter()
            .any(|byte| *byte != 0))
    );
    assert_eq!(
        contributions
            .rules()
            .iter()
            .filter(|binding| binding.role() == UniverseBattleRuleRole::Resonance)
            .count(),
        1
    );
    assert_eq!(
        contributions
            .rules()
            .iter()
            .filter(|binding| binding.role() == UniverseBattleRuleRole::Formation)
            .count(),
        3
    );

    assert_eq!(contributions.modifiers().len(), 7);
    let registry = ModifierRegistry::new(
        contributions
            .modifiers()
            .iter()
            .map(|binding| binding.group().clone())
            .collect(),
        contributions
            .modifiers()
            .iter()
            .map(|binding| binding.definition().clone())
            .collect(),
    )
    .expect("projected modifier definitions validate through combat");
    assert_eq!(registry.len(), 7);
    for target in [
        AbilityTarget::PartyAttackFlat,
        AbilityTarget::PartyDefenseFlat,
        AbilityTarget::PartyMaximumHpFlat,
        AbilityTarget::PartyCritRateRatio,
        AbilityTarget::PartySpeedRatio,
        AbilityTarget::PartyCritDamageRatio,
        AbilityTarget::PartyEffectHitRateRatio,
    ] {
        assert!(
            contributions
                .modifiers()
                .iter()
                .any(|binding| binding.target() == target)
        );
    }
    assert!(
        contributions
            .boundary_values()
            .iter()
            .any(|value| value.target() == AbilityTarget::PathResonanceDamageRatio)
    );
    assert_eq!(
        contributions.digest(),
        [
            235, 122, 85, 225, 222, 199, 106, 121, 94, 192, 63, 124, 189, 133, 128, 208, 200, 105,
            145, 144, 127, 90, 89, 195, 0, 77, 88, 90, 133, 44, 226, 168,
        ]
    );
}
