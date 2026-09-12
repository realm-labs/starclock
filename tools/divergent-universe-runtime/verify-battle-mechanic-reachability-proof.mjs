#!/usr/bin/env node

import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildBattleMechanicReachabilityProof } from "./generate-battle-mechanic-reachability-proof.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const proof = buildBattleMechanicReachabilityProof();
execFileSync("node", [
  "tools/divergent-universe-runtime/generate-battle-mechanic-reachability-proof.mjs",
  "--source-cache", ".cache/content-reference/turnbasedgamedata", "--check",
], { cwd: root, stdio: "inherit" });
assert(proof.status === "CompleteProvenNonRuntimeSourceLibraries"
  && proof.programs.every(({ disposition }) => disposition === "ExcludedWithProof")
  && proof.layouts.every(({ disposition }) => disposition === "MetadataOnlyAudited"),
"battle mechanic non-runtime dispositions drift");
assert(equal(proof.summary, {
  source_programs: 3,
  layout_companions: 3,
  ability_identities: 76,
  in_pair_reference_lines: 579,
  outside_pair_reference_lines: 0,
  standalone_numeric_reference_lines: 0,
  runtime_programs_admitted: 0,
}), "battle mechanic reachability summary drift");
console.log(
  "Divergent Universe battle mechanic reachability verified "
    + "(3 source libraries; 3 layouts; 76 identities; zero external references).",
);

function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }
