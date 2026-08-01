import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(
  process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".",
);
const bless = process.argv.includes("--bless");
const policyPath = "policy/goal07-integrated-scenarios.json";
const progressPath =
  "evidence/standard-universe-mechanics-complete-v1/content-progress.json";
const auditPath =
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json";
const matrixSource =
  "crates/starclock-agent-api/examples/g05_real_universe_seed_matrix.rs";
const targetedEvidencePath =
  "evidence/standard-universe-mechanics-complete-v1/integration/targeted-scenarios.json";
const matrixEvidencePath =
  "evidence/standard-universe-mechanics-complete-v1/integration/seeded-matrix.json";

const policy = json(policyPath);
assert(
  policy.schema_revision === "starclock.goal07-integrated-scenarios.v1",
  "unexpected integrated-scenario policy revision",
);
const progress = json(progressPath);
const audit = json(auditPath);
assert(progress.result === "complete", "content progress is not complete");
assert(
  progress.total_partitions === policy.coverage.content_partitions &&
    progress.completed_partitions === policy.coverage.content_partitions &&
    progress.pending_partitions === 0,
  "content partition coverage drift",
);
for (const [section, expected] of [
  ["records", policy.coverage.content_records],
  ["rules", policy.coverage.mechanic_rules],
  ["fixtures", policy.coverage.semantic_fixtures],
  ["enemy_variants", policy.coverage.enemy_variants],
  ["encounter_members", policy.coverage.encounter_members],
]) {
  assert(audit.summary[section].total === expected, `${section} audit denominator drift`);
}

const receiptDigests = [];
const ruleIds = new Set();
const fixtureIds = new Set();
const enemyIds = new Set();
const memberIds = new Set();
const executionPaths = new Set();
const fixtureMarkers = new Set();
const runtimeDispositions = new Map();
for (const [ordinal, row] of progress.rows.entries()) {
  assert(row.ordinal === ordinal, `partition ${row.id} ordinal drift`);
  assert(row.state === "Complete", `partition ${row.id} is not complete`);
  const receipt = json(row.receipt);
  assert(receipt.partition_id === row.id, `partition ${row.id} receipt mismatch`);
  assert(receipt.state === "Complete", `partition ${row.id} receipt is not complete`);
  const digest = sha256(row.receipt);
  assert(digest === row.receipt_sha256, `partition ${row.id} receipt digest drift`);
  receiptDigests.push(`${row.id}:${digest}`);
  for (const rule of receipt.rules ?? []) {
    assert(!ruleIds.has(rule.id), `rule ${rule.id} has duplicate receipts`);
    assert(
      ["ExecutableRuleIr", "ExecutableSharedPrimitive", "ProfileExcluded"].includes(
        rule.runtime_disposition,
      ),
      `rule ${rule.id} has an unclosed runtime disposition`,
    );
    assert(
      Array.isArray(rule.execution_evidence) && rule.execution_evidence.length > 0,
      `rule ${rule.id} lacks execution evidence`,
    );
    ruleIds.add(rule.id);
    runtimeDispositions.set(
      rule.runtime_disposition,
      (runtimeDispositions.get(rule.runtime_disposition) ?? 0) + 1,
    );
    for (const evidence of rule.execution_evidence) {
      const currentPath = assertFile(evidence.path, `rule ${rule.id} execution evidence`);
      executionPaths.add(currentPath);
    }
  }
  for (const fixture of receipt.fixtures ?? []) {
    assert(!fixtureIds.has(fixture.id), `fixture ${fixture.id} has duplicate receipts`);
    const currentPath = assertFile(fixture.test_path, `fixture ${fixture.id} test`);
    assert(
      text(currentPath).includes(fixture.test_marker),
      `fixture ${fixture.id} test marker drift`,
    );
    fixtureIds.add(fixture.id);
    executionPaths.add(currentPath);
    fixtureMarkers.add(fixture.test_marker);
  }
  collectUnique(receipt.enemy_variants ?? [], enemyIds, "enemy variant");
  collectUnique(receipt.encounter_members ?? [], memberIds, "encounter member");
}
for (const [values, expected, label] of [
  [ruleIds, policy.coverage.mechanic_rules, "rules"],
  [fixtureIds, policy.coverage.semantic_fixtures, "fixtures"],
  [enemyIds, policy.coverage.enemy_variants, "enemy variants"],
  [memberIds, policy.coverage.encounter_members, "encounter members"],
]) {
  assert(values.size === expected, `${label} receipt denominator drift`);
}

const familyCounts = new Map();
for (const rule of audit.rules) {
  familyCounts.set(rule.mechanic_family, (familyCounts.get(rule.mechanic_family) ?? 0) + 1);
  assert(ruleIds.has(rule.id), `audited rule ${rule.id} lacks a complete receipt`);
}
assert(
  familyCounts.size === policy.coverage.mechanic_families,
  "mechanic-family denominator drift",
);
const mechanicFamilies = [...familyCounts]
  .sort(([left], [right]) => left.localeCompare(right))
  .map(([family, rules]) => ({ family, rules }));

for (const scenario of policy.dynamic_scenarios) {
  const currentPath = assertFile(scenario.path, `${scenario.boundary} scenario`);
  assert(
    text(currentPath).includes(scenario.marker),
    `${scenario.boundary} scenario marker drift`,
  );
}
for (const command of policy.focused_commands) {
  for (const [program, ...args] of currentFocusedInvocations(command))
    execFileSync(program, args, { cwd: root, stdio: "inherit" });
}

const matrixStdout = execFileSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--release",
    "-p",
    "starclock-agent-api",
    "--example",
    "g05_real_universe_seed_matrix",
  ],
  {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
    stdio: ["ignore", "pipe", "inherit"],
  },
);
const matrix = JSON.parse(matrixStdout.trim());
assert(matrix.schema_revision === policy.matrix_revision, "matrix revision drift");
assert(matrix.executor_revision === policy.executor_revision, "executor revision drift");
for (const [field, expected] of [
  ["worlds", policy.coverage.worlds],
  ["difficulties", policy.coverage.difficulties],
  ["distinct_path_options", policy.coverage.path_options],
  ["complete_runs", policy.coverage.complete_runs],
]) {
  assert(matrix.coverage[field] === expected, `${field} matrix coverage drift`);
}
assert(matrix.coverage.first_seed === policy.first_seed, "first seed drift");
assert(matrix.runs.length === policy.coverage.complete_runs, "run denominator drift");
const worlds = new Set();
const difficulties = new Set();
const paths = new Set();
const seeds = new Set();
let battleCommands = 0;
let battleStateRecords = 0;
let nestedBattles = 0;
for (const [ordinal, run] of matrix.runs.entries()) {
  assert(run.ordinal === ordinal, `run ${ordinal} ordinal drift`);
  assert(run.seed === policy.first_seed + ordinal, `run ${ordinal} seed drift`);
  assert(run.terminal === "completed", `run ${ordinal} did not complete`);
  assert(
    Number(run.external_actions) > 0 &&
      Number(run.replay_actions) >= Number(run.external_actions),
    `run ${ordinal} action counts are invalid`,
  );
  assert(
    Number(run.battle_commands) === Number(run.battle_state_records),
    `run ${ordinal} battle command/state parity drift`,
  );
  assert(
    /^[0-9a-f]{64}$/.test(run.final_state_hash) &&
      /^[0-9a-f]{64}$/.test(run.replay_sha256),
    `run ${ordinal} hash is invalid`,
  );
  worlds.add(run.world);
  difficulties.add(`${run.world}/${run.difficulty_index}`);
  paths.add(run.path_option_id);
  seeds.add(run.seed);
  nestedBattles += Number(run.nested_battles);
  battleCommands += Number(run.battle_commands);
  battleStateRecords += Number(run.battle_state_records);
}
assert(worlds.size === policy.coverage.worlds, "World coverage is incomplete");
assert(
  difficulties.size === policy.coverage.difficulties,
  "difficulty coverage is incomplete",
);
assert(paths.size === policy.coverage.path_options, "Path coverage is incomplete");
assert(seeds.size === policy.coverage.complete_runs, "matrix seeds are not unique");
assert(nestedBattles > 0, "the complete matrix executed no nested battles");
assert(
  battleCommands === battleStateRecords &&
    battleCommands === Number(matrix.coverage.battle_commands) &&
    battleStateRecords === Number(matrix.coverage.battle_state_records),
  "aggregate battle command/state parity drift",
);

const policyDigest = sha256(policyPath);
const receiptSetDigest = digestLines(receiptDigests);
const archivedTargetedEvidence = bless ? null : json(targetedEvidencePath);
const archivedMatrixEvidence = bless ? null : json(matrixEvidencePath);
if (archivedTargetedEvidence && archivedMatrixEvidence) {
  const archivedPolicyDigest = archivedTargetedEvidence.sha256[policyPath];
  assert(/^[0-9a-f]{64}$/.test(archivedPolicyDigest),
    "historical integrated-scenario policy digest is invalid");
  assert(archivedMatrixEvidence.sha256[policyPath] === archivedPolicyDigest,
    "historical integrated-scenario policy digests differ");
  assert(archivedTargetedEvidence.dynamic_scenarios.length === policy.dynamic_scenarios.length,
    "historical dynamic-scenario denominator drift");
  for (const scenario of archivedTargetedEvidence.dynamic_scenarios) {
    const current = policy.dynamic_scenarios.find(({ boundary }) => boundary === scenario.boundary);
    assert(current?.marker === scenario.marker,
      `historical ${scenario.boundary} scenario marker drift`);
    const currentPath = assertFile(scenario.path, `historical ${scenario.boundary} scenario`);
    assert(text(currentPath).includes(scenario.marker),
      `historical ${scenario.boundary} scenario execution marker drift`);
  }
}
const targetedEvidence = {
  schema_revision: "starclock.goal07-targeted-scenario-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "pass",
  coverage: {
    content_partitions: progress.completed_partitions,
    content_records: audit.summary.records.total,
    mechanic_rules: ruleIds.size,
    semantic_fixtures: fixtureIds.size,
    enemy_variants: enemyIds.size,
    encounter_members: memberIds.size,
    mechanic_families: mechanicFamilies,
    runtime_dispositions: Object.fromEntries(
      [...runtimeDispositions].sort(([left], [right]) => left.localeCompare(right)),
    ),
    execution_paths: executionPaths.size,
    fixture_markers: fixtureMarkers.size,
    dynamic_boundaries: policy.dynamic_scenarios.length,
  },
  dynamic_scenarios: archivedTargetedEvidence?.dynamic_scenarios ?? policy.dynamic_scenarios,
  focused_commands: policy.focused_commands,
  contracts: policy.contracts,
  sha256: {
    [policyPath]: archivedTargetedEvidence?.sha256[policyPath] ?? policyDigest,
    [progressPath]: sha256(progressPath),
    [auditPath]: sha256(auditPath),
    receipt_set: receiptSetDigest,
  },
  new_registry_packages: [],
};
const matrixEvidence = {
  schema_revision: "starclock.goal07-seeded-matrix-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: matrix.result,
  matrix,
  contracts: policy.contracts,
  sha256: {
    [policyPath]: archivedMatrixEvidence?.sha256[policyPath] ?? policyDigest,
    [matrixSource]: sha256(matrixSource),
  },
  new_registry_packages: [],
};
writeOrVerify(targetedEvidencePath, targetedEvidence);
writeOrVerify(matrixEvidencePath, matrixEvidence);
console.log(
  `Goal 07 integrated scenarios verified (${ruleIds.size} rules, ${fixtureIds.size} fixtures, ` +
    `${mechanicFamilies.length} families, ${matrix.runs.length} runs, ${nestedBattles} battles).`,
);

function collectUnique(entries, target, label) {
  for (const entry of entries) {
    assert(!target.has(entry.id), `${label} ${entry.id} has duplicate receipts`);
    target.add(entry.id);
  }
}
function currentFocusedInvocations(command) {
  const packageIndex = command.indexOf("-p") + 1;
  const packageName = command[packageIndex];
  const suite = packageName === "starclock-combat"
    ? "combat_suite"
    : packageName === "starclock-mode-universe"
      ? "universe_suite"
      : null;
  if (!suite) return [command];
  const filters = [];
  for (let index = 0; index < command.length; index += 1)
    if (command[index] === "--test") filters.push(command[index + 1]);
  return filters.map((filter) => [
    "cargo", "test", "-p", "starclock-test-kit", "--test", suite,
    filter, "--all-features",
  ]);
}
function assertFile(relative, label) {
  assert(typeof relative === "string", `${label} path is invalid`);
  if (fs.existsSync(path.join(root, relative))) return relative;
  for (const [historicalRoot, currentRoot] of [
    ["crates/starclock-mode-universe/tests/", "crates/starclock-test-kit/tests/suites/universe/"],
    ["crates/starclock-combat/tests/", "crates/starclock-test-kit/tests/suites/core/combat/"],
    ["crates/starclock-activity/tests/", "crates/starclock-test-kit/tests/suites/activity/activity/"],
    ["crates/starclock-replay/tests/", "crates/starclock-test-kit/tests/suites/exhaustive/replay/"],
  ]) {
    if (!relative.startsWith(historicalRoot)) continue;
    const candidate = `${currentRoot}${relative.slice(historicalRoot.length)}`;
    if (fs.existsSync(path.join(root, candidate))) return candidate;
  }
  assert(false, `${label} is missing`);
}
function writeOrVerify(relative, value) {
  const output = `${JSON.stringify(value, null, 2)}\n`;
  if (bless) {
    fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
    fs.writeFileSync(path.join(root, relative), output);
  } else {
    assertFile(relative, `${relative} evidence`);
    assert(
      text(relative).replaceAll("\r\n", "\n") === output,
      `${relative} is stale; run with --bless`,
    );
  }
}
function digestLines(lines) {
  return crypto.createHash("sha256").update(`${lines.join("\n")}\n`).digest("hex");
}
function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(relative) {
  return crypto
    .createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative)))
    .digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
