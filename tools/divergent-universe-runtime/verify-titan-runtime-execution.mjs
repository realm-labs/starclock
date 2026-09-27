#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { buildTitanRuntimeExecution } from "./generate-titan-runtime-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/titan-runtime-execution.json";
const artifact = buildTitanRuntimeExecution();

assert(text(output) === pretty(artifact), "Titan runtime artifact drift");
assert(artifact.status === "TitanSelectionDescriptorsEffectsPending"
  && artifact.current_boundary.contribution_representation === "SourceDescriptorsOnly"
  && !artifact.current_boundary.activity_effect_consumers_implemented
  && !artifact.current_boundary.battle_effect_consumers_implemented
  && !artifact.current_boundary.public_offer_admission_implemented
  && artifact.current_boundary.terminal_coverage_credit === 0,
"Titan descriptors must not imply executed effects");
assert(equal(artifact.summary, {
  titan_types: 12,
  boons: 84,
  talents: 36,
  contributions: 120,
  pending_obligations: 132,
  terminal_coverage_credit: 0,
  native_test_targets: 6,
}), "Titan runtime summary drift");
assert(artifact.pending_assignments.obligation_ids.length === 132
  && new Set(artifact.pending_assignments.obligation_ids).size === 132,
"Titan exact-once pending obligations drift");
assert(!Object.hasOwn(artifact, "execution_receipt")
  && !Object.hasOwn(artifact, "capability_probes")
  && artifact.native_test_targets.every((target) => !Object.hasOwn(target, "result")),
"test-target existence cannot emit a pass receipt");

console.log(
  "Titan current descriptor inventory verified (132 pending obligations; no runtime execution credit). Run behavioral checks directly with Cargo.",
);

function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
