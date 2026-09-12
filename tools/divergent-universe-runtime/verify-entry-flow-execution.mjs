#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildEntryFlowExecution } from "./generate-entry-flow-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/entry-flow-execution.json";
const artifact = buildEntryFlowExecution();

run("node", ["tools/divergent-universe-runtime/generate-entry-flow-execution.mjs", "--check"]);
assert(text(output) === pretty(artifact), "entry-flow execution artifact drift");
assert(artifact.batch === "G22-P3-B1"
  && artifact.status === "CompleteEntryFlowNoBattleCredit",
"entry-flow execution status drift");
assert(equal(artifact.summary, {
  areas: 28,
  ordinary_areas: 15,
  cyclical_areas: 13,
  layers: 11,
  concrete_offered_rooms: 0,
  obligations_terminal: 916,
  production_fixtures_passed: 4,
  executable_research_gaps: 4,
  executable_policy_sources: 8,
  probes: 3,
}), "entry-flow summary drift");
assert(artifact.room_selection_policy.selected_behavior
  === "LogicalLayerCheckpointNoCandidatePromotion"
  && artifact.room_selection_policy.retained_shared_candidates === 848
  && artifact.room_selection_policy.promoted_candidates === 0,
"room reachability policy drift");
assert(artifact.catalog_closure.finish_conditions === 13
  && artifact.catalog_closure.stage_flow_records === 111
  && artifact.catalog_closure.cyclical_challenges === 13,
"entry-flow catalog closure drift");
assert(artifact.terminal_assignments.exact_obligations === 56
  && artifact.terminal_assignments.policy_obligations === 860
  && artifact.terminal_assignments.fixture_families.length === 4
  && artifact.terminal_assignments.research_gaps.length === 4
  && artifact.terminal_assignments.policy_sources.length === 8,
"entry-flow terminal assignment drift");
assert(artifact.execution_boundary.credit.includes("no battle")
  && artifact.capability_probes.every(({ result }) => result === "Passed"),
"entry-flow execution credit boundary drift");

console.log(
  "Divergent Universe entry-flow execution verified "
    + "(28 areas; 11 layers; 916 obligations; zero promoted rooms; no battle credit).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}

function text(file) {
  return fs.readFileSync(path.join(root, file), "utf8");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
