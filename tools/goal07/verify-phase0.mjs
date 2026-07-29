import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
run("node", ["tools/goal07/verify-foundation.mjs"]);
run("node", ["tools/goal07/verify-retained-audit.mjs"]);
run("node", ["tools/goal07/verify-partitions.mjs"]);

const approximation = json("policy/goal07-evidence-and-approximation.json");
assert(
  approximation.project_policy_resolution.expected_records === 52,
  "external-decision denominator differs",
);
assert(
  approximation.enemy_numeric_policy.expected_variants === 73,
  "numeric-approximation denominator differs",
);
assert(
  approximation.enemy_numeric_policy.forbidden_scope.length === 9,
  "mechanic fields are not excluded from numeric approximation",
);
assert(
  Object.values(approximation.project_policy_resolution.contracts)
    .every((value) => value === true),
  "external-decision contract is incomplete",
);

const dependencies = json("policy/goal07-dependency-baseline.json");
const dependencyBaselineCommit = execFileSync(
  "git",
  ["log", "-n", "1", "--format=%H", "--",
    "policy/goal07-dependency-baseline.json"],
  { cwd: root, encoding: "utf8" },
).trim();
assert(/^[0-9a-f]{40}$/u.test(dependencyBaselineCommit),
  "Goal 07 dependency baseline commit is missing");
assert(
  dependencies.cargo_lock_sha256
    === sha256Bytes(execFileSync(
      "git",
      ["show", `${dependencyBaselineCommit}:Cargo.lock`],
      { cwd: root, encoding: "buffer", maxBuffer: 64 * 1024 * 1024 },
    )),
  "Cargo.lock baseline differs",
);
assert(
  dependencies.workspace_manifest_sha256
    === sha256Bytes(execFileSync(
      "git",
      ["show", `${dependencyBaselineCommit}:Cargo.toml`],
      { cwd: root, encoding: "buffer", maxBuffer: 64 * 1024 * 1024 },
    )),
  "workspace manifest baseline differs",
);
assert(
  dependencies.new_registry_packages === 0
    && dependencies.new_direct_dependencies === 0,
  "Goal 07 Phase 0 introduced a dependency",
);
assert(
  Object.values(dependencies.contracts).every((value) => value === true),
  "dependency contract is incomplete",
);

const performance = json("policy/goal07-performance.json");
assert(
  performance.focused_wall_budget_seconds >= 60
    && performance.focused_wall_budget_seconds <= 180,
  "focused budget is outside 1–3 minutes",
);
assert(performance.workloads.length === 6, "performance workload count differs");
assert(
  new Set(performance.workloads.map((entry) => entry.id)).size
    === performance.workloads.length,
  "performance workload IDs are not unique",
);
assert(
  performance.terminal_limits.catalog_compositions_per_battle === 0,
  "per-battle catalog composition is allowed",
);

const release = json("policy/goal07-release-contract.json");
assert(
  release.schema_revision
    === "starclock.goal07-release-contract-scaffold.v1"
    && release.state === "Scaffold",
  "release scaffold identity differs",
);
assert(
  release.planned_phases === 8
    && release.planned_fixed_batches === 17
    && release.planned_generated_content_batches === 104
    && release.planned_total_batches === 121,
  "release batch denominator differs",
);
assert(
  release.required_prior_contracts.at(-1)
    === "combat-identity-dynamic-assembly-v1",
  "Goal 06 is not the direct prerequisite",
);

const register = json(
  "content-manifests/standard-universe-mechanics-complete-v1/"
    + "evidence-and-approximation-register.json",
);
const baselineSummary = json(
  "evidence/standard-universe-mechanics-complete-v1/phase0/baseline-summary.json",
);
assert(
  register.project_policy_records.length === 52
    && register.enemy_numeric_approximations.length === 73,
  "generated evidence register denominator differs",
);
assert(
  new Set(register.project_policy_records.map((entry) => entry.id)).size === 52,
  "external-decision IDs are not unique",
);
assert(
  new Set(register.enemy_numeric_approximations.map((entry) => entry.id)).size
    === 73,
  "numeric-approximation IDs are not unique",
);
assert(
  baselineSummary.schema_revision
    === "starclock.goal07-phase0-baseline-summary.v1"
    && baselineSummary.result === "complete"
    && baselineSummary.register_sha256 === sha256(
      "content-manifests/standard-universe-mechanics-complete-v1/"
        + "evidence-and-approximation-register.json",
    )
    && baselineSummary.dependency_baseline.cargo_lock_sha256
      === dependencies.cargo_lock_sha256
    && baselineSummary.release_state === "Scaffold",
  "frozen Goal 07 Phase 0 summary drift",
);

const status = text(
  "docs/goals/07-standard-universe-mechanics-completion-status.md",
);
assert(
  status.includes("| `G07-P0-B4` | `Complete` |"),
  "G07-P0-B4 is not complete",
);
const nextBatch = status.match(/^\| Next unblocked batch \| (.+) \|$/mu)?.[1];
assert(
  nextBatch === "None"
    || /^`G07-(?:P1-B[1-6]|P[2-5]-M\d+-S\d+|P[67]-B\d+)`$/u
      .test(nextBatch ?? ""),
  "next batch regressed before G07-P1-B1",
);
console.log(
  "Goal 07 Phase 0 verified "
    + "(121 batches, 52 external decisions, 73 numeric candidates, 6 workloads).",
);

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(relative) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative)))
    .digest("hex");
}
function sha256Bytes(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
