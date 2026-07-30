#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal08-foundation.json");

assert(policy.schema_revision === "starclock.goal08-foundation.v1",
  "unsupported Goal 08 foundation revision");
assert(policy.goal_id === "gold-and-gears-reference-v1",
  "Goal 08 identity drift");
assert(policy.planned_phases === 5 && policy.fixed_batches === 28,
  "Goal 08 execution denominator drift");

const snapshots = json("policy/release-snapshots.json");
const snapshot = snapshots.goals.find(({ goal_id: goalId }) =>
  goalId === policy.required_snapshot.goal_id);
assert(snapshot !== undefined, "Goal 03 immutable snapshot is missing");
for (const field of ["completion_commit", "completion_tree"])
  assert(snapshot[field] === policy.required_snapshot[field],
    `Goal 03 ${field} drift`);
runGit(root, ["cat-file", "-e", `${snapshot.completion_commit}^{commit}`]);
assert(captureGit(root, ["show", "-s", "--format=%T", snapshot.completion_commit]).trim()
  === snapshot.completion_tree, "Goal 03 completion tree drift");
assert(sha256(snapshot.release_policy_path)
  === policy.required_snapshot.release_policy_sha256,
"Goal 03 release policy drift");
assert(sha256(snapshot.release_evidence_path)
  === policy.required_snapshot.release_evidence_sha256,
"Goal 03 release evidence drift");

const releaseEvidence = json(snapshot.release_evidence_path);
assert(releaseEvidence.snapshot.game_version === policy.source_snapshot.game_version
  && releaseEvidence.snapshot.access_date === policy.source_snapshot.access_date,
"Goal 03 structured-source boundary drift");
assert(releaseEvidence.digests.universe_staging_bundle_sha256
  === policy.inherited_reference.universe_staging_bundle_sha256,
"Goal 03 staging bundle drift");
assert(releaseEvidence.digests.preserved_core_runtime_bundle_sha256
  === policy.inherited_reference.preserved_core_runtime_bundle_sha256,
"Goal 03 preserved runtime bundle drift");

for (const source of policy.source_snapshot.repositories) {
  const repository = path.join(root, source.cache_path);
  assert(fs.statSync(path.join(repository, ".git"), { throwIfNoEntry: false }),
    `source cache is missing ${source.cache_path}`);
  assert(captureGit(repository, ["rev-parse", "HEAD"]).trim() === source.revision,
    `source revision drift for ${source.cache_path}`);
  assert(captureGit(repository, ["remote", "get-url", "origin"]).trim()
    === source.remote, `source remote drift for ${source.cache_path}`);
  assert(captureGit(repository, ["status", "--porcelain"]).trim() === "",
    `source cache has local changes ${source.cache_path}`);
  assert(!symbolicBranch(repository), `source cache must be detached ${source.cache_path}`);
}

const inventoryPath = policy.inherited_reference.source_inventory_path;
assert(sha256(inventoryPath) === policy.inherited_reference.source_inventory_sha256,
  "inherited Standard source inventory drift");
const inventory = json(inventoryPath);
const nousRows = inventory.records.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueNous[^/]*\.json$/u.test(sourcePath));
assert(nousRows.length === policy.inherited_reference.rogue_nous_seed_rows,
  "RogueNous seed row count drift");
assert(nousRows.every(({ family }) =>
  family === policy.inherited_reference.rogue_nous_seed_family),
"RogueNous seed classification drift");
for (const row of nousRows) {
  const sourcePath = path.join(
    root,
    ".cache/content-reference/turnbasedgamedata",
    row.path,
  );
  assert(fs.statSync(sourcePath, { throwIfNoEntry: false })?.isFile(),
    `RogueNous seed file is missing ${row.path}`);
  assert(matchesInventoryBytes(sourcePath, row),
    `RogueNous seed byte/hash drift ${row.path}`);
}

const boundary = policy.parallel_boundary;
assert(captureGit(root, ["branch", "--show-current"]).trim()
  .startsWith(boundary.branch_prefix), "Goal 08 branch isolation is missing");
for (const owned of boundary.owned_roots)
  for (const protectedRoot of boundary.protected_roots)
    assert(!overlaps(owned, protectedRoot),
      `owned and protected roots overlap: ${owned} / ${protectedRoot}`);
const changed = [
  ...lines(captureGit(root, ["diff", "--name-only", "HEAD"])),
  ...lines(captureGit(root, ["ls-files", "--others", "--exclude-standard"])),
];
for (const changedPath of changed)
  assert(!boundary.protected_roots.some((prefix) => changedPath.startsWith(prefix)),
    `Goal 08 modified protected path ${changedPath}`);

const status = text("docs/goals/08-gold-and-gears-reference-data-status.md");
assert((status.match(/^\| `G08-P[0-4]-B\d+` \|/gmu) ?? []).length
  === policy.fixed_batches, "Goal 08 fixed batch ledger drift");
assert(status.includes("| State | `InProgress` |"), "Goal 08 is not active");
assert(/\| `G08-P0-B1` \| `(?:InProgress|Complete)` \|/u.test(status),
  "G08-P0-B1 has not started");
for (const document of policy.documents)
  assert(fileExists(document), `Goal 08 document is missing ${document}`);
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 08 foundation contains an unaccepted contract");
assert(policy.authoring_contract.authoritative_format === "xlsx"
  && policy.authoring_contract.editor === "python-openpyxl"
  && policy.authoring_contract.exporter === "sora-cli-0.3.0",
"Goal 08 authoring contract drift");

console.log(
  "Goal 08 foundation verified (Goal 03 snapshot; 21 RogueNous seed rows; " +
  "28 batches; isolated Candidate reference lane).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function fileExists(relative) {
  return fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile();
}
function sha256(relative) {
  return hashFile(path.join(root, relative));
}
function hashFile(absolute) {
  return crypto.createHash("sha256").update(fs.readFileSync(absolute)).digest("hex");
}
function matchesInventoryBytes(absolute, row) {
  const bytes = fs.readFileSync(absolute);
  if (bytes.length === row.bytes
    && crypto.createHash("sha256").update(bytes).digest("hex") === row.sha256)
    return true;
  if (policy.inherited_reference.source_inventory_encoding
    !== "accept-exact-or-lf-to-crlf-equivalent")
    return false;
  const textBytes = bytes.toString("utf8");
  if (textBytes.includes("\r\n")) return false;
  const crlfBytes = Buffer.from(textBytes.replaceAll("\n", "\r\n"), "utf8");
  return crlfBytes.length === row.bytes
    && crypto.createHash("sha256").update(crlfBytes).digest("hex") === row.sha256;
}
function runGit(cwd, args) {
  execFileSync("git", args, { cwd, stdio: "ignore" });
}
function captureGit(cwd, args) {
  return execFileSync("git", args, { cwd, encoding: "utf8" });
}
function symbolicBranch(cwd) {
  try {
    return captureGit(cwd, ["symbolic-ref", "--quiet", "--short", "HEAD"]).trim();
  } catch {
    return "";
  }
}
function lines(value) {
  return value.split(/\r?\n/u).filter(Boolean);
}
function overlaps(left, right) {
  return left.startsWith(right) || right.startsWith(left);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
