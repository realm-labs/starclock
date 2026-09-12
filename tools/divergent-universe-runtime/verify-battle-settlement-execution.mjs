#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildBattleSettlementExecution } from "./generate-battle-settlement-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/battle-settlement-execution.json";
const artifact = buildBattleSettlementExecution();
run("node", ["tools/divergent-universe-runtime/generate-battle-settlement-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "battle settlement artifact drift");
assert(artifact.batch === "G22-P6-B5"
  && artifact.status === "CompleteAtomicBattleResultSettlementCarryProgressionAndTransition",
"battle settlement status drift");
assert(equal(artifact.summary, {
  carry_fields: 4,
  terminal_obligations: 0,
  mechanic_programs: 0,
  probes: 3,
}), "battle settlement summary drift");
assert(equal(artifact.assignment_closure, {
  obligations: 0,
  fixture_families: 0,
  research_gaps: 0,
  policy_sources: 0,
  mechanic_programs: 0,
}) && Object.values(artifact.execution_receipt).every((value) => value === "Passed")
  && artifact.policy_boundary.settlement_rng_draws === 0
  && !artifact.policy_boundary.observed_reward_parity_claimed,
"battle settlement closure or policy drift");
console.log(
  "Divergent Universe battle settlement verified "
    + "(atomic carry/progression/transition; terminal defeat; no invented rewards).",
);

function run(command, args) { execFileSync(command, args, { cwd: root, stdio: "inherit" }); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
