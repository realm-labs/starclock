#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const relative = "content-manifests/currency-wars-runtime-v1/shared-capability-audit.json";
execFileSync("node", [
  "tools/currency-wars-runtime/generate-shared-capability-audit.mjs", "--check",
], { cwd: root, stdio: "inherit" });
execFileSync("node", ["tools/repository-check/verify-native-handlers.mjs"], {
  cwd: root, stdio: "inherit",
});

const audit = json(relative);
const inventory = json("content-manifests/currency-wars-runtime-v1/capability-inventory.json");
const mechanics = json("content-manifests/currency-wars-runtime-v1/mechanic-dispositions.json");
const partitions = json("content-manifests/currency-wars-runtime-v1/mechanic-partitions.json");
assert(audit.batch === "G21-P2-B5" && audit.status === "Complete",
  "shared capability audit is not complete");
assert(inventory.missing_capabilities.length === 1
  && inventory.missing_capabilities[0].capability
    === "shared.version-4.4-postfix-opcode-semantics",
"unexpected capability gap survived P2");
const policy = audit.configuration_program_policy;
assert(policy.state === "VersionedProjectPolicyExecutable"
  && policy.confidence === "PolicyOnlyNotObservedParity"
  && policy.affected_expression_shape_ids.length
    === audit.summary.unresolved_expression_shapes
  && policy.affected_mechanic_ids.length === audit.summary.affected_mechanic_programs,
"configuration-program policy coverage drift");
assert(policy.replacement_trigger.includes("Verification fails")
  && policy.replacement_condition.includes("all ten"),
"configuration-program replacement trigger is missing");
assert(policy.executable_policy_partitions.length === 9
  && policy.executable_policy_partitions.at(-1).batch === "G21-P6-M09"
  && policy.executable_policy_partitions.at(-1).policy_programs === 42
  && policy.executable_policy_partitions.at(-1).metadata_programs === 22,
"G21-P6-M09 shared configuration-program closure is missing");

for (const probe of audit.capability_probes) {
  const source = fs.readFileSync(absolute(probe.file), "utf8");
  for (const fragment of probe.required_fragments)
    assert(source.includes(fragment), `${probe.id} lost probe fragment: ${fragment}`);
}
for (const directory of audit.content_id_branch_audit.roots) {
  for (const file of recursiveRustFiles(absolute(directory))) {
    const source = fs.readFileSync(file, "utf8");
    for (const symbol of audit.content_id_branch_audit.forbidden_mode_symbols)
      assert(!source.includes(symbol), `${path.relative(root, file)} names Currency Wars in shared core`);
    for (const token of ["PostfixExpr", "OpCodes", "DynamicHashes"])
      assert(!source.includes(token), `${path.relative(root, file)} interprets raw postfix configuration`);
  }
}
assert(audit.native_handler_audit.admitted_battle_handlers === 0
  && audit.native_handler_audit.admitted_activity_handlers === 0
  && audit.native_handler_audit.mechanic_static_handler_references === 0
  && mechanics.programs.every(({ static_handler: value }) => value === null),
"native handler was admitted without a reviewed need");
assert(audit.partition_freeze.state === "FrozenPendingExecution"
  && audit.partition_freeze.partition_count === 43
  && audit.partition_freeze.program_count === 2_367
  && audit.partition_freeze.partition_set_sha256 === partitions.freeze.partition_set_sha256,
"generated mechanic partitions are not frozen");
assert(audit.excluded_source_closure.count === 17
  && new Set(audit.excluded_source_closure.obligation_ids).size === 17,
"P2-B5 evidence-only source closure drift");

console.log(
  `Currency Wars shared capability audit verified (${audit.summary.probes} probes; `
    + `${audit.summary.audited_shared_rust_files} shared Rust files; `
    + `${audit.summary.frozen_partitions} partitions; zero handlers).`,
);

function recursiveRustFiles(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) return recursiveRustFiles(target);
    return entry.isFile() && entry.name.endsWith(".rs") ? [target] : [];
  });
}
function json(file) { return JSON.parse(fs.readFileSync(absolute(file), "utf8")); }
function absolute(file) { return path.join(root, file); }
function assert(condition, message) { if (!condition) throw new Error(message); }
