#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const packRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
);
const outputPath = path.join(
  root,
  "evidence",
  "galactic-baseballer-reference-v1",
  "reference-boundary-results.json",
);
const check = process.argv.includes("--check");
const branchBase = "0191cc71b1735d6e101e6e04817181423c599232";
const profileIds = [
  "galactic-baseballer.demon-king.v3_3",
  "galactic-baseballer.departure.v2_2",
];
const protectedRoots = [
  "config/generated",
  "config/universe-generated",
  "config/gold-and-gears-generated",
  "config/swarm-disaster-generated",
  "config/unknowable-domain-generated",
  "config/divergent-universe-generated",
  "config/currency-wars-generated",
  "config/anomaly-arbitration-generated",
  "content-manifests/standard-universe-v1",
  "content-manifests/gold-and-gears-v1",
  "content-manifests/swarm-disaster-v1",
  "content-manifests/unknowable-domain-v1",
  "content-manifests/divergent-universe-v1",
  "content-manifests/currency-wars-v1",
  "content-manifests/anomaly-arbitration-v1",
  "content-reference/standard-universe-v1",
  "content-reference/gold-and-gears-v1",
  "content-reference/swarm-disaster-v1",
  "content-reference/unknowable-domain-v1",
  "content-reference/divergent-universe-v1",
  "content-reference/currency-wars-v1",
  "content-reference/anomaly-arbitration-v1",
  "evidence/standard-universe-reference-v1",
  "evidence/gold-and-gears-reference-v1",
  "evidence/swarm-disaster-reference-v1",
  "evidence/unknowable-domain-reference-v1",
  "evidence/divergent-universe-reference-v1",
  "evidence/currency-wars-reference-v1",
  "evidence/anomaly-arbitration-reference-v1",
];
const runtimeScanRoots = [
  "crates",
  "config/data",
  "config/schema",
  "config/generated",
  "config/universe-generated",
  "config/gold-and-gears-generated",
  "config/swarm-disaster-generated",
  "config/unknowable-domain-generated",
  "config/divergent-universe-generated",
  "config/currency-wars-generated",
  "config/anomaly-arbitration-generated",
  "content-manifests/standard-universe-v1",
  "content-manifests/gold-and-gears-v1",
  "content-manifests/swarm-disaster-v1",
  "content-manifests/unknowable-domain-v1",
  "content-manifests/divergent-universe-v1",
  "content-manifests/currency-wars-v1",
  "content-manifests/anomaly-arbitration-v1",
  "content-reference/standard-universe-v1",
  "content-reference/gold-and-gears-v1",
  "content-reference/swarm-disaster-v1",
  "content-reference/unknowable-domain-v1",
  "content-reference/divergent-universe-v1",
  "content-reference/currency-wars-v1",
  "content-reference/anomaly-arbitration-v1",
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function digest(value) {
  return createHash("sha256").update(canonical(value)).digest("hex");
}

function git(args, options = {}) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    ...options,
  }).trim();
}

function lines(value) {
  return value === "" ? [] : value.split("\n").filter(Boolean);
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function allowedPath(file) {
  return [
    "config/galactic-baseballer/",
    "config/galactic-baseballer-generated/",
    "content-manifests/galactic-baseballer-v1/",
    "content-reference/galactic-baseballer-v1/",
    "evidence/galactic-baseballer-reference-v1/",
    "tools/galactic-baseballer-reference/",
  ].some((prefix) => file.startsWith(prefix))
    || file === "docs/goal-16-foundation.md"
    || file.startsWith("docs/goals/16-galactic-baseballer-")
    || file === "docs/goals/README.md"
    || file === "policy/goal16-foundation.json"
    || file === "policy/repository-checks.json";
}

async function json(file) {
  return JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
}

const [
  profiles,
  profileDifferences,
  weapons,
  accessories,
  recipes,
  inputs,
  enemies,
  enemySkills,
  enemyStatuses,
  coverage,
  gaps,
] = await Promise.all([
  json("profiles.json"),
  json("profile-differences.json"),
  json("weapons.json"),
  json("accessories.json"),
  json("synthesis-recipes.json"),
  json("synthesis-inputs.json"),
  json("enemies.json"),
  json("enemy-skills.json"),
  json("enemy-statuses.json"),
  json("coverage.json"),
  json("research-gaps.json"),
]);

assert(profiles.length === 2, "profile denominator drift");
assert(
  canonical(profiles.map(({ id }) => id).sort(compareText))
    === canonical(profileIds),
  "versioned profile identity drift",
);
const departure = profiles.find(({ id }) =>
  id === "galactic-baseballer.departure.v2_2");
const demon = profiles.find(({ id }) =>
  id === "galactic-baseballer.demon-king.v3_3");
assert(departure.released_version === "2.2", "Departure release version drift");
assert(demon.released_version === "3.3", "Demon King release version drift");
assert(
  profiles.every(({ retained_baseline_version: version }) => version === "4.4"),
  "retained Version 4.4 baseline drift",
);
assert(
  profiles.every(({ runtime_enabled: enabled }) => enabled === false),
  "a reference profile became runtime enabled",
);
assert(
  profiles.every(({ shared_system_id: id }) =>
    id === "galactic-baseballer.shared-base.v1"),
  "both profiles do not explicitly bind the shared base",
);
assert(
  departure.activity_module_id === demon.activity_module_id
    && departure.activity_module_id === "5003501",
  "shared activity module drift",
);
assert(
  demon.does_not_replace_profile_ids.includes(departure.id),
  "Demon King no longer explicitly preserves Departure",
);
assert(
  profileDifferences.length === 1
    && profileDifferences[0].relationship_counts.SharedValueExplicitlyRepeated
      === 38
    && profileDifferences[0].relationship_counts.DemonKingChanged === 25
    && profileDifferences[0].relationship_counts.DemonKingAdded === 13
    && profileDifferences[0].relationship_counts.DepartureOnlyNotInherited
      === 7,
  "profile difference accounting drift",
);

const weaponIds = new Set(weapons.map(({ id }) => id));
const accessoryIds = new Set(accessories.map(({ id }) => id));
const recipeById = new Map(recipes.map((recipe) => [recipe.id, recipe]));
const inputsByRecipe = new Map();
for (const input of inputs) {
  const rows = inputsByRecipe.get(input.recipe_id) ?? [];
  rows.push(input);
  inputsByRecipe.set(input.recipe_id, rows);
}
assert(recipes.length === 27, "synthesis recipe denominator drift");
assert(inputs.length === 54, "synthesis input denominator drift");
assert(recipeById.size === recipes.length, "duplicate synthesis recipe ID");
assert(
  new Set(recipes.map(({ output_weapon_id: id }) => id)).size
    === recipes.length,
  "duplicate synthesis output",
);

const recipeByOutput = new Map(
  recipes.map((recipe) => [recipe.output_weapon_id, recipe]),
);
const graph = new Map(recipes.map(({ id }) => [id, []]));
for (const recipe of recipes) {
  const recipeInputs = (inputsByRecipe.get(recipe.id) ?? [])
    .sort((left, right) => left.input_order - right.input_order);
  assert(recipeInputs.length === recipe.input_count, `${recipe.id}: input count drift`);
  assert(
    recipeInputs.every(({ input_order: order }, index) => order === index),
    `${recipe.id}: input order gap`,
  );
  assert(weaponIds.has(recipe.output_weapon_id), `${recipe.id}: unknown output`);
  for (const input of recipeInputs) {
    assert(
      (input.input_kind === "Weapon" && weaponIds.has(input.input_id))
        || (input.input_kind === "Accessory" && accessoryIds.has(input.input_id)),
      `${input.id}: unknown typed input`,
    );
    assert(
      canonical(input.profile_ids) === canonical(recipe.profile_ids),
      `${input.id}: cross-profile recipe input`,
    );
    const prerequisite = recipeByOutput.get(input.input_id);
    if (prerequisite !== undefined) graph.get(prerequisite.id).push(recipe.id);
  }
}
const indegree = new Map(recipes.map(({ id }) => [id, 0]));
for (const targets of graph.values()) {
  for (const target of targets) indegree.set(target, indegree.get(target) + 1);
}
const ready = [...indegree].filter(([, degree]) => degree === 0)
  .map(([id]) => id).sort(compareText);
const topologicalOrder = [];
while (ready.length > 0) {
  const current = ready.shift();
  topologicalOrder.push(current);
  for (const target of [...graph.get(current)].sort(compareText)) {
    const degree = indegree.get(target) - 1;
    indegree.set(target, degree);
    if (degree === 0) {
      ready.push(target);
      ready.sort(compareText);
    }
  }
}
assert(topologicalOrder.length === recipes.length, "synthesis graph contains a cycle");

const enemyBySource = new Map();
for (const enemy of enemies) {
  assert(enemy.resolution === "ExactStableIdentity", `${enemy.id}: unresolved enemy`);
  assert(enemy.ownership === "Shared", `${enemy.id}: copied profile-owned enemy`);
  assert(
    enemy.source_refs.some(({ path_or_page: sourcePath, note }) =>
      sourcePath === "content-reference/v4.4/enemy-variants.json"
      && note.includes("without copying")),
    `${enemy.id}: inherited identity receipt missing`,
  );
  const identity = canonical([
    enemy.inherited_enemy_variant_id,
    enemy.inherited_enemy_template_id,
  ]);
  const prior = enemyBySource.get(enemy.source_monster_id);
  assert(prior === undefined || prior === identity, `${enemy.source_monster_id}: identity collision`);
  enemyBySource.set(enemy.source_monster_id, identity);
}
const skillBySource = new Map();
for (const skill of enemySkills) {
  assert(skill.resolution === "ExactStableIdentity", `${skill.id}: unresolved enemy skill`);
  assert(skill.ownership === "Shared", `${skill.id}: copied profile-owned skill`);
  assert(
    skill.source_refs.some(({ path_or_page: sourcePath, note }) =>
      sourcePath === "content-reference/v4.4/enemy-abilities.json"
      && note.includes("without duplication")),
    `${skill.id}: inherited ability receipt missing`,
  );
  const identity = canonical([
    skill.inherited_enemy_ability_id,
    skill.inherited_enemy_id,
  ]);
  const prior = skillBySource.get(skill.source_skill_id);
  assert(prior === undefined || prior === identity, `${skill.source_skill_id}: ability collision`);
  skillBySource.set(skill.source_skill_id, identity);
}
assert(
  enemyStatuses.every(({ resolution, ownership }) =>
    resolution === "ExactSourceLocator" && ownership === "Shared"),
  "enemy status locator boundary drift",
);

assert(coverage.length === 2232, "coverage denominator drift");
assert(
  coverage.filter(({ coverage_state: state }) => state === "DataReady").length
    === 2207,
  "DataReady count drift",
);
assert(
  coverage.filter(({ coverage_state: state }) => state === "EvidenceOnly")
    .length === 25,
  "EvidenceOnly count drift",
);
assert(
  !coverage.some(({ coverage_state: state }) => state === "Blocked"),
  "blocking coverage row remains",
);
assert(
  gaps.length === 12
    && gaps.every(({ state, terminal_blocker: blocker }) =>
      state === "ReplaceableNonBlocking" && blocker === false),
  "replacement-boundary state drift",
);

const protectedResults = protectedRoots.map((protectedPath) => {
  const before = git(["rev-parse", `${branchBase}:${protectedPath}`]);
  const after = git(["rev-parse", `HEAD:${protectedPath}`]);
  assert(before === after, `protected root changed: ${protectedPath}`);
  return { path: protectedPath, tree_sha: after };
});
const committedChanges = lines(git([
  "diff",
  "--name-only",
  `${branchBase}..HEAD`,
]));
const workingChanges = [
  ...lines(git(["diff", "--name-only"])),
  ...lines(git(["diff", "--name-only", "--cached"])),
  ...lines(git(["ls-files", "--others", "--exclude-standard"])),
];
const changedPaths = [...new Set([...committedChanges, ...workingChanges])]
  .sort(compareText);
assert(
  changedPaths.every(allowedPath),
  `out-of-bound changed path: ${changedPaths.find((file) => !allowedPath(file))}`,
);

let runtimeMatches = "";
try {
  runtimeMatches = git([
    "grep",
    "-I",
    "-n",
    "-i",
    "-e",
    "galactic-baseballer",
    "-e",
    "银河球棒侠",
    "--",
    ...runtimeScanRoots,
  ]);
} catch (error) {
  assert(error.status === 1, "runtime isolation scan failed");
}
assert(runtimeMatches === "", "Galactic Baseballer leaked into runtime/other-mode roots");
const worktrees = git(["worktree", "list", "--porcelain"]);
const mainWorktree = "/Users/mikai/CLionProjects/starclock";
assert(root !== mainWorktree, "Goal 16 is running in the main worktree");
assert(
  worktrees.includes(`worktree ${root}`)
    && worktrees.includes(`worktree ${mainWorktree}`),
  "parallel worktree registration drift",
);
assert(
  git(["branch", "--show-current"])
    === "codex/goal16-galactic-baseballer-reference",
  "Goal 16 branch drift",
);

const tierCounts = Object.fromEntries(
  ["Legendary", "Twin", "Supreme"].map((tier) => [
    tier,
    recipes.filter(({ tier: value }) => value === tier).length,
  ]),
);
const result = {
  schema_revision: "starclock.galactic-baseballer-reference-boundary-results.v1",
  goal_id: "galactic-baseballer-reference-v1",
  batch_id: "G16-P4-B2",
  baseline_game_version: "4.4",
  status: "Passed",
  profile_audit: {
    profile_ids: profileIds,
    released_versions: ["2.2", "3.3"],
    retained_baseline_version: "4.4",
    shared_system_id: "galactic-baseballer.shared-base.v1",
    shared_activity_module_id: "5003501",
    later_profile_replaces_original: false,
    runtime_enabled_profile_count: 0,
    difference_relationship_counts: profileDifferences[0].relationship_counts,
  },
  shared_identity_audit: {
    enemy_resolution_rows: enemies.length,
    distinct_source_monster_ids: enemyBySource.size,
    enemy_identity_collisions: 0,
    enemy_skill_resolution_rows: enemySkills.length,
    distinct_source_skill_ids: skillBySource.size,
    enemy_skill_identity_collisions: 0,
    source_status_locator_rows: enemyStatuses.length,
    copied_enemy_or_skill_definitions: 0,
  },
  synthesis_audit: {
    recipe_count: recipes.length,
    input_count: inputs.length,
    tier_counts: tierCounts,
    graph_edge_count: [...graph.values()]
      .reduce((sum, targets) => sum + targets.length, 0),
    topological_order_count: topologicalOrder.length,
    topological_order_sha256: digest(topologicalOrder),
    cycle_count: 0,
    cross_profile_input_count: 0,
    unknown_input_or_output_count: 0,
  },
  coverage_audit: {
    frozen_obligation_count: coverage.length,
    data_ready_count: 2207,
    evidence_only_count: 25,
    blocked_count: 0,
    replaceable_nonblocking_gap_count: gaps.length,
  },
  isolation_audit: {
    branch_base: branchBase,
    branch: git(["branch", "--show-current"]),
    worktree: root,
    main_worktree: mainWorktree,
    changed_path_scope: "all committed and working-tree paths validated",
    out_of_boundary_changed_path_count: 0,
    protected_root_count: protectedResults.length,
    protected_roots: protectedResults,
    runtime_or_other_mode_match_count: 0,
  },
};
const encoded = `${JSON.stringify(result, null, 2)}\n`;
if (check) {
  assert(await readFile(outputPath, "utf8") === encoded, "reference boundary result drift");
} else {
  await writeFile(outputPath, encoded);
}
console.log(
  `reference boundaries ${check ? "verified" : "audited"}: `
    + "2 profiles, "
    + `${enemies.length} enemy and ${enemySkills.length} skill identity rows, `
    + `${recipes.length} acyclic recipes, ${protectedResults.length} protected roots`,
);
