import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(process.argv[2] && !process.argv[2].startsWith("--") ? process.argv[2] : ".");
const bless = process.argv.includes("--bless");
const policy = json("policy/goal04-foundation.json");
assert(policy.schema_revision === "starclock.goal04-foundation.v1", "unexpected foundation revision");

const slice = policy.vertical_slice;
const world = find("worlds.json", slice.world_id);
const difficulty = find("world-difficulties.json", slice.difficulty_id);
const universePath = find("paths.json", slice.path_id);
const room = find("rooms.json", slice.room_id);
const domain = find("domains.json", slice.domain_id);
const pool = find("encounter-pools.json", slice.encounter_pool_id);
const group = find("encounter-groups.json", slice.encounter_group_id);
const blessing = find("blessings.json", slice.reward_blessing_id);
const level1 = find("blessing-levels.json", slice.reward_blessing_level_id);
const level2 = find("blessing-levels.json", slice.enhanced_blessing_level_id);
const service = find("services.json", slice.service_id);

assert(world.difficulty_ids.includes(difficulty.id), "vertical-slice difficulty is not in World 1");
assert(difficulty.world_id === world.id && difficulty.profile_kind === "Standard", "vertical-slice difficulty is not Standard World 1");
assert(room.domain_id === domain.id && pool.room_id === room.id, "room/domain/pool binding differs");
const candidate = pool.weighted_group_ids.find((value) => value.condition_key === slice.encounter_condition_key);
assert(candidate?.group_id === group.id, "encounter condition does not resolve the frozen group");
const enemies = sorted(group.weighted_member_ids.flatMap((member) => member.waves.flatMap((wave) => wave.enemy_variant_ids.map((enemy) => enemy.enemy_variant_id))));
assert(equal(enemies, sorted(slice.enemy_variant_ids)), "vertical-slice enemy variants differ");
assert(blessing.path_id === universePath.id && level1.blessing_id === blessing.id && level1.level === 1, "level-1 reward binding differs");
assert(level2.blessing_id === blessing.id && level2.level === 2, "enhanced reward binding differs");
assert(service.kind === "EnhanceBlessing", "vertical-slice service kind differs");

const characters = new Set(json("content-reference/v4.4/characters.json").map((record) => record.id));
assert(slice.participant_fixture.every((id) => characters.has(id)), "vertical-slice participant is missing");
const enemyVariants = new Set(json("content-reference/v4.4/enemy-variants.json").map((record) => record.id));
assert(slice.enemy_variant_ids.every((id) => enemyVariants.has(id)), "vertical-slice enemy is missing from core reference");
assert(slice.complete_world_claim === false && slice.claim.includes("not-complete-world"), "vertical slice overclaims World completion");

const performance = policy.performance;
assert(performance.workloads.length === 6, "performance workload count differs");
assert(new Set(performance.workloads.map((workload) => workload.id)).size === 6, "duplicate performance workload");
for (const metric of ["allocation_count", "catalog_clone_count", "replayed_prefix_count", "final_hash"])
  assert(performance.required_metrics.includes(metric), `performance metrics omit ${metric}`);
assert(performance.shape_invariants.catalog_clone_count_per_session === 0, "catalog clone invariant differs");
assert(performance.shape_invariants.replayed_prefix_count_per_incremental_command === 0, "replay prefix invariant differs");

const baselineCommit = capture("git", ["rev-parse", `${policy.dependency_baseline.baseline_commit}^{commit}`]).trim();
const baselineLock = Buffer.from(capture("git", ["show", `${baselineCommit}:Cargo.lock`]));
const baselineDependencyPolicy = Buffer.from(capture("git", ["show", `${baselineCommit}:policy/dependency-and-tool-policy.json`]));
assert(sha256Bytes(baselineLock) === policy.dependency_baseline.cargo_lock_sha256, "Cargo.lock baseline differs");
assert(sha256Bytes(baselineDependencyPolicy) === policy.dependency_baseline.reviewed_policy_sha256, "reviewed dependency policy baseline differs");
const baselinePackages = (baselineLock.toString("utf8").match(/^\[\[package\]\]$/gm) ?? []).length;
assert(baselinePackages === policy.dependency_baseline.cargo_packages, "locked baseline package count differs");
assert(policy.dependency_baseline.new_runtime_dependencies.length === 0, "foundation introduces an unreviewed runtime dependency");

assert(sha256("policy/ci-matrix.json") === policy.ci.inherited_policy_sha256, "inherited CI policy differs");
const inheritedCi = json("policy/ci-matrix.json");
assert(equal(inheritedCi.native_profiles.map((profile) => profile.id), policy.ci.native_profiles), "native CI profiles differ");
assert(equal(inheritedCi.compile_only_profiles.map((profile) => profile.id), policy.ci.compile_only_profiles), "compile-only CI profiles differ");
const workflow = text(policy.ci.workflow).replaceAll("\r\n", "\n");
assert(workflow.includes(`run: ${policy.ci.native_gate}`), "Goal 04 native CI gate is absent");

const status = text("docs/goals/04-standard-universe-runtime-status.md");
assert(/^\| `G04-P0-B4` \| `(InProgress|Complete)` \|/m.test(status), "G04-P0-B4 is not active or complete");
const document = text("docs/standard-universe-runtime-execution-baseline.md");
for (const marker of [slice.id, performance.workload_revision, "147-package", "fifty atomic batch", "not a release claim"])
  assert(document.includes(marker), `execution baseline omits ${marker}`);

const evidence = {
  schema_revision: "starclock.goal04-foundation-evidence.v1",
  goal_id: policy.goal_id,
  result: "frozen",
  vertical_slice: {
    id: slice.id,
    world: world.id,
    difficulty: difficulty.id,
    path: universePath.id,
    room: room.id,
    encounter_group: group.id,
    enemies,
    reward: blessing.id,
    service: service.id,
    complete_world_claim: false
  },
  performance: {
    revision: performance.workload_revision,
    workloads: performance.workloads.map((workload) => workload.id),
    required_metrics: performance.required_metrics,
    shape_invariants: performance.shape_invariants
  },
  ci: { native_profiles: policy.ci.native_profiles, compile_only_profiles: policy.ci.compile_only_profiles, native_gate: policy.ci.native_gate },
  dependencies: { baseline_commit: baselineCommit, cargo_packages: baselinePackages, cargo_lock_sha256: sha256Bytes(baselineLock), reviewed_policy_sha256: sha256Bytes(baselineDependencyPolicy), new_runtime_dependencies: 0 },
  release_scaffold_sha256: policy.release_scaffold_sha256
};
const relative = "evidence/standard-universe-runtime-v1/foundation/execution-baseline.json";
const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
  fs.writeFileSync(path.join(root, relative), output);
} else {
  assert(fs.existsSync(path.join(root, relative)), "foundation evidence is missing; run with --bless");
  assert(text(relative).replaceAll("\r\n", "\n") === output, "foundation evidence is stale; run with --bless");
}
const releasePolicy = json("policy/goal04-release-contract.json");
if (releasePolicy.state === "Scaffold")
  assert(sha256("policy/goal04-release-contract.json") === policy.release_scaffold_sha256, "release scaffold baseline differs");
console.log(`Goal 04 foundation verified (${slice.id}, ${performance.workloads.length} workloads, ${policy.ci.native_profiles.length}+${policy.ci.compile_only_profiles.length} CI profiles, ${baselinePackages} baseline packages).`);

function find(file, id) { const record = json(`content-reference/standard-universe-v1/${file}`).find((value) => value.id === id); assert(record, `${file} omits ${id}`); return record; }
function capture(command, args) { return execFileSync(command, args, { cwd: root, encoding: "utf8" }); }
function sorted(values) { return [...values].sort((a, b) => a < b ? -1 : a > b ? 1 : 0); }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function text(relative) { return fs.readFileSync(path.join(root, relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex"); }
function sha256Bytes(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
