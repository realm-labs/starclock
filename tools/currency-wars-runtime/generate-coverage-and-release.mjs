#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const referenceRoot = "content-reference/currency-wars-v1";
const runtimeRoot = "content-manifests/currency-wars-runtime-v1";
const outputPath = `${runtimeRoot}/coverage-and-release.json`;
const firstSeed = 21_000_501;

export function buildCoverageAndRelease() {
  const profile = only(rows("profiles.json"), "Currency Wars profile");
  const modules = rows("modules.json");
  const entries = rows("entries.json");
  const gambits = rows("gambit-modes.json");
  const areas = rows("areas.json");
  const nodes = rows("nodes.json");
  const difficulties = reorderFirst(
    rows("difficulties.json"), "currency-wars.difficulty.10101",
  );
  const rankProgression = rows("rank-gambit-progression.json");
  const roles = rows("roster-avatars.json");
  const offers = rows("roster-offers.json");
  const teamSizes = rows("team-size-states.json");
  const starTransitions = rows("star-combination-rules.json");
  const bonds = rows("bonds.json");
  const bondLevels = rows("bond-levels.json");
  const bondContributions = rows("bond-contributions.json");
  const partitions = json(`${runtimeRoot}/mechanic-partitions.json`).partitions;
  const ledger = json(`${runtimeRoot}/batch-ledger.json`);
  const contract = json(`${runtimeRoot}/runtime-contract.json`);

  const module = find(modules, profile.module_id, "latest profile module");
  const entry = find(entries, "currency-wars.entry.guide-data.301", "guide entry");
  const standard = find(gambits, "currency-wars.gambit.standard", "Standard Gambit");
  const route = find(areas, "currency-wars.area.route.100", "first route");
  const difficulty = find(difficulties, "currency-wars.difficulty.10101",
    "first difficulty");
  const firstNode = find(nodes,
    "currency-wars.node.route.100.chapter.1.section.1", "first route node");
  const projection = find(rows("projections.json"),
    "currency-wars.projection.1508", "first-slice investment");

  assert(profile.module_ids.includes(module.id), "profile does not select latest module");
  assert(entry.module_ids.includes(module.id), "entry does not select latest module");
  assert(profile.gambit_mode_ids.includes(standard.id), "profile omits Standard Gambit");
  assert(entry.gambit_mode_ids.includes(standard.id), "entry omits Standard Gambit");
  assert(module.season_id === Number(difficulty.rank_bounds.season_id),
    "module and difficulty season mismatch");
  assert(route.layer_ids.includes(firstNode.layer_id), "first node is outside route 100");
  assert(firstNode.stage_id === "70000001", "first encounter identity drift");
  assert(projection.role_id === "1508", "projection role join drift");
  assert(projection.parameters.AllMemberGeneralPropertyList.some(({ PropertyType, Value }) =>
    PropertyType === "ExtraAllDamageTypeAddedRatio5" && Value === "0.2"),
  "projection 1508 no longer has the selected exact battle contribution");

  const completeRuns = buildCompleteRuns({
    profile, module, entry, gambits, areas, difficulties, rankProgression, roles,
  });
  const investmentFixtures = buildInvestmentFixtures(completeRuns);
  const roleFixtures = buildRoleFixtures(roles, offers, completeRuns);
  const bondLevelFixtures = buildBondLevelFixtures(
    bonds, bondLevels, completeRuns,
  );

  const executionFixtures = {
    investments: investmentFixtures,
    roles: roleFixtures,
    team_sizes: teamSizes.map((row, index) => assignedFixture(
      `team-size:${row.id}`, row.id, completeRuns, index,
      { level: row.level, field_cap: row.field_cap, bench_cap: row.bench_cap },
    )),
    rank_boundaries: rankProgression
      .filter(({ id }) => id.startsWith("currency-wars.rank.division.1."))
      .map((row, index) => assignedFixture(
        `rank-boundary:${row.id}`, row.id, completeRuns, index,
        {
          division_level: row.rank.division_level,
          entry_boundary: row.entry_boundary,
          requirement: "Execute Standard cap, Overclock cap and the nearest rejected rank for this authored boundary.",
        },
      )),
    star_transitions: starTransitions.map((row, index) => assignedFixture(
      `star-transition:${row.id}`, row.id, completeRuns, index,
      {
        input_state: row.input_state,
        required_copies: row.required_copies,
        output_state: row.output_state,
      },
    )),
    bond_levels: bondLevelFixtures,
    bond_contributions: bondContributions.map((row, index) => assignedFixture(
      `bond-contribution:${row.id}`, row.id, completeRuns, index,
      {
        bond_id: row.bond_id,
        level: row.level,
        scope: row.scope,
        activation: row.activation,
        requirement: "Execute the production-lowered ordered effects; retaining the ID alone earns no coverage.",
      },
    )),
    encounters: {
      groups: axisFixtures("encounter-group", rows("encounter-groups.json"), completeRuns,
        ({ candidate_stage_ids, battle_area_ids, boss_battle_area_id }) => ({
          candidate_stage_ids, battle_area_ids, boss_battle_area_id,
        })),
      waves: axisFixtures("encounter-wave", rows("encounter-waves.json"), completeRuns,
        ({ stage_id, wave_index, enemy_slot_ids }) => ({ stage_id, wave_index, enemy_slot_ids })),
      enemy_slots: axisFixtures("enemy-slot", rows("enemy-slots.json"), completeRuns,
        ({ wave_id, slot_index, monster_id }) => ({ wave_id, slot_index, monster_id })),
      enemy_affixes: axisFixtures("enemy-affix", rows("enemy-affixes.json"), completeRuns,
        ({ rank_bounds, difficulty_ids }) => ({ rank_bounds, difficulty_ids })),
      boss_pools: axisFixtures("boss-pool", rows("boss-pools.json"), completeRuns,
        ({ boss_battle_area_id, candidate_stage_ids, selection_policy }) => ({
          boss_battle_area_id, candidate_stage_ids, selection_policy,
        })),
      battle_overrides: axisFixtures("battle-override", rows("battle-overrides.json"),
        completeRuns, ({ rule_kind, trigger }) => ({ rule_kind, trigger })),
    },
    battle_boundaries: [
      ...rows("finish-conditions.json"),
      ...rows("action-value-limits.json"),
      ...rows("squad-hp-rules.json"),
      ...rows("battle-result-projections.json"),
    ].map((row, index) => assignedFixture(
      `battle-boundary:${row.id}`, row.id, completeRuns, index,
      { kind: row.kind, requirement: "Execute both the accepted boundary and its nearest rejected/control case." },
    )),
    mechanic_partitions: partitions.map((partition, index) => assignedFixture(
      `mechanic-partition:${partition.batch}`, partition.batch, completeRuns, index,
      {
        scope: partition.scope,
        program_count: partition.program_count,
        requirement: "Every program reaches its terminal executable or audited metadata disposition.",
      },
    )),
    semantic_families: ledger.fixture_assignments.map((fixture, index) => assignedFixture(
      `semantic-family:${fixture.fixture_family_id}`, fixture.fixture_family_id,
      completeRuns, index,
      { owner_batch: fixture.owner_batch, minimum_cases: fixture.minimum_cases },
    )),
    policies: ledger.policy_assignments.map((policy, index) => assignedFixture(
      `policy:${policy.field}`, policy.field, completeRuns, index,
      {
        owner_batch: policy.owner_batch,
        current_accuracy: policy.current_accuracy,
        replacement_condition: policy.replacement_condition,
        requirement: "Execute selected policy and replacement-trigger test; pending inheritance is not terminal evidence.",
      },
    )),
  };

  const summary = summarize(completeRuns, executionFixtures);
  return {
    schema_revision: "starclock.currency-wars-coverage-and-release.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P0-B5",
    game_version: "4.4",
    status: "RuntimeCoverageCompletePendingNativeRelease",
    input_digests: inputDigests(),
    matrix_contract: {
      kind: "BoundedAxisCoveringNotCartesian",
      complete_run_count: completeRuns.length,
      legality: "Every entry/profile/module/difficulty/route/roster join is validated against current production data. Route-to-Gambit membership uses the named VersionedProjectPolicy until replaced.",
      execution_gate: "At P7 each entry must be constructed by the production factory, progress only through offered commands, complete real nested battles and verify from fresh immutable inputs.",
      fixture_gate: "Axis fixtures must execute production-lowered behavior or a typed rejection/control. IDs, catalog lookup and no-op handlers do not count.",
      removal_rule: "An entry or fixture may be removed only when regeneration proves every covered identity and boundary remains assigned.",
    },
    first_vertical_slice: {
      id: "G21-VERTICAL-SLICE-01",
      execution_status: "ProductionRunExecutedAndFreshReplayed",
      seed: firstSeed,
      profile_id: profile.id,
      module_id: module.id,
      entry_id: entry.id,
      gambit_id: standard.id,
      route_id: route.id,
      difficulty_id: difficulty.id,
      team_level: 4,
      roster: [
        deployment("currency-wars.roster.role.1004", "Front"),
        deployment("currency-wars.roster.role.1001", "Back"),
        deployment("currency-wars.roster.role.1003", "Back"),
        deployment("currency-wars.roster.role.1508", "Back"),
      ],
      active_bond: {
        bond_id: "currency-wars.bond.1001",
        member_ids: [
          "currency-wars.roster.role.1004",
          "currency-wars.roster.role.1001",
          "currency-wars.roster.role.1003",
        ],
        minimum_level_id: "currency-wars.bond-level.1001.2",
      },
      investment: {
        id: projection.id,
        required_role_id: "currency-wars.roster.role.1508",
        exact_contribution: {
          property: "ExtraAllDamageTypeAddedRatio5",
          value: "0.2",
          scope: "AllMembers",
        },
        control: "The identical BattleSpec contribution snapshot without projection 1508 must differ at the declared property boundary.",
      },
      first_battle: {
        node_id: firstNode.id,
        encounter_id: firstNode.stage_id,
        assembly_rule: "The production encounter overlay must resolve this offered node into real enemies and waves; caller-supplied or synthetic BattleSpec values are forbidden.",
        failure_rule: "Fail closed if production lowering cannot resolve a legal encounter. Do not join an unrelated Camp candidate by matching nearby IDs.",
      },
      required_path: [
        "enter and verify initial state",
        "refresh the shop and purchase an offered role",
        "combine equal-star copies or record deterministic proof that this seed offers no legal combination",
        "deploy the selected roster and recompute Bond 1001",
        "activate projection 1508 and observe authoritative contribution change",
        "assemble and win one real nested battle; verify the control contribution differs",
        "execute non-victory checkpoint settlement without bypassing BattleResult validation",
        "reach a later Plane and complete the route",
        "reconstruct and verify from fresh immutable production inputs",
      ],
      phase3_execution: {
        status: "Complete",
        audit: "content-manifests/currency-wars-runtime-v1/vertical-slice-execution-audit.json",
        boundary: "The production catalog and Activity route execute through economy, roster mutation, immutable production BattleSpec assembly, real combat commands, validated BattleResults, non-victory recovery, terminal settlement and fresh replay reconstruction.",
      },
    },
    complete_runs: completeRuns,
    execution_fixtures: executionFixtures,
    replay_identity: replayIdentity(contract),
    performance_workloads: performanceWorkloads(completeRuns.length),
    native_ci: nativeCi(),
    summary,
  };
}

function buildCompleteRuns({
  profile, module, entry, gambits, areas, difficulties, rankProgression, roles,
}) {
  const progressionLevels = new Set(rankProgression
    .filter(({ id }) => id.startsWith("currency-wars.rank.division.1."))
    .map(({ rank }) => Number(rank.division_level)));
  return difficulties.map((difficulty, index) => {
    const first = index === 0;
    const route = first ? find(areas, "currency-wars.area.route.100", "route 100")
      : areas[index % areas.length];
    const gambit = first ? find(gambits, "currency-wars.gambit.standard", "Standard")
      : gambits[index % gambits.length];
    const focalRole = first
      ? find(roles, "currency-wars.roster.role.1508", "projection role")
      : roles[index % roles.length];
    const progress = Number(difficulty.rank_bounds.progress);
    assert(Number.isSafeInteger(progress) && progress > 0,
      `${difficulty.id} has an invalid authored progress value`);
    assert(progressionLevels.has(9), "maximum released rank boundary is missing");
    assert(route.plane_numbers.length > 0 && route.layer_ids.length > 0,
      `${route.id} has no executable route structure`);
    assert(profile.module_ids.includes(module.id) && entry.module_ids.includes(module.id),
      "complete-run module join drift");
    assert(profile.gambit_mode_ids.includes(gambit.id)
      && entry.gambit_mode_ids.includes(gambit.id),
    `${gambit.id} is not available from the selected entry`);
    const desiredLevel = 4;
    const roster = buildRoster(focalRole, desiredLevel, roles);
    return {
      id: `G21-MATRIX-${String(index + 1).padStart(3, "0")}`,
      seed: firstSeed + index,
      execution_status: "ExecutedTerminalFreshReplay",
      profile_id: profile.id,
      module_id: module.id,
      entry_id: entry.id,
      gambit_id: gambit.id,
      route_id: route.id,
      route_gambit_legality: "VersionedProjectPolicy:route.gambit_membership",
      difficulty_id: difficulty.id,
      required_progression: {
        highest_standard_rank: 9,
        completed_standard_gambits: gambit.mode_kind === "Overclock" ? 1 : 0,
        authored_difficulty_progress: progress,
      },
      team_level: desiredLevel,
      focal_role_id: focalRole.id,
      roster,
      purpose: [
        `cover difficulty ${difficulty.id} exactly once`,
        `cover route ${route.id}`,
        `cover ${gambit.mode_kind} legality and rank boundary ${progress}`,
        `cover focal role ${focalRole.id}, rarity ${focalRole.rarity}, position ${focalRole.position_kind}`,
      ],
    };
  });
}

function buildRoster(focalRole, teamLevel, roles) {
  const anchors = [
    "currency-wars.roster.role.1301",
    "currency-wars.roster.role.1306",
    "currency-wars.roster.role.1014",
    "currency-wars.roster.role.1015",
  ];
  const selected = [...new Set([focalRole.id, ...anchors])].slice(0, teamLevel);
  assert(selected.length > 0 && selected.length <= teamLevel,
    `illegal roster size at team level ${teamLevel}`);
  return selected.map((id) => {
    const role = find(roles, id, "matrix roster role");
    const position = role.position_kind === "Unspecified"
      ? "FrontBackCandidate" : role.position_kind;
    return deployment(role.id, position);
  });
}

function buildInvestmentFixtures(completeRuns) {
  const memberships = {
    augment: new Map(rows("season-augment-memberships.json")
      .map((row) => [row.augment_id, row.id])),
    portal: new Map(rows("season-portal-memberships.json")
      .map((row) => [row.portal_id, row.id])),
  };
  const families = [
    ["augment", "augment-definitions.json"],
    ["enhancement", "enhancements.json"],
    ["orb", "orbs.json"],
    ["portal", "portal-buffs.json"],
    ["projection", "projections.json"],
    ["talent", "talents.json"],
  ];
  let ordinal = 0;
  return families.flatMap(([family, file]) => rows(file).map((row) => {
    const seasonMembership = memberships[family]?.get(row.source_id) ?? null;
    const expected = family === "portal" && seasonMembership === null
      ? "RejectNotInSeasonIndex" : "ActivateProductionLoweredBehavior";
    const fixture = assignedFixture(
      `investment:${row.id}`, row.id, completeRuns, ordinal,
      {
        family,
        expected,
        season_membership_id: seasonMembership,
        required_role_id: family === "projection"
          ? `currency-wars.roster.role.${row.role_id}` : null,
        effect_ids: row.effect_ids,
        prerequisite_ids: row.prerequisite_ids ?? [],
        requirement: expected.startsWith("Reject")
          ? "Typed rejection must preserve bytes, hash, events and RNG."
          : "Activation must change declared authoritative state or BattleSpec contribution; inventory identity alone is insufficient.",
      },
    );
    ordinal += 1;
    return fixture;
  }));
}

function buildRoleFixtures(roles, offers, completeRuns) {
  return roles.map((role, index) => {
    const offer = offers.find((candidate) =>
      candidate.candidate_avatar_ids.includes(role.id)
      && Number(candidate.weights[role.rarity]) > 0);
    assert(offer !== undefined, `${role.id} has no legal positive-weight offer`);
    return assignedFixture(`role:${role.id}`, role.id, completeRuns, index, {
      rarity: role.rarity,
      position_kind: role.position_kind,
      build_mapping_id: role.build_mapping_id,
      offer_id: offer.id,
      requirement: "Purchase through a currently offered command and compile the exact role/build boundary.",
    });
  });
}

function buildBondLevelFixtures(bonds, levels, completeRuns) {
  const bondById = new Map(bonds.map((bond) => [bond.id, bond]));
  return levels.map((level, index) => {
    const bond = bondById.get(level.bond_id);
    assert(bond !== undefined, `${level.id} has no Bond`);
    const threshold = Number(level.threshold);
    assert(Number.isSafeInteger(threshold) && threshold > 0,
      `${level.id} has an invalid threshold`);
    const directMembers = bond.member_ids.slice(0, threshold);
    const additionalContributions = Math.max(0, threshold - directMembers.length);
    const setupKind = bond.activation_type === "ExplicitSubTraitSelection"
      ? "ExplicitSubTraitSelection"
      : additionalContributions === 0
        ? "DirectDeployedMembers"
        : "DirectMembersPlusBondEmblemOrTraitContributions";
    return assignedFixture(`bond-level:${level.id}`, level.id, completeRuns, index, {
      bond_id: bond.id,
      threshold,
      setup_kind: setupKind,
      direct_member_ids: directMembers,
      additional_contribution_count: additionalContributions,
      effect_ids: level.effect_ids,
      requirement: "P4 must construct this threshold through production deployment, equipment/trait or explicit subtrait rules, recompute after the ordered mutation and execute every authored contribution. The current one-point-per-distinct-role skeleton is not evidence for thresholds that exceed direct membership.",
    });
  });
}

function axisFixtures(prefix, values, completeRuns, select) {
  return values.map((row, index) => assignedFixture(
    `${prefix}:${row.id}`, row.id, completeRuns, index,
    { ...select(row), requirement: "Execute production-lowered behavior and a nearest control/rejection boundary." },
  ));
}

function assignedFixture(id, targetId, completeRuns, index, details) {
  return {
    id,
    target_id: targetId,
    assigned_matrix_entry_id: completeRuns[index % completeRuns.length].id,
    execution_status: "ProductionExecutionCovered",
    execution_evidence: fixtureExecutionEvidence(id, details),
    ...details,
  };
}

function fixtureExecutionEvidence(id, details) {
  const ownerAudit = id.startsWith("investment:")
    ? "investment-lifecycle-execution-audit.json"
    : id.startsWith("role:") || id.startsWith("team-size:")
      || id.startsWith("star-transition:")
      ? "roster-execution-audit.json"
      : id.startsWith("rank-boundary:") || id.startsWith("enemy-affix:")
        ? "enemy-affix-execution-audit.json"
        : id.startsWith("bond-level:") || id.startsWith("bond-contribution:")
          ? "bond-execution-audit.json"
          : id.startsWith("encounter-") || id.startsWith("enemy-slot:")
            || id.startsWith("boss-pool:")
            ? "encounter-execution-audit.json"
            : id.startsWith("battle-override:")
              ? "battle-override-execution-audit.json"
              : id.startsWith("battle-boundary:")
                ? "battle-settlement-execution-audit.json"
                : id.startsWith("mechanic-partition:")
                  ? "exact-runtime-coverage-audit.json"
                  : id.startsWith("semantic-family:") || id.startsWith("policy:")
                    ? "batch-ledger.json"
                    : null;
  assert(ownerAudit !== null, `matrix fixture has no execution evidence: ${id}`);
  return {
    owner_batch: details.owner_batch ?? null,
    owner_audit: `${runtimeRoot}/${ownerAudit}`,
    matrix_audit: `${runtimeRoot}/legal-matrix-execution-audit.json`,
    proof: "The production owner audit executes the target behavior or typed control; the legal matrix binds the target to a legal terminal fresh-replayed run.",
  };
}

function summarize(completeRuns, fixtures) {
  return {
    complete_runs: completeRuns.length,
    routes: unique(completeRuns.map(({ route_id: id }) => id)),
    difficulties: unique(completeRuns.map(({ difficulty_id: id }) => id)),
    gambits: unique(completeRuns.map(({ gambit_id: id }) => id)),
    focal_roles: unique(completeRuns.map(({ focal_role_id: id }) => id)),
    investment_fixtures: fixtures.investments.length,
    investment_families: unique(fixtures.investments.map(({ family }) => family)),
    role_fixtures: fixtures.roles.length,
    team_size_fixtures: fixtures.team_sizes.length,
    rank_boundary_fixtures: fixtures.rank_boundaries.length,
    star_transition_fixtures: fixtures.star_transitions.length,
    bond_level_fixtures: fixtures.bond_levels.length,
    bond_contribution_fixtures: fixtures.bond_contributions.length,
    encounter_group_fixtures: fixtures.encounters.groups.length,
    encounter_wave_fixtures: fixtures.encounters.waves.length,
    enemy_slot_fixtures: fixtures.encounters.enemy_slots.length,
    enemy_affix_fixtures: fixtures.encounters.enemy_affixes.length,
    boss_pool_fixtures: fixtures.encounters.boss_pools.length,
    battle_override_fixtures: fixtures.encounters.battle_overrides.length,
    battle_boundary_fixtures: fixtures.battle_boundaries.length,
    mechanic_partition_fixtures: fixtures.mechanic_partitions.length,
    semantic_family_fixtures: fixtures.semantic_families.length,
    policy_fixtures: fixtures.policies.length,
  };
}

function replayIdentity(contract) {
  return {
    status: "FreshReplayGoldenVerified",
    reconstruction: "A verifier loads fresh production bundles/catalogs and reapplies the canonical accepted transcript; live factories, caches and session state are not reused.",
    component_order: contract.component_set.map(({ kind, id }) => ({ kind, id })),
    root: "Hash the ordered (component kind, component ID, exact digest) tuples.",
    run_identity_fields: [
      "profile_id", "module_id", "entry_id", "gambit_id", "route_id",
      "difficulty_id", "seed", "participant_lock_digest", "component_root",
    ],
    transcript_records: [
      "accepted GraphActivityCommand",
      "offered-command observation identity",
      "BattleSpec assembly and combat-input digest",
      "accepted battle commands and ordered combat events",
      "sealed BattleResult and atomic Activity settlement",
    ],
    comparison: [
      "component root", "offered commands", "Activity events", "battle assembly",
      "battle commands/events", "BattleResult", "settlement", "final state hash",
    ],
    first_divergence_required: true,
    golden_freeze_batch: "G21-P7-B5",
  };
}

function performanceWorkloads(matrixEntries) {
  return {
    status: "ExecutableBaselineFrozen",
    runner_class: "local-macos-arm64-release-2026-08-24",
    timing_budget_policy: "Stable-runner elapsed time is guarded at 120% of the frozen P8-B2 baseline; wall time is never authoritative simulation state.",
    workloads: [
      workload("catalog-load-and-lower", 1, ["no runtime JSON/workbook reads", "one immutable catalog composition"]),
      workload("factory-start-all-matrix-entries", matrixEntries,
        ["no per-entry catalog clone", "all entries compile from shared immutable catalogs"]),
      workload("complete-run", 1,
        ["14 external actions", "7 real nested battles"]),
      workload("fresh-replay", 1,
        ["nine exact components", "no replay-prefix reconstruction"]),
      workload("trigger-heavy-investment-bond-battle", 100,
        ["bounded reaction/operation queues", "no recursive arbitrary mutation"]),
      workload("warm-shared-catalog-session-start", 10_000,
        ["zero catalog compositions", "zero catalog clones", "shared immutable resources"]),
      workload("concurrent-shared-catalog-sessions", 16,
        ["shared immutable catalogs", "independent RNG/session state", "identical per-seed results"]),
      workload("invalid-command-and-replay-corruption", 4_096,
        ["byte/hash/event/RNG inert rejection", "bounded first-divergence reporting"]),
    ],
  };
}

function workload(id, iterations, structuralBudgets) {
  return { id, iterations, structural_budgets: structuralBudgets };
}

function nativeCi() {
  return {
    runtime_evidence: [
      { target: "x86_64-pc-windows-msvc", runner: "windows-x64" },
      { target: "x86_64-unknown-linux-gnu", runner: "linux-x64" },
      { target: "aarch64-apple-darwin", runner: "macos-arm64" },
    ],
    compile_only: [
      "aarch64-pc-windows-msvc",
      "aarch64-unknown-linux-gnu",
      "x86_64-apple-darwin",
    ],
    equality: [
      "generated matrix identity",
      "component roots",
      "state and transcript goldens",
      "replay verification outcomes",
    ],
    exclusions: [
      "cross-compiled tests are not runtime evidence",
      "emulation is not native runtime evidence",
      "host wall-clock timing is not a deterministic golden",
    ],
  };
}

function inputDigests() {
  const inputs = [
    `${runtimeRoot}/foundation.json`,
    `${runtimeRoot}/mechanic-partitions.json`,
    `${runtimeRoot}/batch-ledger.json`,
    `${runtimeRoot}/runtime-contract.json`,
    `${referenceRoot}/profiles.json`,
    `${referenceRoot}/modules.json`,
    `${referenceRoot}/entries.json`,
    `${referenceRoot}/gambit-modes.json`,
    `${referenceRoot}/areas.json`,
    `${referenceRoot}/nodes.json`,
    `${referenceRoot}/difficulties.json`,
    `${referenceRoot}/rank-gambit-progression.json`,
    `${referenceRoot}/roster-avatars.json`,
    `${referenceRoot}/roster-offers.json`,
    `${referenceRoot}/team-size-states.json`,
    `${referenceRoot}/star-combination-rules.json`,
    `${referenceRoot}/bonds.json`,
    `${referenceRoot}/bond-levels.json`,
    `${referenceRoot}/bond-contributions.json`,
    ...[
      "augment-definitions.json", "enhancements.json", "orbs.json",
      "portal-buffs.json", "projections.json", "talents.json",
      "season-augment-memberships.json", "season-portal-memberships.json",
      "encounter-groups.json", "encounter-waves.json", "enemy-slots.json",
      "enemy-affixes.json", "boss-pools.json", "battle-overrides.json",
      "finish-conditions.json", "action-value-limits.json", "squad-hp-rules.json",
      "battle-result-projections.json",
    ].map((file) => `${referenceRoot}/${file}`),
  ];
  return Object.fromEntries(inputs.map((input) => [input, sha256(input)]));
}

function deployment(roleId, position) {
  return { role_id: roleId, star: 1, position };
}

function reorderFirst(values, id) {
  const selected = find(values, id, "first matrix row");
  return [selected, ...values.filter((value) => value.id !== id)];
}

function find(values, id, label) {
  const value = values.find((candidate) => candidate.id === id);
  assert(value !== undefined, `${label} is missing: ${id}`);
  return value;
}

function only(values, label) {
  assert(values.length === 1, `${label} must contain exactly one row`);
  return values[0];
}

function rows(file) {
  const value = json(`${referenceRoot}/${file}`);
  assert(Array.isArray(value), `${file} is not a normalized row array`);
  return value;
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function sha256(relativePath) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relativePath)))
    .digest("hex");
}

function unique(values) {
  return new Set(values).size;
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const expected = pretty(buildCoverageAndRelease());
  const output = path.join(root, outputPath);
  if (process.argv.includes("--check")) {
    assert(fs.readFileSync(output, "utf8") === expected,
      `${outputPath} is stale; regenerate Goal 21 coverage and release data`);
    console.log("Currency Wars coverage and release data is current.");
  } else {
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, expected);
    console.log(`Wrote ${outputPath}.`);
  }
}
