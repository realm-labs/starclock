#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
assert(process.argv.slice(2).every((value) => value === "--check"),
  "usage: generate-coverage-matrix.mjs [--check]");
const policyPath = "policy/goal20-coverage-and-release.json";
const policy = json(policyPath);
assert(policy.schema_revision === "starclock.goal20-coverage-and-release.v1",
  "unsupported Goal 20 coverage policy");
for (const input of Object.values(policy.inputs))
  assert(sha256(input.path) === input.sha256, `coverage input drift: ${input.path}`);

const areas = json(policy.inputs.areas.path)
  .filter(({ area_kind: kind }) => kind === "Formal").sort(byId);
const paths = json(policy.inputs.audience_paths.path).sort(byId);
const dice = new Map(json(policy.inputs.audience_dice.path)
  .map((definition) => [definition.id, definition]));
const faces = new Set(json(policy.inputs.dice_faces.path).map(({ id }) => id));
const dispositions = json(policy.inputs.runtime_dispositions.path);
const matrixPolicy = policy.coverage_matrix;
assert(areas.length === matrixPolicy.formal_difficulties, "difficulty count drift");
assert(paths.length === matrixPolicy.audience_paths, "Audience Path count drift");
assert(dice.size === matrixPolicy.audience_dice, "Audience Die count drift");
for (const selectedPath of paths) {
  const selectedDie = dice.get(selectedPath.audience_die_id);
  assert(selectedDie !== undefined, `${selectedPath.id}: paired die missing`);
  assert(selectedDie.path_id === selectedPath.path_id, `${selectedPath.id}: path/die mismatch`);
  assert(selectedDie.face_ids.length > 0, `${selectedDie.id}: empty face set`);
  for (const faceId of selectedDie.face_ids)
    assert(faces.has(faceId), `${selectedDie.id}: unknown face ${faceId}`);
}

const runs = [];
for (let index = 0; index < matrixPolicy.baseline_runs; index++)
  runs.push(run(runs.length, areas[index % areas.length], paths[index],
    "baseline-axis", null));
for (let index = 0; index < matrixPolicy.boundary_runs; index++)
  runs.push(run(runs.length, areas[(index + 2) % areas.length], paths[index],
    "countdown-disarray-boundary", matrixPolicy.boundary_cases[index]));
assert(runs.length === matrixPolicy.complete_runs, "seeded matrix count drift");

const boundaries = dispositions.policy_boundaries;
assert(boundaries.length === 31, "policy boundary denominator drift");
for (const [index, boundary] of boundaries.entries()) {
  runs[index % runs.length].policy_probes.push({
    id: boundary.id,
    implementation_batches: boundary.implementation_batches,
    current_state: "AssignedPendingResolution",
    allowed_terminal_states: policy.policy_terminal_states,
  });
}
const coverage = {
  schema_revision:"starclock.swarm-disaster-seeded-matrix.v1",
  goal_id:policy.goal_id,
  batch:policy.batch,
  generated_on:"2026-08-01",
  matrix_revision:matrixPolicy.revision,
  policy_sha256:sha256(policyPath),
  summary:{
    complete_runs:runs.length,
    formal_difficulties:uniqueValues(runs,"area_id"),
    audience_paths:uniqueValues(runs,"path_id"),
    audience_dice:uniqueValues(runs,"audience_die_id"),
    reachable_die_faces:new Set(runs.flatMap(({ face_ids: ids }) => ids)).size,
    boundary_cases:runs.filter(({ boundary_case: value }) => value !== null).length,
    policy_probes:runs.reduce((sum, entry) => sum + entry.policy_probes.length, 0),
    invalid_rows:0
  },
  validity_contract:{
    countdown_initial:matrixPolicy.countdown_initial,
    disarray_levels:matrixPolicy.disarray_levels,
    unlock_profile:matrixPolicy.unlock_profile
  },
  runs
};
assert(coverage.summary.formal_difficulties === 5, "matrix misses a difficulty");
assert(coverage.summary.audience_paths === 8, "matrix misses an Audience Path");
assert(coverage.summary.audience_dice === 8, "matrix misses an Audience Die");
assert(coverage.summary.reachable_die_faces === 42, "matrix misses a die face");
assert(coverage.summary.boundary_cases === 8, "matrix boundary count drift");
assert(coverage.summary.policy_probes === 31, "matrix policy probe count drift");
const output = "evidence/swarm-disaster-runtime-v1/foundation/coverage-matrix.json";
writeOrCheck(output, encode(coverage));
console.log(`Goal 20 coverage matrix ${check ? "verified" : "generated"} ` +
  `(${runs.length} runs; 5 difficulties; 8 Paths/dice; 42 faces; 31 policies).`);

function run(ordinal, area, selectedPath, coverageKind, boundaryCase) {
  const selectedDie = dice.get(selectedPath.audience_die_id);
  return {
    id:`G20-MATRIX-${String(ordinal + 1).padStart(2, "0")}`,
    ordinal,
    seed:matrixPolicy.seed_start + ordinal,
    area_id:area.id,
    difficulty:area.difficulty,
    path_id:selectedPath.path_id,
    audience_path_id:selectedPath.id,
    audience_die_id:selectedDie.id,
    face_ids:[...selectedDie.face_ids],
    unlock_profile:matrixPolicy.unlock_profile,
    coverage_kind:coverageKind,
    boundary_case:boundaryCase,
    countdown_initial:matrixPolicy.countdown_initial,
    expected_planes:3,
    expected_terminal:"Complete",
    replay_verification:"FreshFactoryRequired",
    policy_probes:[]
  };
}
function uniqueValues(entries, field) { return new Set(entries.map((e) => e[field])).size; }
function byId(left, right) { return left.id < right.id ? -1 : left.id > right.id ? 1 : 0; }
function writeOrCheck(relative, value) {
  const file = path.join(root, relative);
  if (check) {
    assert(fs.statSync(file, { throwIfNoEntry:false })?.isFile(), `${relative} missing`);
    assert(fs.readFileSync(file,"utf8") === value, `${relative} has generated drift`);
    return;
  }
  fs.mkdirSync(path.dirname(file), { recursive:true });
  fs.writeFileSync(file,value);
}
function encode(value) { return `${JSON.stringify(value,null,2)}\n`; }
function json(relative) { return JSON.parse(fs.readFileSync(path.join(root,relative),"utf8")); }
function sha256(relative) { return crypto.createHash("sha256").update(fs.readFileSync(path.join(root,relative))).digest("hex"); }
function assert(condition,message) { if (!condition) throw new Error(message); }
