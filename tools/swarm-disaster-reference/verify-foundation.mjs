#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/goal09-foundation.json");

assert(policy.schema_revision === "starclock.goal09-foundation.v1",
  "unsupported Goal 09 foundation revision");
assert(policy.goal_id === "swarm-disaster-reference-v1",
  "Goal 09 identity drift");
assert(policy.planned_phases === 5 && policy.fixed_batches === 29,
  "Goal 09 execution denominator drift");

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
assert(releaseEvidence.digests.source_manifest_sha256
  === policy.inherited_reference.source_manifest_sha256,
"Goal 03 source manifest drift");
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
assert(sha256("content-manifests/standard-universe-v1/content-manifest.json")
  === policy.inherited_reference.source_manifest_sha256,
"inherited Standard content manifest drift");
assert(sha256("content-reference/standard-universe-v1/pack-index.json")
  === policy.inherited_reference.normalized_pack_index_sha256,
"inherited Standard pack index drift");
const inventory = json(inventoryPath);
const dlcRows = inventory.records.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueDLC[^/]*\.json$/u.test(sourcePath));
assert(dlcRows.length === policy.inherited_reference.rogue_dlc_seed_rows,
  "RogueDLC seed row count drift");
assert(dlcRows.every(({ family }) =>
  family === policy.inherited_reference.rogue_dlc_seed_family),
"RogueDLC seed classification drift");
for (const row of dlcRows) {
  const sourcePath = path.join(
    root,
    ".cache/content-reference/turnbasedgamedata",
    row.path,
  );
  assert(fs.statSync(sourcePath, { throwIfNoEntry: false })?.isFile(),
    `RogueDLC seed file is missing ${row.path}`);
  assert(matchesInventoryBytes(sourcePath, row),
    `RogueDLC seed byte/hash drift ${row.path}`);
}

const gold = policy.goal08_checkpoint;
runGit(root, ["cat-file", "-e", `${gold.commit}^{commit}`]);
assert(captureGit(root, ["show", "-s", "--format=%T", gold.commit]).trim()
  === gold.tree, "Goal 08 checkpoint tree drift");
assert(gitBlobSha256(gold.commit, gold.source_inventory_path)
  === gold.source_inventory_sha256, "Goal 08 source inventory checkpoint drift");
const goldManifestBytes = gitBlob(gold.commit, gold.content_manifest_path);
assert(hashBytes(goldManifestBytes) === gold.content_manifest_sha256,
  "Goal 08 content manifest checkpoint drift");
const goldManifest = JSON.parse(goldManifestBytes.toString("utf8"));
assert(goldManifest.schema_revision === gold.manifest_schema_revision,
  "Goal 08 manifest schema drift");
assert(goldManifest.counts.records === gold.records
  && goldManifest.counts.ownership.GoldAndGears === gold.gold_owned
  && goldManifest.counts.ownership.Shared === gold.shared,
"Goal 08 ownership checkpoint drift");
assert(goldManifest.ownership_policy.fail_closed.includes("RogueDLC"),
  "Goal 08 shared-DLC fail-closed contract is missing");

const boundary = policy.parallel_boundary;
assert(captureGit(root, ["branch", "--show-current"]).trim()
  .startsWith(boundary.branch_prefix), "Goal 09 branch isolation is missing");
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
    `Goal 09 modified protected path ${changedPath}`);

const status = text("docs/goals/09-swarm-disaster-reference-data-status.md");
assert((status.match(/^\| `G09-P[0-4]-B\d+` \|/gmu) ?? []).length
  === policy.fixed_batches, "Goal 09 fixed batch ledger drift");
assert(status.includes("| State | `InProgress` |"), "Goal 09 is not active");
assert(/\| `G09-P0-B1` \| `(?:InProgress|Complete)` \|/u.test(status),
  "G09-P0-B1 has not started");
for (const document of policy.documents)
  assert(fileExists(document), `Goal 09 document is missing ${document}`);
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 09 foundation contains an unaccepted contract");
assert(policy.authoring_contract.authoritative_format === "xlsx"
  && policy.authoring_contract.editor === "python-openpyxl"
  && policy.authoring_contract.exporter === "sora-cli-0.3.0",
"Goal 09 authoring contract drift");

console.log(
  "Goal 09 foundation verified (Goal 03 snapshot; 32 RogueDLC seed rows; " +
  "Goal 08 commit-backed ownership checkpoint; 29 batches; isolated Candidate lane).",
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
  return hashBytes(fs.readFileSync(path.join(root, relative)));
}
function hashBytes(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}
function gitBlob(commit, relative) {
  return execFileSync("git", ["show", `${commit}:${relative}`], {
    cwd: root,
    maxBuffer: 8 * 1024 * 1024,
  });
}
function gitBlobSha256(commit, relative) {
  return hashBytes(gitBlob(commit, relative));
}
function matchesInventoryBytes(absolute, row) {
  const bytes = fs.readFileSync(absolute);
  if (bytes.length === row.bytes && hashBytes(bytes) === row.sha256) return true;
  if (policy.inherited_reference.source_inventory_encoding
    !== "accept-exact-or-lf-to-crlf-equivalent")
    return false;
  const textBytes = bytes.toString("utf8");
  if (textBytes.includes("\r\n")) return false;
  const crlfBytes = Buffer.from(textBytes.replaceAll("\n", "\r\n"), "utf8");
  return crlfBytes.length === row.bytes && hashBytes(crlfBytes) === row.sha256;
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
