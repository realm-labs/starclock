#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const root = path.resolve(".");
const values = process.argv.slice(2);
assert(values.length === 5 && values[0] === "--completion-commit" &&
  /^[0-9a-f]{40}$/u.test(values[1]) && values[2] === "--run-id" &&
  /^\d+$/u.test(values[3]) && values[4] === "--record",
"usage: record-hosted-ci.mjs --completion-commit SHA --run-id ID --record");
const completionCommit = values[1];
const runId = values[3];
const policyPath = "policy/goal20-release-contract.json";
const policy = JSON.parse(capture("git", ["show", `${completionCommit}:${policyPath}`]));
const releaseEvidence = JSON.parse(capture(
  "git", ["show", `${completionCommit}:${policy.release_evidence}`],
));
const status = capture("git", ["show", `${completionCommit}:docs/goals/20-swarm-disaster-runtime-status.md`]);
assert(policy.schema_revision === "starclock.goal20-release-contract.v1" &&
  policy.goal_id === "swarm-disaster-runtime-v1" && policy.release_batch === "G20-P8-B4",
"completion commit does not contain the Goal 20 release policy");
assert(releaseEvidence.schema_revision === "starclock.goal20-release-evidence.v1" &&
  releaseEvidence.goal_id === policy.goal_id && releaseEvidence.result === "complete" &&
  releaseEvidence.completion_commit === "this file's containing G20-P8-B4 commit",
"completion commit does not contain the frozen Goal 20 release evidence");
assert(status.includes("| `G20-P8-B4` | `Complete` |") &&
  status.includes("| State | `ReleasePendingSnapshot` |"),
"completion commit is not the pre-snapshot Goal 20 terminal ledger");
assert(capture("git", ["cat-file", "-t", completionCommit]) === "commit" &&
  /^[0-9a-f]{40}$/u.test(capture("git", ["show", "-s", "--format=%T", completionCommit])),
"Goal 20 completion Git identity is unavailable");

const snapshots = json("policy/release-snapshots.json");
assert(!snapshots.goals.some(({ goal_id: goalId }) => goalId === policy.goal_id),
  "Goal 20 snapshot is already registered");
const run = JSON.parse(capture("gh", ["run", "view", runId, "--json",
  "attempt,conclusion,event,headSha,jobs,status,url,workflowName"]));
assert(run.headSha === completionCommit && run.status === "completed" &&
  run.conclusion === "success" && ["push", "workflow_dispatch"].includes(run.event),
"hosted CI run is not a successful Goal 20 completion-commit run");
const expectedProfiles = [...policy.native_profiles, ...policy.compile_only_profiles];
for (const profile of expectedProfiles) {
  const prefix = policy.native_profiles.includes(profile) ? "Native" : "Compile only";
  const job = run.jobs.find(({ name }) => name === `${prefix} / ${profile}`);
  assert(job?.conclusion === "success", `${profile}: hosted CI job did not succeed`);
}

const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "starclock-g20-ci-"));
try {
  execFileSync("gh", ["run", "download", runId, "--dir", temporary], {
    cwd: root,
    stdio: "inherit",
  });
  const receipts = walk(temporary).flatMap((file) => {
    if (!file.endsWith(".json")) return [];
    const bytes = fs.readFileSync(file);
    const evidence = JSON.parse(bytes.toString("utf8"));
    if (evidence.schema_revision !== "starclock.ci-evidence.v2") return [];
    return [{
      profile: evidence.profile,
      execution_mode: evidence.execution_mode,
      tests_executed: evidence.tests_executed,
      evidence_origin: evidence.evidence_origin,
      commit: evidence.revision.commit,
      workflow_run_id: String(evidence.revision.workflow_run_id),
      workflow_run_attempt: String(evidence.revision.workflow_run_attempt),
      runner: evidence.runner,
      rust_host: evidence.toolchain.rust_host,
      target: evidence.target,
      artifact_sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
    }];
  }).sort((left, right) => left.profile.localeCompare(right.profile));
  assert(JSON.stringify(receipts.map(({ profile }) => profile).sort()) ===
    JSON.stringify([...expectedProfiles].sort()),
  "hosted CI artifacts are missing or duplicated");
  assert(receipts.every((receipt) => receipt.evidence_origin === "hosted-ci" &&
    receipt.commit === completionCommit && receipt.workflow_run_id === runId),
  "hosted CI artifact revision drift");
  for (const receipt of receipts) {
    const native = policy.native_profiles.includes(receipt.profile);
    assert(receipt.execution_mode === (native ? "native" : "compile-only") &&
      receipt.tests_executed === native,
    `${receipt.profile}: execution claim differs from the release contract`);
  }
  const report = {
    schema_revision: "starclock.goal20-hosted-native-ci.v1",
    goal_id: policy.goal_id,
    result: "pass",
    completion_commit: completionCommit,
    completion_tree: capture("git", ["show", "-s", "--format=%T", completionCommit]),
    run: {
      id: runId,
      attempt: String(run.attempt),
      workflow: run.workflowName,
      event: run.event,
      status: run.status,
      conclusion: run.conclusion,
      url: run.url,
    },
    profiles: receipts,
  };
  const output = path.join(root, policy.hosted_native_evidence);
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, `${JSON.stringify(report, null, 2)}\n`);
  console.log(`Recorded Goal 20 hosted CI run ${runId} for ${receipts.length} profiles.`);
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

function walk(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    return entry.isDirectory() ? walk(target) : [target];
  });
}
function capture(command, args) {
  return execFileSync(command, args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  }).trim();
}
function json(relative) { return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8")); }
function assert(condition, message) { if (!condition) throw new Error(message); }
