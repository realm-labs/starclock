#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("content-manifests/unknowable-domain-v1/foundation.json");

assert(policy.schema_revision === "starclock.unknowable-domain-foundation.v1",
  "unsupported Goal 10 foundation revision");
assert(policy.goal_id === "unknowable-domain-reference-v1",
  "Goal 10 identity drift");
assert(policy.planned_phases === 5 && policy.fixed_batches === 28,
  "Goal 10 execution denominator drift");
runGit(root, ["cat-file", "-e", `${policy.launch_commit}^{commit}`]);
assert(captureGit(root, ["show", "-s", "--format=%T", policy.launch_commit]).trim()
  === policy.launch_tree, "Goal 10 launch tree drift");

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
  assert(!symbolicBranch(repository),
    `source cache must be detached ${source.cache_path}`);
  runGit(repository, ["fsck", "--connectivity-only"]);
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
const magicRows = inventory.records.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueMagic[^/]*\.json$/u.test(sourcePath));
assert(magicRows.length === policy.inherited_reference.rogue_magic_seed_rows,
  "RogueMagic seed row count drift");
assert(magicRows.every(({ family }) =>
  family === policy.inherited_reference.rogue_magic_seed_family),
"RogueMagic seed classification drift");
for (const row of magicRows) {
  const sourcePath = path.join(
    root,
    ".cache/content-reference/turnbasedgamedata",
    row.path,
  );
  assert(fs.statSync(sourcePath, { throwIfNoEntry: false })?.isFile(),
    `RogueMagic seed file is missing ${row.path}`);
  assert(matchesInventoryBytes(sourcePath, row),
    `RogueMagic seed byte/hash drift ${row.path}`);
}

const entries = policy.source_entry_contract;
assert(magicRows.length === entries.known_mode_tables,
  "known RogueMagic table count differs from inherited seed");
for (const relative of [...entries.text_maps, entries.stage_table,
  ...entries.required_config_entries])
  assert(fileExistsInTurnbasedCache(relative),
    `required Goal 10 source entry is missing ${relative}`);
assert(entries.seed_is_denominator === false
  && entries.membership_rule
    === "explicit-selector-transitive-reference-or-inherited-stable-id-closure",
"Goal 10 reachability contract drift");

verifyGoal08Checkpoint(policy.goal08_checkpoint);
verifyGoal09Checkpoint(policy.goal09_checkpoint);

const soraPolicy = json(policy.sora_contract.policy_path);
assert(sha256(policy.sora_contract.policy_path) === policy.sora_contract.policy_sha256,
  "Sora toolchain policy drift");
assert(soraPolicy.schema_revision === "starclock.sora-toolchain.v1"
  && soraPolicy.version === policy.sora_contract.required_version,
"Goal 10 Sora authority drift");
const systemSora = spawnSync("sora", ["--version"], { encoding: "utf8" });
if (systemSora.status === 0) {
  assert(systemSora.stdout.trim() ===
    `sora ${policy.sora_contract.system_path_observation}`,
  "recorded host PATH Sora observation drift");
}
assert(policy.sora_contract.system_path_is_authority === false
  && policy.sora_contract.pinned_local_install_is_authority === true,
"Goal 10 Sora resolution contract drift");

const boundary = policy.parallel_boundary;
assert(captureGit(root, ["branch", "--show-current"]).trim() === boundary.branch,
  "Goal 10 branch isolation is missing");
assert(captureGit(root, ["rev-parse", "--abbrev-ref",
  "--symbolic-full-name", "@{upstream}"]).trim() === boundary.upstream,
"Goal 10 upstream isolation is missing");
for (const owned of boundary.owned_roots)
  for (const protectedRoot of boundary.protected_roots)
    assert(!overlaps(owned, protectedRoot),
      `owned and protected roots overlap: ${owned} / ${protectedRoot}`);
const changed = [
  ...lines(captureGit(root, ["diff", "--name-only", "HEAD"])),
  ...lines(captureGit(root, ["ls-files", "--others", "--exclude-standard"])),
];
const allowed = [...boundary.owned_roots, ...boundary.owned_goal_documents];
for (const changedPath of changed) {
  assert(allowed.some((entry) => changedPath === entry
    || changedPath.startsWith(entry)),
  `Goal 10 modified a path outside its ownership boundary ${changedPath}`);
  assert(!boundary.protected_roots.some((prefix) => changedPath.startsWith(prefix)),
    `Goal 10 modified protected path ${changedPath}`);
}

const status = text("docs/goals/10-unknowable-domain-reference-data-status.md");
assert((status.match(/^\| `G10-P[0-4]-B\d+` \|/gmu) ?? []).length
  === policy.fixed_batches, "Goal 10 fixed batch ledger drift");
assert(status.includes("| State | `InProgress` |"), "Goal 10 is not active");
assert(/\| `G10-P0-B1` \| `(?:InProgress|Complete)` \|/u.test(status),
  "G10-P0-B1 has not started");
for (const document of policy.documents)
  assert(fileExists(document), `Goal 10 document is missing ${document}`);
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 10 foundation contains an unaccepted contract");
assert(policy.authoring_contract.authoritative_format === "xlsx"
  && policy.authoring_contract.editor === "python-openpyxl"
  && policy.authoring_contract.exporter === "sora-cli-0.3.0"
  && policy.authoring_contract.workbooks.length === 3,
"Goal 10 authoring contract drift");

console.log(
  "Goal 10 foundation verified (Goal 03 snapshot; 32 RogueMagic seed rows; " +
  "Goal 08 optional local and Goal 09 remote checkpoints; 28 batches; " +
  "isolated Candidate lane).",
);

function verifyGoal08Checkpoint(checkpoint) {
  assert(checkpoint.required_for_foundation === false
    && checkpoint.remote_reachable === false,
  "Goal 08 optional-checkpoint contract drift");
  if (!gitObjectExists(checkpoint.commit)) return;
  assert(captureGit(root, ["show", "-s", "--format=%T", checkpoint.commit]).trim()
    === checkpoint.tree, "Goal 08 checkpoint tree drift");
  assert(gitBlobSha256(checkpoint.commit, checkpoint.source_inventory_path)
    === checkpoint.source_inventory_sha256,
  "Goal 08 source inventory checkpoint drift");
  const bytes = gitBlob(checkpoint.commit, checkpoint.content_manifest_path);
  assert(hashBytes(bytes) === checkpoint.content_manifest_sha256,
    "Goal 08 content manifest checkpoint drift");
  const manifest = JSON.parse(bytes.toString("utf8"));
  assert(manifest.schema_revision === checkpoint.manifest_schema_revision,
    "Goal 08 manifest schema drift");
  assert(manifest.counts.records === checkpoint.records
    && manifest.counts.ownership.GoldAndGears === checkpoint.gold_owned
    && manifest.counts.ownership.Shared === checkpoint.shared,
  "Goal 08 ownership checkpoint drift");
}

function verifyGoal09Checkpoint(checkpoint) {
  assert(checkpoint.required_for_foundation === true,
    "Goal 09 checkpoint must be required");
  runGit(root, ["cat-file", "-e", `${checkpoint.commit}^{commit}`]);
  assert(captureGit(root, ["show", "-s", "--format=%T", checkpoint.commit]).trim()
    === checkpoint.tree, "Goal 09 checkpoint tree drift");
  runGit(root, ["merge-base", "--is-ancestor", checkpoint.commit,
    `${checkpoint.remote}/${checkpoint.branch}`]);
  const bytes = gitBlob(checkpoint.commit, checkpoint.source_inventory_path);
  assert(hashBytes(bytes) === checkpoint.source_inventory_sha256,
    "Goal 09 source inventory checkpoint drift");
  const sourceInventory = JSON.parse(bytes.toString("utf8"));
  assert(sourceInventory.schema_revision === checkpoint.schema_revision,
    "Goal 09 source inventory schema drift");
  assert(sourceInventory.counts.total === checkpoint.records
    && sourceInventory.counts.by_repository.turnbasedgamedata
      === checkpoint.turnbasedgamedata_records
    && sourceInventory.counts.by_repository.starrailres
      === checkpoint.starrailres_records,
  "Goal 09 source inventory counts drift");
}

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function fileExists(relative) {
  return fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isFile();
}
function fileExistsInTurnbasedCache(relative) {
  return fs.statSync(path.join(root, ".cache/content-reference/turnbasedgamedata",
    relative), { throwIfNoEntry: false })?.isFile();
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
    maxBuffer: 16 * 1024 * 1024,
  });
}
function gitBlobSha256(commit, relative) {
  return hashBytes(gitBlob(commit, relative));
}
function gitObjectExists(commit) {
  return spawnSync("git", ["cat-file", "-e", `${commit}^{commit}`], {
    cwd: root,
  }).status === 0;
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
