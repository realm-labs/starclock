#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
assert(process.argv.slice(2).every((value) => value === "--check"),
  "usage: generate-coverage-matrix.mjs [--check]");
const policyPath = "policy/goal14-coverage-and-release.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal14-coverage-and-release.v1",
  "unsupported Goal 14 coverage policy");
for (const input of Object.values(policy.inputs))
  assert(sha256(input.path) === input.sha256, `coverage input drift: ${input.path}`);

const areas = json(policy.inputs.areas.path)
  .filter(({ area_group: group }) => group === "Formal")
  .sort(byId);
const dice = json(policy.inputs.dice_definitions.path).sort(byId);
const faces = new Map(json(policy.inputs.dice_faces.path)
  .map((face) => [face.source_id, face]));
const slots = json(policy.inputs.dice_slots.path).sort(byId);
const paths = json(policy.inputs.paths.path).sort(byId);
const conundrum = json(policy.inputs.conundrum_levels.path).sort((left, right) =>
  compare(left.track, right.track) || left.level - right.level);
const research = json(policy.inputs.research_gaps.path);
const matrixPolicy = policy.coverage_matrix;

assert(areas.length === matrixPolicy.formal_difficulties, "formal difficulty count drift");
assert(dice.length === matrixPolicy.custom_dice, "Custom Dice count drift");
assert(paths.length === matrixPolicy.paths, "Path count drift");
assert(slots.length === matrixPolicy.dice_slots, "dice slot count drift");
assert(conundrum.length === 12, "Conundrum level count drift");
for (const definition of dice) {
  assert(definition.default_surface_ids.length === matrixPolicy.dice_slots,
    `${definition.id}: default loadout does not fill six slots`);
  assert(new Set(definition.default_surface_ids).size === matrixPolicy.dice_slots,
    `${definition.id}: duplicate default face`);
  for (const faceId of definition.default_surface_ids)
    assert(faces.has(faceId), `${definition.id}: unknown default face ${faceId}`);
}

const runs = [];
for (let index = 0; index < matrixPolicy.baseline_runs; index++) {
  const definition = dice[index];
  runs.push(run({
    ordinal: runs.length,
    seed: matrixPolicy.baseline_seed_start + index,
    area: areas[index % areas.length],
    path: paths[index % paths.length],
    dice: definition,
    stats: 0,
    auxiliary: 0,
    prerequisite: null,
    coverage_kind: "baseline-axis",
  }));
}
for (let index = 0; index < conundrum.length; index++) {
  const level = conundrum[index];
  runs.push(run({
    ordinal: runs.length,
    seed: matrixPolicy.conundrum_seed_start + index,
    area: required(areas, matrixPolicy.conundrum_area),
    path: paths[(index + 3) % paths.length],
    dice: dice[(index + 5) % dice.length],
    stats: level.track === "Stats" ? level.level : 0,
    auxiliary: level.track === "Auxiliary" ? level.level : 0,
    prerequisite: matrixPolicy.conundrum_prerequisite,
    coverage_kind: `conundrum-${level.track.toLowerCase()}`,
  }));
}
runs.push(run({
  ordinal: runs.length,
  seed: matrixPolicy.conundrum_seed_start + conundrum.length,
  area: required(areas, matrixPolicy.conundrum_area),
  path: paths.at(-1),
  dice: dice.at(-1),
  stats: 6,
  auxiliary: 6,
  prerequisite: matrixPolicy.conundrum_prerequisite,
  coverage_kind: "conundrum-combined-cap",
}));
assert(runs.length === matrixPolicy.complete_runs, "seeded matrix run count drift");

const policyBySource = new Map(policy.policy_gaps.map((gap) => [gap.source_id, gap]));
const researchBySource = new Map(research.map((gap) =>
  [gap.policy_source_id.replace("source.goal08.project-policy.", ""), gap]));
assert(policyBySource.size === 16 && researchBySource.size === 16,
  "policy-boundary denominator drift");
assert(equal(new Set(policyBySource.keys()), new Set(researchBySource.keys())),
  "policy ownership does not match frozen research gaps");
for (const [index, gap] of policy.policy_gaps.entries()) {
  const source = researchBySource.get(gap.source_id);
  assert(source.blocking === false && source.gap_state === "PolicyBound",
    `${gap.register_id}: source policy is not nonblocking PolicyBound`);
  runs[index].policy_probes.push({
    register_id: gap.register_id,
    source_id: gap.source_id,
    owner_batches: gap.owner_batches,
    current_state: "AssignedPendingResolution",
    allowed_terminal_states: policy.policy_terminal_states,
  });
}

const coverage = {
  schema_revision: "starclock.gold-and-gears-seeded-matrix.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  generated_on: "2026-07-30",
  matrix_revision: matrixPolicy.revision,
  policy_sha256: sha256(policyPath),
  summary: {
    complete_runs: runs.length,
    formal_difficulties: uniqueValues(runs, "area_id"),
    paths: uniqueValues(runs, "path_id"),
    custom_dice: uniqueValues(runs, "custom_dice_id"),
    default_loadouts: new Set(runs.map(({ default_face_ids: ids }) => ids.join(","))).size,
    stats_levels: sortedNumbers(runs.map(({ stats_conundrum: value }) => value)),
    auxiliary_levels: sortedNumbers(runs.map(({ auxiliary_conundrum: value }) => value)),
    combined_cap_runs: runs.filter(({ stats_conundrum: stats, auxiliary_conundrum: aux }) =>
      stats === 6 && aux === 6).length,
    policy_probes: runs.reduce((sum, entry) => sum + entry.policy_probes.length, 0),
    invalid_rows: 0,
  },
  validity_contract: {
    conundrum_area: matrixPolicy.conundrum_area,
    prerequisite: matrixPolicy.conundrum_prerequisite,
    total_conundrum_cap: 12,
    unlock_profile: matrixPolicy.unlock_profile,
  },
  runs,
};
assert(coverage.summary.formal_difficulties === matrixPolicy.formal_difficulties,
  "matrix misses a formal difficulty");
assert(coverage.summary.paths === matrixPolicy.paths, "matrix misses a Path");
assert(coverage.summary.custom_dice === matrixPolicy.custom_dice,
  "matrix misses a Custom Dice");
assert(equal(coverage.summary.stats_levels, [0, 1, 2, 3, 4, 5, 6]),
  "matrix Stats Conundrum levels drift");
assert(equal(coverage.summary.auxiliary_levels, [0, 1, 2, 3, 4, 5, 6]),
  "matrix Auxiliary Conundrum levels drift");
assert(coverage.summary.combined_cap_runs === 1, "matrix combined cap run drift");
assert(coverage.summary.policy_probes === 16, "matrix policy probe count drift");
for (const entry of runs) {
  assert(entry.stats_conundrum + entry.auxiliary_conundrum <= 12,
    `${entry.id}: total Conundrum cap exceeded`);
  if (entry.stats_conundrum > 0 || entry.auxiliary_conundrum > 0)
    assert(entry.area_id === matrixPolicy.conundrum_area
      && entry.prerequisite === matrixPolicy.conundrum_prerequisite,
    `${entry.id}: invalid Conundrum entry`);
}

const outputPath = "evidence/gold-and-gears-runtime-v1/foundation/coverage-matrix.json";
writeOrCheck(outputPath, encode(coverage));
console.log(
  `Goal 14 coverage matrix ${check ? "verified" : "generated"} ` +
  `(${runs.length} valid runs; 5 difficulties; 9 Paths; 12 dice; 16 policy probes).`,
);

function run({
  ordinal,
  seed,
  area,
  path: selectedPath,
  dice: selectedDice,
  stats,
  auxiliary,
  prerequisite,
  coverage_kind: coverageKind,
}) {
  return {
    id: `G14-MATRIX-${String(ordinal + 1).padStart(2, "0")}`,
    ordinal,
    seed,
    area_id: area.id,
    difficulty: area.difficulty,
    path_id: selectedPath.id,
    custom_dice_id: selectedDice.id,
    default_face_ids: selectedDice.default_surface_ids,
    unlock_profile: matrixPolicy.unlock_profile,
    prerequisite,
    stats_conundrum: stats,
    auxiliary_conundrum: auxiliary,
    coverage_kind: coverageKind,
    expected_planes: 3,
    expected_terminal: "Complete",
    replay_verification: "FreshFactoryRequired",
    policy_probes: [],
  };
}
function required(entries, id) {
  const value = entries.find((entry) => entry.id === id);
  assert(value !== undefined, `missing ${id}`);
  return value;
}
function uniqueValues(entries, field) {
  return new Set(entries.map((entry) => entry[field])).size;
}
function sortedNumbers(values) {
  return [...new Set(values)].sort((left, right) => left - right);
}
function byId(left, right) {
  return compare(left.id, right.id);
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function equal(left, right) {
  if (left instanceof Set && right instanceof Set)
    return left.size === right.size && [...left].every((value) => right.has(value));
  return JSON.stringify(left) === JSON.stringify(right);
}
function writeOrCheck(relative, value) {
  const file = path.join(root, relative);
  if (check) {
    assert(fs.statSync(file, { throwIfNoEntry: false })?.isFile(),
      `${relative} is missing; run without --check`);
    assert(fs.readFileSync(file, "utf8") === value, `${relative} has generated drift`);
    return;
  }
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, value);
}
function encode(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}
function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function sha256(relative) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative)))
    .digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
