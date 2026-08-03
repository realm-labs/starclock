use std::sync::Arc;

use starclock_activity::{
    ActivityStateHash, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId,
    ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_combat::{
    Battle, BattleEventKind, BattleSeed, CombatantSpecDigest, Command, Energy, Hp,
    ParticipantSource, ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    StatValue, TeamSide, UnitDefinitionId, UnitLevel, catalog::action::AbilityKind,
};

use crate::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
    },
    battle_contribution::UniverseBattleContributionCompiler,
    battle_materialization::{
        UniverseBattleMaterializer, UniverseBattleRoster,
        catalog_composition::UniverseBattleCatalogComposition,
    },
    battle_snapshot::StandardUniverseBattleSnapshot,
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    id::{AbilityTreeNodeId, BlessingId, CurioId, ResonanceId},
    path_runtime::PathRuntimeCatalog,
    run_runtime::RunRuntimeCatalog,
};

use super::StandardUniverseBattleAssembler;
use super::StandardUniverseResolvedAssembly;

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../../config/universe-generated/config.sora");

#[derive(Clone)]
struct FixtureSelection {
    state_marker: u8,
    path_key: &'static str,
    blessing_levels: Vec<(BlessingId, u32)>,
    formations: Vec<(ResonanceId, u32)>,
    curio: Option<CurioId>,
    ability_tree: Vec<AbilityTreeNodeId>,
}

#[test]
fn current_inventory_transitions_reassemble_real_battle_inputs() {
    let catalog = catalog();
    let (lock, roster) = roster(&catalog);
    let composition = Arc::new(UniverseBattleCatalogComposition::compile(&catalog).unwrap());
    let empty = fixture_snapshot(
        &catalog,
        &lock,
        FixtureSelection {
            state_marker: 1,
            path_key: "universe.path.abundance",
            blessing_levels: vec![],
            formations: vec![],
            curio: None,
            ability_tree: vec![],
        },
    );
    let template = Arc::new(
        UniverseBattleMaterializer
            .compile_snapshot_from_composition(&catalog, &composition, &roster, &empty)
            .unwrap(),
    );
    let assembler =
        StandardUniverseBattleAssembler::new(Arc::clone(&catalog), composition, roster, template)
            .unwrap();

    let blessing = id_for_blessing(&catalog, "universe.blessing.612344");
    let curio = id_for_curio(&catalog, "universe.curio.8");
    let ability_tree = ability_tree_with_prerequisite(&catalog, "universe.ability-tree.2");
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.hunt")
        .unwrap();
    let resonance_blessings = path
        .blessings()
        .iter()
        .take(3)
        .copied()
        .map(|blessing| (blessing, 1))
        .collect::<Vec<_>>();

    let acquired = fixture_snapshot(
        &catalog,
        &lock,
        FixtureSelection {
            state_marker: 2,
            path_key: "universe.path.abundance",
            blessing_levels: vec![(blessing, 1)],
            formations: vec![],
            curio: None,
            ability_tree: vec![],
        },
    );
    let upgraded = fixture_snapshot(
        &catalog,
        &lock,
        FixtureSelection {
            state_marker: 3,
            path_key: "universe.path.abundance",
            blessing_levels: vec![(blessing, 2)],
            formations: vec![],
            curio: None,
            ability_tree: vec![],
        },
    );
    let curio_active = fixture_snapshot(
        &catalog,
        &lock,
        FixtureSelection {
            state_marker: 4,
            path_key: "universe.path.abundance",
            blessing_levels: vec![],
            formations: vec![],
            curio: Some(curio),
            ability_tree: vec![],
        },
    );
    let curio_suppressed = fixture_snapshot(
        &catalog,
        &lock,
        FixtureSelection {
            state_marker: 5,
            path_key: "universe.path.abundance",
            blessing_levels: vec![],
            formations: vec![],
            curio: None,
            ability_tree: vec![],
        },
    );
    let curio_removed = fixture_snapshot(
        &catalog,
        &lock,
        FixtureSelection {
            state_marker: 6,
            path_key: "universe.path.abundance",
            blessing_levels: vec![],
            formations: vec![],
            curio: None,
            ability_tree: vec![],
        },
    );
    let resonance = fixture_snapshot(
        &catalog,
        &lock,
        FixtureSelection {
            state_marker: 7,
            path_key: "universe.path.hunt",
            blessing_levels: resonance_blessings,
            formations: vec![],
            curio: None,
            ability_tree: vec![],
        },
    );
    let ability = fixture_snapshot(
        &catalog,
        &lock,
        FixtureSelection {
            state_marker: 8,
            path_key: "universe.path.abundance",
            blessing_levels: vec![],
            formations: vec![],
            curio: None,
            ability_tree,
        },
    );

    let empty = assembler.resolve_snapshot(&empty, None).unwrap();
    let acquired = assembler.resolve_snapshot(&acquired, None).unwrap();
    let upgraded = assembler.resolve_snapshot(&upgraded, None).unwrap();
    let active = assembler.resolve_snapshot(&curio_active, None).unwrap();
    let suppressed = assembler.resolve_snapshot(&curio_suppressed, None).unwrap();
    let removed = assembler.resolve_snapshot(&curio_removed, None).unwrap();
    let resonance = assembler.resolve_snapshot(&resonance, None).unwrap();
    let ability = assembler.resolve_snapshot(&ability, None).unwrap();

    let input = |resolved: &StandardUniverseResolvedAssembly| {
        resolved.materialization().difficulty_specs()[0]
            .battle_spec()
            .combat_input_digest()
    };
    assert_ne!(input(&empty), input(&acquired));
    assert_ne!(input(&acquired), input(&upgraded));
    assert_ne!(input(&empty), input(&active));
    assert_ne!(input(&active), input(&suppressed));
    assert_eq!(
        input(&suppressed),
        input(&removed),
        "two non-contributing Curio lifecycle states are combat-equivalent"
    );
    assert_ne!(
        suppressed.assembly_key(),
        removed.assembly_key(),
        "Activity provenance remains independently attributable"
    );
    assert_ne!(input(&empty), input(&resonance));
    assert_ne!(input(&empty), input(&ability));

    assert_blessing_level_changes_damage(&acquired, &upgraded);
    assert_curio_start_effect_is_suppressed(&active, &suppressed);
    assert!(
        resonance.materialization().difficulty_specs()[0]
            .battle_spec()
            .resources(TeamSide::Player)
            .keyed()
            .iter()
            .any(|resource| {
                resource.stable_key() == Some("standard-universe.path-resonance-energy")
            })
    );
    assert!(
        ability.materialization().difficulty_specs()[0]
            .battle_spec()
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Player)
            .all(|participant| !participant.combatant().modifier_bindings().is_empty())
    );
}

fn catalog() -> Arc<UniverseCatalog> {
    let core = starclock_data::catalog::load(CORE_BUNDLE).unwrap();
    UniverseCatalog::load(UNIVERSE_BUNDLE, core).unwrap()
}

fn roster(catalog: &UniverseCatalog) -> (ParticipantLock, UniverseBattleRoster) {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let mut locks = Vec::new();
    let mut combatants = Vec::new();
    for index in 0_u8..4 {
        let form = UnitDefinitionId::new(u32::from(index) + 1).unwrap();
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
            Speed::from_scaled(200_000_000 - i64::from(index) * 1_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![basic], vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([index + 1; 32]).unwrap(),
        )
        .unwrap()
        .with_base_attack_defense(
            StatValue::from_scaled(100_000_000).unwrap(),
            StatValue::from_scaled(100_000_000).unwrap(),
        )
        .with_energy(Energy::ZERO, Energy::from_scaled(100_000_000).unwrap())
        .unwrap();
        let participant = ParticipantId::new(u32::from(index) + 1).unwrap();
        locks.push(
            ParticipantLockEntry::new(
                participant,
                0,
                index,
                form,
                OpaqueParticipantBuild::new(
                    combatant.digest(),
                    BuildDigest::new([index + 17; 32]).unwrap(),
                    ParticipantSourceKind::FixedResolved,
                )
                .unwrap(),
            )
            .unwrap(),
        );
        combatants.push((participant, combatant));
    }
    let lock = ParticipantLock::seal(policy, locks).unwrap();
    let roster = UniverseBattleRoster::new(&lock, combatants).unwrap();
    (lock, roster)
}

fn fixture_snapshot(
    catalog: &Arc<UniverseCatalog>,
    lock: &ParticipantLock,
    selection: FixtureSelection,
) -> StandardUniverseBattleSnapshot {
    let path_definition = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == selection.path_key)
        .unwrap();
    let blessings = BlessingRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_from_owned(&selection.blessing_levels)
        .unwrap();
    let path = PathRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions(path_definition.id(), &blessings, &selection.formations)
        .unwrap();
    let curio_runtime = CurioRuntimeCatalog::compile(catalog).unwrap();
    let curio_definition = selection
        .curio
        .map(|curio| curio_runtime.definition(curio).unwrap());
    let curios = curio_runtime
        .contributions_from_owned(
            &curio_definition
                .iter()
                .map(|definition| (definition.curio(), 1))
                .collect::<Vec<_>>(),
            &curio_definition
                .iter()
                .map(|definition| (definition.curio(), definition.initial_state()))
                .collect::<Vec<_>>(),
            &[],
        )
        .unwrap();
    let ability_tree = RunRuntimeCatalog::compile(catalog)
        .unwrap()
        .ability_contributions(&selection.ability_tree)
        .unwrap();
    let context = AbilityExecutionContext::new(
        AbilityProjectionScope::Battle,
        AbilityBoundary::BattleStart,
        path.selected_path_blessings(),
        false,
    );
    let ability_projection = AbilityRuntimeCatalog::compile(catalog)
        .unwrap()
        .project(&selection.ability_tree, context)
        .unwrap();
    let contributions = UniverseBattleContributionCompiler::compile(Arc::clone(catalog))
        .unwrap()
        .compile_snapshot(
            &path,
            &blessings,
            &curios,
            &ability_tree,
            &ability_projection,
        )
        .unwrap();
    StandardUniverseBattleSnapshot::new(
        ActivityStateHash::new([selection.state_marker; 32]).unwrap(),
        lock,
        context,
        path,
        blessings,
        curios,
        ability_tree,
        ability_projection,
        contributions,
        &[],
    )
    .unwrap()
}

fn id_for_blessing(catalog: &UniverseCatalog, key: &str) -> BlessingId {
    catalog
        .blessings()
        .iter()
        .find(|definition| definition.stable_key() == key)
        .unwrap()
        .id()
}

fn id_for_curio(catalog: &UniverseCatalog, key: &str) -> CurioId {
    catalog
        .curios()
        .iter()
        .find(|definition| definition.stable_key() == key)
        .unwrap()
        .id()
}

fn ability_tree_with_prerequisite(catalog: &UniverseCatalog, key: &str) -> Vec<AbilityTreeNodeId> {
    let selected = catalog
        .ability_tree_nodes()
        .iter()
        .find(|definition| definition.stable_key() == key)
        .unwrap();
    let mut ids = selected.prerequisites().to_vec();
    ids.push(selected.id());
    ids.sort_unstable();
    ids
}

fn assert_blessing_level_changes_damage(
    acquired: &StandardUniverseResolvedAssembly,
    upgraded: &StandardUniverseResolvedAssembly,
) {
    let acquired_damage = first_action_damage(acquired, 0x31);
    let upgraded_damage = first_action_damage(upgraded, 0x31);
    assert_ne!(acquired_damage, upgraded_damage);
}

fn assert_curio_start_effect_is_suppressed(
    active: &StandardUniverseResolvedAssembly,
    suppressed: &StandardUniverseResolvedAssembly,
) {
    let active = start_events(active, 0x32);
    let suppressed = start_events(suppressed, 0x32);
    assert!(
        active
            .iter()
            .any(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
    );
    assert!(
        suppressed
            .iter()
            .all(|event| !matches!(event.kind(), BattleEventKind::Damage(_)))
    );
}

fn start_events(
    assembly: &StandardUniverseResolvedAssembly,
    marker: u8,
) -> Box<[starclock_combat::BattleEvent]> {
    let materialization = assembly.materialization();
    let mut battle = Battle::create(
        Arc::clone(materialization.combat_catalog()),
        materialization.difficulty_specs()[0].battle_spec().clone(),
        BattleSeed::new([marker; 32]),
    )
    .unwrap();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap()
        .events()
        .to_vec()
        .into_boxed_slice()
}

fn first_action_damage(assembly: &StandardUniverseResolvedAssembly, marker: u8) -> Vec<i64> {
    let materialization = assembly.materialization();
    let spec = durable_spec(materialization.difficulty_specs()[0].battle_spec(), marker);
    let mut battle = Battle::create(
        Arc::clone(materialization.combat_catalog()),
        spec,
        BattleSeed::new([marker; 32]),
    )
    .unwrap();
    battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    if battle
        .decision()
        .is_some_and(|decision| decision.kind() == starclock_combat::DecisionKind::InterruptWindow)
    {
        battle
            .apply(Command::PassInterruptWindow {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap();
    }
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .unwrap()
        .clone();
    battle
        .apply(command)
        .unwrap()
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(damage) => Some(damage.applied.get()),
            _ => None,
        })
        .collect()
}

fn durable_spec(
    original: &starclock_combat::BattleSpec,
    marker: u8,
) -> starclock_combat::BattleSpec {
    let mut participants = original.participants().to_vec();
    let enemy_index = participants
        .iter()
        .position(|participant| participant.side() == TeamSide::Enemy)
        .unwrap();
    let enemy = &participants[enemy_index];
    let source = match enemy.source() {
        ParticipantSource::EncounterEnemy(source) => source,
        _ => panic!("fixture enemy source"),
    };
    let base = enemy.combatant();
    let mut combatant = ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        Hp::new(2_000_000_000).unwrap(),
        base.speed(),
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
    participants[enemy_index] = ParticipantSpec::new(
        TeamSide::Enemy,
        enemy.formation(),
        ParticipantSource::EncounterEnemy(source),
        combatant,
    )
    .with_wave(enemy.wave())
    .unwrap();
    starclock_combat::BattleSpec::new(
        original.assembly_digest(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}
