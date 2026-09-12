#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildBattleAssemblyExecution } from "./generate-battle-assembly-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/battle-assembly-execution.json";
const artifact = buildBattleAssemblyExecution();
run("node", ["tools/divergent-universe-runtime/generate-battle-assembly-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "battle assembly artifact drift");
assert(artifact.batch === "G22-P6-B4"
  && artifact.status === "CompleteImmutableCurrentBattleSpecAssemblyAndConstructionValidation",
"battle assembly status drift");
assert(equal(artifact.summary, {
  identity_components: 7,
  wave_shapes: 8,
  terminal_obligations: 0,
  probes: 4,
}), "battle assembly summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 1,
  research_gaps: 1,
  policy_sources: 1,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && !artifact.policy_boundary.observed_enemy_or_encounter_parity_claimed,
"battle assembly closure or policy drift");
console.log(
  "Divergent Universe battle assembly verified "
    + "(7 identity components; 8 wave shapes; construction-validated BattleSpec).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
