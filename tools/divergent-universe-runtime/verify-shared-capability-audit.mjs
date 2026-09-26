#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const relative =
  "content-manifests/divergent-universe-runtime-v1/shared-capability-audit.json";
execFileSync("node", [
  "tools/divergent-universe-runtime/generate-shared-capability-audit.mjs", "--check",
], { cwd: root, stdio: "inherit" });
execFileSync("node", ["tools/repository-check/verify-native-handlers.mjs"], {
  cwd: root, stdio: "inherit",
});

const audit = json(relative);
const inventory = json(
  "content-manifests/divergent-universe-runtime-v1/capability-inventory.json",
);
const mechanics = json(
  "content-manifests/divergent-universe-runtime-v1/mechanic-dispositions.json",
);
const partitions = json(
  "content-manifests/divergent-universe-runtime-v1/mechanic-partitions.json",
);
assert(audit.batch === "G22-P2-B5"
  && audit.status === "CompleteNoRuntimeExecutionCredit",
"shared capability audit is not complete");
assert(inventory.missing_capabilities.length === 42
  && inventory.missing_capabilities.every(({ capability }) =>
    capability !== "shared.version-4.4-postfix-opcode-semantics"),
"capability gap denominator drift");
assert(audit.summary.probes === 11
  && audit.summary.mode_owned_capabilities === 42
  && audit.summary.mode_owned_shapes === 63
  && audit.summary.unresolved_expression_shapes === 29
  && audit.summary.postfix_affected_mechanic_programs === 3,
"shared probe or gap assignment drift");
const policy = audit.configuration_program_policy;
assert(policy.state === "ProvenNonRuntimeUnreachableAtPinnedRevision"
  && policy.confidence === "ExactReleasedReachabilityEvidence"
  && policy.affected_expression_shape_ids.length === 29
  && policy.affected_mechanic_ids.length === 3,
"postfix configuration policy coverage drift");
assert(policy.replacement_trigger.includes("outside reference")
  && policy.replacement_condition.includes("released profile"),
"postfix policy replacement trigger is missing");

for (const probe of audit.capability_probes) {
  const source = fs.readFileSync(absolute(probe.file), "utf8");
  for (const fragment of probe.required_fragments)
    assert(source.includes(fragment), `${probe.id} lost probe fragment: ${fragment}`);
}
for (const directory of audit.content_id_branch_audit.roots) {
  for (const file of recursiveRustFiles(absolute(directory))) {
    const source = fs.readFileSync(file, "utf8");
    for (const symbol of audit.content_id_branch_audit.forbidden_mode_symbols)
      assert(!source.includes(symbol), `${path.relative(root, file)} names DU in shared core`);
    for (const token of audit.content_id_branch_audit.forbidden_raw_interpreter_tokens)
      assert(!source.includes(token), `${path.relative(root, file)} interprets raw postfix data`);
  }
}
assert(audit.content_id_branch_audit.audited_file_count === 211,
  "shared Rust audit denominator drift");
assert(audit.native_handler_audit.admitted_battle_handlers === 0
  && audit.native_handler_audit.admitted_activity_handlers === 0
  && audit.native_handler_audit.mechanic_static_handler_references === 0
  && mechanics.programs.every(({ static_handler: value }) => value === null),
"native handler was admitted without a reviewed need");
assert(audit.partition_freeze.state === "FrozenPendingExecution"
  && audit.partition_freeze.partition_count === 13
  && audit.partition_freeze.program_count === 669
  && audit.partition_freeze.partition_set_sha256
    === partitions.freeze.partition_set_sha256,
"generated mechanic partitions are not frozen");
assert(audit.partition_freeze.partitions.every(({ batch, freeze_sha256: digest }) =>
  partitions.partitions.some((partition) =>
    partition.batch === batch && partition.freeze_sha256 === digest)),
"per-partition freeze digest drift");

console.log(
  "Divergent Universe shared capability audit verified "
    + "(11 probes; 211 shared Rust files; 13 partitions; zero handlers).",
);

function recursiveRustFiles(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) return recursiveRustFiles(target);
    return entry.isFile() && entry.name.endsWith(".rs") ? [target] : [];
  });
}

function json(file) {
  return JSON.parse(fs.readFileSync(absolute(file), "utf8"));
}

function absolute(file) {
  return path.join(root, file);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
