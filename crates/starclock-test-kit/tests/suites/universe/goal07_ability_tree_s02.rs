use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

use starclock_activity::{
    BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
};
use starclock_combat::{
    CombatantSpecDigest, Energy, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    StatValue, TeamSide, UnitDefinitionId, UnitLevel,
    catalog::action::AbilityKind,
    modifier::model::{FormulaPurpose, FormulaStage},
    rule::model::{ProgramStep, RuleOperationTemplate, RuleValue, ValueExpr},
};
use starclock_mode_universe::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
        AbilityTarget,
    },
    battle_contribution::{UniverseBattleContributionCompiler, UniverseBattleContributionSet},
    battle_materialization::{UniverseBattleMaterializer, UniverseBattleRoster},
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    path_runtime::PathRuntimeCatalog,
    progression::AbilityOperation,
    run_runtime::RunRuntimeCatalog,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");
const RECORDS: [&str; 16] = [
    "universe.ability-tree.24",
    "universe.ability-tree.25",
    "universe.ability-tree.26",
    "universe.ability-tree.27",
    "universe.ability-tree.28",
    "universe.ability-tree.29",
    "universe.ability-tree.3",
    "universe.ability-tree.30",
    "universe.ability-tree.31",
    "universe.ability-tree.32",
    "universe.ability-tree.33",
    "universe.ability-tree.34",
    "universe.ability-tree.35",
    "universe.ability-tree.36",
    "universe.ability-tree.37",
    "universe.ability-tree.38",
];

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
    }))
}

#[test]
fn goal07_p2_m01_s02_executes_every_assigned_rule_and_operation_fixture() {
    let catalog = catalog();
    let selected = catalog
        .ability_tree_nodes()
        .iter()
        .filter(|node| RECORDS.contains(&node.stable_key()))
        .map(|node| node.id())
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), RECORDS.len());
    let runtime = AbilityRuntimeCatalog::compile(&catalog).expect("Ability runtime");
    let contexts = [
        AbilityExecutionContext::new(
            AbilityProjectionScope::Run,
            AbilityBoundary::AfterBattle,
            14,
            false,
        ),
        AbilityExecutionContext::new(
            AbilityProjectionScope::Battle,
            AbilityBoundary::BattleStart,
            14,
            false,
        ),
        AbilityExecutionContext::new(
            AbilityProjectionScope::Battle,
            AbilityBoundary::EnterEliteOrBossDomain,
            14,
            false,
        ),
    ];
    let projections =
        contexts.map(|context| runtime.project(&selected, context).expect("projection"));
    let executed = projections
        .iter()
        .flat_map(|projection| projection.applied_effects())
        .map(|effect| effect.source())
        .collect::<BTreeSet<_>>();
    assert_eq!(executed, selected.iter().copied().collect());
    let operations = projections
        .iter()
        .flat_map(|projection| projection.applied_effects())
        .map(|effect| effect.operation())
        .collect::<BTreeSet<_>>();
    assert!(operations.contains(&AbilityOperation::AddResource));
    assert!(operations.contains(&AbilityOperation::SetRatio));

    let battle = &projections[1];
    assert_raw(
        battle,
        AbilityTarget::PathResonanceInitialEnergy,
        20_000_000,
    );
    assert_raw(battle, AbilityTarget::PathResonanceDamageRatio, 350_000);
    assert_raw(
        battle,
        AbilityTarget::PartyDamageTakenReductionRatio,
        50_000,
    );
    assert_raw(battle, AbilityTarget::PartyInitialEnergy, 1_000_000);
    assert_eq!(battle.value(AbilityTarget::PartyEnergy), None);

    let elite = &projections[2];
    assert_raw(elite, AbilityTarget::PartyInitialEnergy, 1_000_000);
    assert_raw(elite, AbilityTarget::PartyEnergy, 1_000_000);
}

#[test]
fn s02_values_materialize_into_energy_damage_mitigation_and_resonance_damage() {
    let catalog = catalog();
    let contributions = complete_contributions(&catalog);
    let mitigation = contributions
        .modifiers()
        .iter()
        .filter(|binding| binding.target() == AbilityTarget::PartyDamageTakenReductionRatio)
        .collect::<Vec<_>>();
    assert_eq!(mitigation.len(), 6);
    assert_eq!(
        mitigation
            .iter()
            .map(|binding| binding.definition().purpose)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            FormulaPurpose::OrdinaryDamage,
            FormulaPurpose::Dot,
            FormulaPurpose::AdditionalDamage,
            FormulaPurpose::ElationDamage,
            FormulaPurpose::Break,
            FormulaPurpose::SuperBreak,
        ])
    );
    assert!(mitigation.iter().all(|binding| {
        binding.definition().stage == FormulaStage::Mitigation
            && binding.value().raw_six_decimal() == 50_000
    }));

    let roster = roster(&catalog);
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster, &contributions)
        .expect("materialized production battle");
    let spec = materialized.overlay().bindings()[0]
        .preparation()
        .variants()[0]
        .battle_spec();
    let player = spec
        .participants()
        .iter()
        .find(|participant| participant.side() == TeamSide::Player)
        .expect("player participant");
    assert_eq!(
        player.combatant().current_energy(),
        player.combatant().maximum_energy()
    );

    let resonance = player
        .combatant()
        .abilities()
        .iter()
        .filter_map(|id| materialized.combat_catalog().ability(*id))
        .find(|ability| {
            ability
                .action()
                .is_some_and(|action| action.kind() == AbilityKind::Ultimate)
        })
        .expect("materialized Path Resonance");
    let program = materialized
        .combat_catalog()
        .program(resonance.program())
        .expect("materialized Resonance program");
    let ProgramStep::Operation(RuleOperationTemplate::Damage { amount, .. }) = &program.steps()[0]
    else {
        panic!("Path Resonance must lower to an executable damage program");
    };
    assert!(
        expression_has_scalar(amount, 9_900_000),
        "550% base Resonance damage receives the complete selected tree's exact 80% increase: {amount:?}"
    );
}

#[test]
fn third_formation_requires_the_authored_ability_tree_capability() {
    let catalog = catalog();
    let path = &catalog.paths()[0];
    let blessings = BlessingRuntimeCatalog::compile(&catalog)
        .unwrap()
        .contributions_from_owned(
            &path
                .blessings()
                .iter()
                .take(14)
                .map(|id| (*id, 1))
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let formations = path
        .formations()
        .iter()
        .map(|id| (*id, 1))
        .collect::<Vec<_>>();
    let runtime = PathRuntimeCatalog::compile(&catalog).unwrap();
    assert!(
        runtime
            .contributions(path.id(), &blessings, &formations)
            .is_err()
    );
    assert_eq!(
        runtime
            .contributions_with_formation_slots(path.id(), &blessings, &formations, 3)
            .unwrap()
            .formations()
            .len(),
        3
    );
}

fn complete_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.hunt")
        .expect("The Hunt path");
    let owned = path
        .blessings()
        .iter()
        .take(14)
        .map(|id| (*id, 1))
        .collect::<Vec<_>>();
    let blessings = BlessingRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_from_owned(&owned)
        .unwrap();
    let formations = path
        .formations()
        .iter()
        .map(|id| (*id, 1))
        .collect::<Vec<_>>();
    let path_contributions = PathRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_with_formation_slots(path.id(), &blessings, &formations, 3)
        .unwrap();
    let curios = CurioRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_from_owned(&[], &[], &[])
        .unwrap();
    let selected = catalog
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let abilities = RunRuntimeCatalog::compile(catalog)
        .unwrap()
        .ability_contributions(&selected)
        .unwrap();
    let projection = AbilityRuntimeCatalog::compile(catalog)
        .unwrap()
        .project(
            &selected,
            AbilityExecutionContext::new(
                AbilityProjectionScope::Battle,
                AbilityBoundary::EnterEliteOrBossDomain,
                14,
                false,
            ),
        )
        .unwrap();
    UniverseBattleContributionCompiler::compile(Arc::clone(catalog))
        .unwrap()
        .compile_snapshot(
            &path_contributions,
            &blessings,
            &curios,
            &abilities,
            &projection,
        )
        .unwrap()
}

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled() == expected,
        ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Divide { lhs, rhs, .. }
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        _ => false,
    }
}

fn roster(catalog: &UniverseCatalog) -> UniverseBattleRoster {
    let form = UnitDefinitionId::new(1).unwrap();
    let unit = catalog
        .simulation_catalog()
        .combat_catalog()
        .unit(form)
        .unwrap();
    let basic = unit
        .abilities()
        .iter()
        .copied()
        .find(|ability| {
            catalog
                .simulation_catalog()
                .combat_catalog()
                .ability(*ability)
                .and_then(|definition| definition.action())
                .is_some_and(|action| action.kind() == AbilityKind::Basic)
        })
        .unwrap();
    let combatant = ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(80).unwrap(),
        Hp::new(100_000).unwrap(),
        Speed::from_scaled(200_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![basic], Vec::new(), Vec::new()).unwrap(),
        CombatantSpecDigest::new([1; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(1_000_000_000).unwrap(),
        StatValue::from_scaled(1_000_000_000).unwrap(),
    )
    .with_energy(Energy::ZERO, Energy::from_scaled(100_000_000).unwrap())
    .unwrap();
    let participant = ParticipantId::new(1).unwrap();
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let lock = ParticipantLock::seal(
        policy,
        vec![
            ParticipantLockEntry::new(
                participant,
                0,
                0,
                form,
                OpaqueParticipantBuild::new(
                    combatant.digest(),
                    BuildDigest::new([2; 32]).unwrap(),
                    "goal07-ability-tree-s02",
                    ParticipantSourceKind::FixedResolved,
                )
                .unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    UniverseBattleRoster::new(&lock, vec![(participant, combatant)]).unwrap()
}

fn assert_raw(
    projection: &starclock_mode_universe::ability_runtime::AbilityRuntimeProjection,
    target: AbilityTarget,
    expected: i64,
) {
    assert_eq!(
        projection
            .value(target)
            .map(|value| value.raw_six_decimal()),
        Some(expected)
    );
}
