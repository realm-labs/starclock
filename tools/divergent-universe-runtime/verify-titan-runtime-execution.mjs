#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { buildTitanRuntimeExecution } from "./generate-titan-runtime-execution.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/titan-runtime-execution.json";
const artifact = buildTitanRuntimeExecution();

assert(text(output) === pretty(artifact), "Titan runtime artifact drift");
assert(artifact.status === "TitanEntryFragmentsPartialOtherEffectsPending"
  && artifact.current_boundary.contribution_representation === "SourceDescriptorsWithRunEntryFragmentConsumer"
  && artifact.current_boundary.run_entry_fragment_consumer === "PresentRequiresNativeVerification"
  && artifact.current_boundary.run_entry_fragment_gain === "30"
  && !artifact.current_boundary.all_activity_effect_consumers_implemented
  && !artifact.current_boundary.battle_effect_consumers_implemented
  && !artifact.current_boundary.public_offer_admission_implemented
  && artifact.current_boundary.terminal_coverage_credit === 0,
"partial Titan entry effects must not imply complete runtime execution");
assert(artifact.profile_boundary.current_module_source_id === "6002201"
  && artifact.profile_boundary.current_tourn_mode === "Tourn3"
  && artifact.profile_boundary.titan_current_profile_reachability === "Unproven"
  && artifact.profile_boundary.admitted_current_profile_selector_proofs === 0
  && !artifact.profile_boundary.retained_source_rows_are_membership_evidence
  && !artifact.profile_boundary.caller_owned_entry_projection_proves_membership
  && !artifact.profile_boundary.current_profile_gameplay_parity_claimed
  && !artifact.profile_boundary.non_runtime_exclusion_proven,
"retained Titan facts prove neither current-profile admission nor exclusion");
assert(equal(artifact.summary, {
  titan_types: 12,
  boons: 84,
  talents: 36,
  contributions: 120,
  pending_obligations: 132,
  terminal_coverage_credit: 0,
  native_test_targets: 13,
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
