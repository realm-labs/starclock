#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildCoverageAndRelease } from "./generate-coverage-and-release.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const referenceRoot = "content-reference/currency-wars-v1";
const runtimeRoot = "content-manifests/currency-wars-runtime-v1";

run("node", ["tools/currency-wars-runtime/generate-coverage-and-release.mjs", "--check"]);
const release = buildCoverageAndRelease();
const ledger = json(`${runtimeRoot}/batch-ledger.json`);
const contract = json(`${runtimeRoot}/runtime-contract.json`);
const foundation = json(`${runtimeRoot}/foundation.json`);
const dispositions = json(`${runtimeRoot}/runtime-dispositions.json`);

assert(release.status === "RuntimeReleaseComplete",
  "P8-B5 matrix release state drift");
assert(release.complete_runs.length === 97 && release.complete_runs.length < 128,
  "complete-run matrix is not the frozen bounded 97-entry set");
assert(release.complete_runs.every(({ execution_status: status }) =>
  status === "ExecutedTerminalFreshReplay"),
"P7-B6 complete runs are not terminal and fresh-replayed");
assertUnique(release.complete_runs.map(({ id }) => id), "matrix entry ID");
assertUnique(release.complete_runs.map(({ seed }) => seed), "matrix seed");

assertExactIds(release.complete_runs.map(({ route_id: id }) => id), rows("areas.json"),
  "matrix routes", false);
assertExactIds(release.complete_runs.map(({ difficulty_id: id }) => id),
  rows("difficulties.json"), "matrix difficulties", true);
assertExactIds(release.complete_runs.map(({ gambit_id: id }) => id), rows("gambit-modes.json"),
  "matrix Gambits", false);
assertExactIds(release.complete_runs.map(({ focal_role_id: id }) => id),
  rows("roster-avatars.json"), "matrix focal roles", false);

const profile = only(rows("profiles.json"), "profile");
const moduleIds = new Set(rows("modules.json").map(({ id }) => id));
const entry = find(rows("entries.json"), "currency-wars.entry.guide-data.301", "entry");
const roles = new Map(rows("roster-avatars.json").map((row) => [row.id, row]));
for (const row of release.complete_runs) {
  assert(row.profile_id === profile.id, `${row.id} profile drift`);
  assert(moduleIds.has(row.module_id) && profile.module_ids.includes(row.module_id)
    && entry.module_ids.includes(row.module_id), `${row.id} module join is invalid`);
  assert(profile.gambit_mode_ids.includes(row.gambit_id)
    && entry.gambit_mode_ids.includes(row.gambit_id), `${row.id} Gambit join is invalid`);
  assert(row.route_gambit_legality
    === "VersionedProjectPolicy:route.gambit_membership",
  `${row.id} hides the unresolved route/Gambit join`);
  assert(row.required_progression.highest_standard_rank === 9,
    `${row.id} does not use the released maximum-rank legal setup`);
  assert(row.roster.length > 0 && row.roster.length <= row.team_level,
    `${row.id} roster exceeds its team level`);
  assert(row.roster.some(({ role_id: id }) => id === row.focal_role_id),
    `${row.id} does not execute its focal role`);
  for (const deployed of row.roster) {
    const role = roles.get(deployed.role_id);
    assert(role !== undefined, `${row.id} uses an unknown role`);
    const expected = role.position_kind === "Unspecified"
      ? "FrontBackCandidate" : role.position_kind;
    assert(deployed.position === expected, `${row.id} position join drift`);
  }
}

const first = release.first_vertical_slice;
const firstRun = release.complete_runs[0];
for (const field of [
  "seed", "profile_id", "module_id", "entry_id", "gambit_id", "route_id",
  "difficulty_id", "team_level",
])
  assert(first[field] === firstRun[field], `first vertical-slice ${field} drift`);
assert(first.execution_status === "ProductionRunExecutedAndFreshReplayed"
  && first.phase3_execution.status === "Complete"
  && first.phase3_execution.boundary.includes("immutable production BattleSpec assembly"),
"first slice does not expose its completed production execution boundary");
assert(first.investment.id === "currency-wars.projection.1508"
  && first.investment.exact_contribution.property
    === "ExtraAllDamageTypeAddedRatio5"
  && first.investment.exact_contribution.value === "0.2",
"first slice no longer selects the exact Projection 1508 contribution");
assert(first.first_battle.encounter_id === "70000001"
  && first.first_battle.failure_rule.includes("Fail closed"),
"first battle must use the route encounter and fail closed on missing assembly");
assert(first.required_path.length === 9,
  "first slice does not cross every required end-to-end boundary");

const fixtures = release.execution_fixtures;
assertFixtureTargets(fixtures.investments, investmentRows(), "investment");
assert(new Set(fixtures.investments.map(({ family }) => family)).size === 6,
  "all six investment families are not assigned");
assert(fixtures.investments.length === foundation.denominators.investment_identities,
  "investment fixture denominator drift");
assert(fixtures.investments.every((fixture) =>
  fixture.requirement.includes("authoritative")
    || fixture.expected === "RejectNotInSeasonIndex"),
"an investment fixture gives credit for identity without behavior");
assert(fixtures.investments.filter(({ expected }) =>
  expected === "RejectNotInSeasonIndex").map(({ target_id: id }) => id).join(",")
  === "currency-wars.portal-buff.1015",
"season-excluded investment rejection boundary drift");

assertFixtureTargets(fixtures.roles, rows("roster-avatars.json"), "role");
assertFixtureTargets(fixtures.team_sizes, rows("team-size-states.json"), "team size");
assertFixtureTargets(fixtures.rank_boundaries,
  rows("rank-gambit-progression.json").filter(({ id }) =>
    id.startsWith("currency-wars.rank.division.1.")), "rank boundary");
assertFixtureTargets(fixtures.star_transitions, rows("star-combination-rules.json"),
  "star transition");
assertFixtureTargets(fixtures.bond_levels, rows("bond-levels.json"), "Bond level");
assertFixtureTargets(fixtures.bond_contributions, rows("bond-contributions.json"),
  "Bond contribution");
for (const fixture of fixtures.bond_levels)
  assert(fixture.direct_member_ids.length + fixture.additional_contribution_count
    === fixture.threshold, `${fixture.id} does not construct its exact threshold`);

const encounterAxes = [
  [fixtures.encounters.groups, "encounter-groups.json", "encounter group"],
  [fixtures.encounters.waves, "encounter-waves.json", "encounter wave"],
  [fixtures.encounters.enemy_slots, "enemy-slots.json", "enemy slot"],
  [fixtures.encounters.enemy_affixes, "enemy-affixes.json", "enemy affix"],
  [fixtures.encounters.boss_pools, "boss-pools.json", "boss pool"],
  [fixtures.encounters.battle_overrides, "battle-overrides.json", "battle override"],
];
for (const [axis, file, label] of encounterAxes)
  assertFixtureTargets(axis, rows(file), label);
assertFixtureTargets(fixtures.battle_boundaries, [
  ...rows("finish-conditions.json"),
  ...rows("action-value-limits.json"),
  ...rows("squad-hp-rules.json"),
  ...rows("battle-result-projections.json"),
], "battle boundary");
assertFixtureTargets(fixtures.mechanic_partitions,
  json(`${runtimeRoot}/mechanic-partitions.json`).partitions
    .map(({ batch: id }) => ({ id })), "mechanic partition");
assertFixtureTargets(fixtures.semantic_families,
  ledger.fixture_assignments.map(({ fixture_family_id: id }) => ({ id })),
"semantic family");
assertFixtureTargets(fixtures.policies,
  ledger.policy_assignments.map(({ field: id }) => ({ id })), "policy");
for (const fixture of allFixtures(fixtures)) {
  assert(fixture.execution_status === "ProductionExecutionCovered"
    && fixture.execution_evidence.owner_audit.startsWith(
      "content-manifests/currency-wars-runtime-v1/",
    )
    && fixture.execution_evidence.matrix_audit.endsWith(
      "/legal-matrix-execution-audit.json",
    ), `${fixture.id} has no production execution evidence`);
  assert(release.complete_runs.some(({ id }) => id === fixture.assigned_matrix_entry_id),
    `${fixture.id} is not assigned to a matrix entry`);
}

assert(release.replay_identity.component_order.length === 9,
  "replay component count drift");
assert(JSON.stringify(release.replay_identity.component_order)
  === JSON.stringify(contract.component_set.map(({ kind, id }) => ({ kind, id }))),
"replay component order is not the frozen runtime contract order");
assert(release.replay_identity.status === "FreshReplayGoldenVerified",
  "fresh replay golden is not terminal");
assert(release.replay_identity.first_divergence_required,
  "replay identity must require first-divergence reporting");
assert(release.performance_workloads.status === "ExecutableBaselineFrozen"
  && release.performance_workloads.workloads.length === 8,
  "performance workload count drift");
assert(release.performance_workloads.workloads.find(({ id }) =>
  id === "warm-shared-catalog-session-start").iterations === 10_000,
"warm cache workload drift");
assert(release.native_ci.runtime_evidence.map(({ target }) => target).join(",")
  === "x86_64-pc-windows-msvc,x86_64-unknown-linux-gnu,aarch64-apple-darwin",
"native runtime-evidence matrix drift");
assert(release.native_ci.compile_only.length === 3,
  "native compile-only matrix drift");
verifyNativeCiClosure(release.native_ci);
assert(dispositions.summary.native_handlers_admitted === 0,
  "P0 coverage must not admit native handlers");

const serialized = JSON.stringify(release);
for (const forbidden of ["IdentityOnly", "CatalogOnly", "CompletedProductionExecution"])
  assert(!serialized.includes(forbidden), `coverage artifact contains forbidden credit: ${forbidden}`);

console.log(
  `Currency Wars coverage and release verified (${release.complete_runs.length} runs; `
    + `${fixtures.investments.length} investments; ${fixtures.bond_levels.length} Bond levels; `
    + `${fixtures.encounters.enemy_affixes.length} affixes; 8 workloads; 3 native runners).`,
);

function investmentRows() {
  return [
    "augment-definitions.json", "enhancements.json", "orbs.json",
    "portal-buffs.json", "projections.json", "talents.json",
  ].flatMap((file) => rows(file));
}

function allFixtures(fixtures) {
  return [
    fixtures.investments, fixtures.roles, fixtures.team_sizes,
    fixtures.rank_boundaries, fixtures.star_transitions, fixtures.bond_levels,
    fixtures.bond_contributions, ...Object.values(fixtures.encounters),
    fixtures.battle_boundaries, fixtures.mechanic_partitions,
    fixtures.semantic_families, fixtures.policies,
  ].flat();
}

function verifyNativeCiClosure(nativeCi) {
  const policy = json("policy/currency-wars-native-evidence.json");
  const runtimeTargets = nativeCi.runtime_evidence.map(({ target }) => target);
  assert(JSON.stringify(policy.runtime_targets) === JSON.stringify(runtimeTargets),
    "native evidence policy runtime targets drift from the release matrix");
  assert(JSON.stringify(policy.compile_only_targets)
    === JSON.stringify(nativeCi.compile_only),
  "native evidence policy compile-only targets drift from the release matrix");

  const workflow = fs.readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const jobMarker = "  currency-wars-native:";
  const jobOffset = workflow.indexOf(jobMarker);
  assert(jobOffset >= 0, "Currency Wars native CI job is missing");
  const job = workflow.slice(jobOffset);
  const entries = [...job.matchAll(
    /-\s+runner:\s*(\S+)\s+target:\s*(\S+)\s+compile_target:\s*(\S+)/g,
  )].map((match) => ({
    runner: match[1],
    target: match[2],
    compile_target: match[3],
  }));
  const expectedEntries = [
    {
      runner: "windows-2025",
      target: "x86_64-pc-windows-msvc",
      compile_target: "aarch64-pc-windows-msvc",
    },
    {
      runner: "ubuntu-24.04",
      target: "x86_64-unknown-linux-gnu",
      compile_target: "aarch64-unknown-linux-gnu",
    },
    {
      runner: "macos-15",
      target: "aarch64-apple-darwin",
      compile_target: "x86_64-apple-darwin",
    },
  ];
  assert(JSON.stringify(entries) === JSON.stringify(expectedEntries),
    "hosted native CI runner/target pairs drift from the release contract");
  assert(job.includes("if: matrix.compile_target == 'aarch64-unknown-linux-gnu'")
    && job.includes("sudo apt-get update && sudo apt-get install --yes "
      + "--no-install-recommends gcc-aarch64-linux-gnu libc6-dev-arm64-cross"),
  "Linux ARM64 compile-only target is missing its C cross toolchain");
  for (const command of [
    "cargo test --release -p starclock-ai --test currency_wars_baseline",
    "cargo test --release -p starclock-ai --test currency_wars_matrix -- --ignored --exact generated_legal_matrix_completes_real_battles_and_fresh_replay",
    "node tools/currency-wars-runtime/native-evidence.mjs --target ${{ matrix.target }} --check --output evidence/currency-wars-runtime-v1/native-${{ matrix.target }}.json",
    "cargo check --workspace --all-targets --target ${{ matrix.compile_target }}",
  ]) {
    assert(job.includes(command), `hosted native CI command is missing: ${command}`);
  }
}

function assertFixtureTargets(fixtures, expectedRows, label) {
  assertExactIds(fixtures.map(({ target_id: id }) => id), expectedRows,
    `${label} fixture targets`, true);
  assertUnique(fixtures.map(({ id }) => id), `${label} fixture ID`);
}

function assertExactIds(actual, expectedRows, label, exactOnce = true) {
  const actualSet = new Set(actual);
  const expected = expectedRows.map(({ id }) => id);
  assert(actualSet.size === expected.length
    && expected.every((id) => actualSet.has(id)), `${label} coverage drift`);
  if (exactOnce)
    assert(actual.length === expected.length, `${label} is not exact-once`);
}

function assertUnique(values, label) {
  assert(new Set(values).size === values.length, `${label} is not unique`);
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
  assert(Array.isArray(value), `${file} is not a row array`);
  return value;
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}
