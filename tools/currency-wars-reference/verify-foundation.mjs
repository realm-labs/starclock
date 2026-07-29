#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceCache = sourceCacheArgument(process.argv.slice(2));
const policy = json("content-manifests/currency-wars-v1/foundation.json");

assert(policy.schema_revision === "starclock.currency-wars-foundation.v1",
  "unsupported Goal 12 foundation revision");
assert(policy.goal_id === "currency-wars-reference-v1",
  "Goal 12 identity drift");
assert(policy.planned_phases === 5 && policy.fixed_batches === 29,
  "Goal 12 execution denominator drift");
runGit(root, ["cat-file", "-e", `${policy.launch_commit}^{commit}`]);
assert(captureGit(root, ["show", "-s", "--format=%T", policy.launch_commit]).trim()
  === policy.launch_tree, "Goal 12 launch tree drift");
runGit(root, ["merge-base", "--is-ancestor", policy.launch_commit,
  `${policy.launch_remote.remote}/${policy.launch_remote.branch}`]);
assert(policy.launch_remote.verified_commit === policy.launch_commit,
  "Goal 12 launch publication identity drift");

const snapshots = json("policy/release-snapshots.json");
const snapshot = snapshots.goals.find(({ goal_id: goalId }) =>
  goalId === policy.required_snapshot.goal_id);
assert(snapshot !== undefined, "Goal 03 immutable snapshot is missing");
for (const field of ["completion_commit", "completion_tree"])
  assert(snapshot[field] === policy.required_snapshot[field],
    `Goal 03 ${field} drift`);
runGit(root, ["cat-file", "-e", `${snapshot.completion_commit}^{commit}`]);
assert(captureGit(root, ["show", "-s", "--format=%T",
  snapshot.completion_commit]).trim() === snapshot.completion_tree,
"Goal 03 completion tree drift");
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

const repositories = new Map();
for (const source of policy.source_snapshot.repositories) {
  const repository = path.join(sourceCache, source.cache_name);
  repositories.set(source.cache_name, repository);
  assert(fs.statSync(path.join(repository, ".git"), { throwIfNoEntry: false }),
    `source cache is missing ${source.cache_name}`);
  assert(captureGit(repository, ["rev-parse", "HEAD"]).trim() === source.revision,
    `source revision drift for ${source.cache_name}`);
  assert(captureGit(repository, ["show", "-s", "--format=%T", "HEAD"]).trim()
    === source.tree, `source tree drift for ${source.cache_name}`);
  assert(captureGit(repository, ["remote", "get-url", "origin"]).trim()
    === source.remote, `source remote drift for ${source.cache_name}`);
  assert(captureGit(repository, ["status", "--porcelain"]).trim() === "",
    `source cache has local changes ${source.cache_name}`);
  assert(!symbolicBranch(repository),
    `source cache must be detached ${source.cache_name}`);
  runGit(repository, ["cat-file", "-e", `${source.revision}^{commit}`]);
  runGit(repository, ["fsck", "--connectivity-only", "--no-dangling"]);
}

const turnRepository = repositories.get("turnbasedgamedata");
const resRepository = repositories.get("StarRailRes");
assert(turnRepository !== undefined && resRepository !== undefined,
  "required source repository mapping is missing");
for (const entry of policy.required_source_entries.turnbased_files)
  verifyGitEntry(turnRepository,
    policy.source_snapshot.repositories[0].revision, entry);
for (const entry of policy.required_source_entries.starrailres_files)
  verifyGitEntry(resRepository,
    policy.source_snapshot.repositories[1].revision, entry);

const familyRows = lines(captureGit(turnRepository, [
  "ls-tree", "-r", "--name-only",
  policy.source_snapshot.repositories[0].revision, "--", "ExcelOutput",
])).filter((entry) => /^ExcelOutput\/Rogue(?:Persona|Tourn)[^/]*\.json$/u
  .test(entry)).sort().map((entry) => {
  const bytes = gitBlob(turnRepository,
    policy.source_snapshot.repositories[0].revision, entry);
  return { path: entry, bytes: bytes.length, sha256: hashBytes(bytes) };
});
const requiredFamily = policy.required_source_entries.turnbased_family;
assert(familyRows.filter(({ path: entry }) =>
  entry.includes("/RoguePersona")).length === requiredFamily.persona_files,
"RoguePersona seed file count drift");
assert(familyRows.filter(({ path: entry }) =>
  entry.includes("/RogueTourn")).length === requiredFamily.tourn_files,
"RogueTourn seed file count drift");
assert(hashBytes(Buffer.from(`${JSON.stringify(familyRows)}\n`))
  === requiredFamily.canonical_rows_sha256,
"Persona/Tourn canonical source-entry digest drift");

const inherited = policy.inherited_reference;
assert(sha256(inherited.source_inventory_path)
  === inherited.source_inventory_sha256,
"inherited Standard source inventory drift");
assert(sha256("content-manifests/standard-universe-v1/content-manifest.json")
  === inherited.source_manifest_sha256,
"inherited Standard content manifest drift");
assert(sha256("content-reference/standard-universe-v1/pack-index.json")
  === inherited.normalized_pack_index_sha256,
"inherited Standard pack index drift");
const inventory = json(inherited.source_inventory_path);
const tournRows = inventory.records.filter(({ path: sourcePath }) =>
  /^ExcelOutput\/RogueTourn[^/]*\.json$/u.test(sourcePath));
assert(tournRows.length === inherited.rogue_tourn_seed_rows,
  "inherited RogueTourn row count drift");
assert(tournRows.every(({ family }) => family === inherited.rogue_tourn_seed_family),
  "inherited RogueTourn classification drift");
for (const row of tournRows) {
  const bytes = gitBlob(turnRepository,
    policy.source_snapshot.repositories[0].revision, row.path);
  assert(matchesInventoryBytes(bytes, row, inherited.source_inventory_encoding),
    `inherited RogueTourn byte/hash drift ${row.path}`);
}

verifySelectors(turnRepository,
  policy.source_snapshot.repositories[0].revision,
  policy.source_entry_contract.selectors);
for (const checkpoint of policy.ownership_checkpoints)
  verifyOwnershipCheckpoint(checkpoint);
verifyGoal11Checkpoint(policy.goal11_checkpoint);

const soraPolicy = json(policy.sora_contract.policy_path);
assert(sha256(policy.sora_contract.policy_path) === policy.sora_contract.policy_sha256,
  "Sora toolchain policy drift");
assert(soraPolicy.schema_revision === "starclock.sora-toolchain.v1"
  && soraPolicy.version === policy.sora_contract.required_version,
"Goal 12 Sora authority drift");
const systemSora = spawnSync("sora", ["--version"], { encoding: "utf8" });
if (systemSora.status === 0)
  assert(systemSora.stdout.trim()
    === `sora ${policy.sora_contract.system_path_observation}`,
  "recorded host PATH Sora observation drift");
assert(policy.sora_contract.system_path_is_authority === false
  && policy.sora_contract.pinned_local_install_is_authority === true,
"Goal 12 Sora resolution contract drift");

const boundary = policy.parallel_boundary;
assert(captureGit(root, ["branch", "--show-current"]).trim() === boundary.branch,
  "Goal 12 branch isolation is missing");
assert(captureGit(root, ["rev-parse", "--abbrev-ref",
  "--symbolic-full-name", "@{upstream}"]).trim() === boundary.upstream,
"Goal 12 upstream isolation is missing");
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
  `Goal 12 modified a path outside its ownership boundary ${changedPath}`);
  assert(!boundary.protected_roots.some((prefix) =>
    changedPath.startsWith(prefix)),
  `Goal 12 modified protected path ${changedPath}`);
}

const status = text("docs/goals/12-currency-wars-reference-data-status.md");
assert((status.match(/^\| `G12-P[0-4]-B\d+` \|/gmu) ?? []).length
  === policy.fixed_batches, "Goal 12 fixed batch ledger drift");
assert(status.includes("| State | `InProgress` |"), "Goal 12 is not active");
assert(/\| `G12-P0-B1` \| `(?:InProgress|Complete)` \|/u.test(status),
  "G12-P0-B1 has not started");
for (const document of policy.documents)
  assert(fileExists(document), `Goal 12 document is missing ${document}`);
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 12 foundation contains an unaccepted contract");
assert(policy.authoring_contract.authoritative_format === "xlsx"
  && policy.authoring_contract.editor === "python-openpyxl"
  && policy.authoring_contract.exporter === "sora-cli-0.3.0"
  && policy.authoring_contract.workbooks.length === 3,
"Goal 12 authoring contract drift");

console.log(
  "Goal 12 foundation verified (Goal 03 snapshot; 11 Persona and 64 Tourn " +
  "seed files; Goal 08/09/10 committed ownership checkpoints; Goal 11 " +
  "setup boundary; 29 batches; isolated Candidate lane).",
);

function sourceCacheArgument(values) {
  const index = values.indexOf("--source-cache");
  if (index === -1)
    return path.resolve(root, process.env.STARCLOCK_SOURCE_CACHE
      ?? ".cache/content-reference");
  assert(values[index + 1] !== undefined, "--source-cache requires a path");
  assert(values.length === 2, "unsupported Goal 12 foundation arguments");
  return path.resolve(values[index + 1]);
}
function verifyGitEntry(repository, revision, entry) {
  const bytes = gitBlob(repository, revision, entry.path);
  assert(bytes.length === entry.bytes, `source byte count drift ${entry.path}`);
  assert(hashBytes(bytes) === entry.sha256, `source hash drift ${entry.path}`);
}
function verifySelectors(repository, revision, selectors) {
  const resident = gitJson(repository, revision,
    "ExcelOutput/RogueActivityResidentConfig.json");
  assert(resident.some((row) => row.ActivityID === selectors.activity_id
    && row.SubMode === selectors.sub_mode
    && row.ActivityModuleID === selectors.activity_module_id),
  "Currency Wars activity selector drift");
  const modules = gitJson(repository, revision, "ExcelOutput/RogueTournModule.json");
  assert(modules.some((row) => row.MainTournID === selectors.main_tourn_id
    && row.SubTournID === selectors.sub_tourn_id
    && row.ActivityModuleID === selectors.activity_module_id),
  "Currency Wars module selector drift");
  const areaGroups = gitJson(repository, revision,
    "ExcelOutput/RogueTournAreaGroupByTourn.json");
  assert(areaGroups.some((row) => Object.values(row).includes(selectors.tourn_mode)),
    "Currency Wars Tourn3 selector drift");
}
function verifyOwnershipCheckpoint(checkpoint) {
  runGit(root, ["cat-file", "-e", `${checkpoint.commit}^{commit}`]);
  assert(captureGit(root, ["show", "-s", "--format=%T", checkpoint.commit]).trim()
    === checkpoint.tree, `${checkpoint.goal} checkpoint tree drift`);
  if (checkpoint.remote_reachable)
    runGit(root, ["merge-base", "--is-ancestor", checkpoint.commit,
      `${checkpoint.remote}/${checkpoint.branch}`]);
  const inventoryBytes = gitBlob(root, checkpoint.commit,
    checkpoint.source_inventory_path);
  assert(hashBytes(inventoryBytes) === checkpoint.source_inventory_sha256,
    `${checkpoint.goal} source inventory checkpoint drift`);
  const manifestBytes = gitBlob(root, checkpoint.commit,
    checkpoint.content_manifest_path);
  assert(hashBytes(manifestBytes) === checkpoint.content_manifest_sha256,
    `${checkpoint.goal} content manifest checkpoint drift`);
  const manifest = JSON.parse(manifestBytes.toString("utf8"));
  assert(manifest.schema_revision === checkpoint.manifest_schema_revision,
    `${checkpoint.goal} manifest schema drift`);
  assert(manifest.counts.records === checkpoint.records
    && manifest.counts.ownership[checkpoint.mode] === checkpoint.mode_owned
    && manifest.counts.ownership.Shared === checkpoint.shared,
  `${checkpoint.goal} ownership checkpoint drift`);
}
function verifyGoal11Checkpoint(checkpoint) {
  runGit(root, ["cat-file", "-e", `${checkpoint.commit}^{commit}`]);
  assert(captureGit(root, ["show", "-s", "--format=%T", checkpoint.commit]).trim()
    === checkpoint.tree, "Goal 11 checkpoint tree drift");
  runGit(root, ["merge-base", "--is-ancestor", checkpoint.commit,
    `${checkpoint.remote}/${checkpoint.branch}`]);
  assert(checkpoint.remote_reachable
    && checkpoint.committed_ownership_manifest_available === false,
  "Goal 11 setup-only boundary drift");
  const probe = spawnSync("git", ["cat-file", "-e",
    `${checkpoint.commit}:content-manifests/divergent-universe-v1/content-manifest.json`],
  { cwd: root });
  assert(probe.status !== 0,
    "Goal 11 checkpoint unexpectedly contains an ownership manifest");
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
function sha256(relative) {
  return hashBytes(fs.readFileSync(path.join(root, relative)));
}
function hashBytes(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}
function gitBlob(repository, commit, relative) {
  return execFileSync("git", ["show", `${commit}:${relative}`], {
    cwd: repository,
    maxBuffer: 128 * 1024 * 1024,
  });
}
function gitJson(repository, commit, relative) {
  return JSON.parse(gitBlob(repository, commit, relative).toString("utf8"));
}
function matchesInventoryBytes(bytes, row, encodingPolicy) {
  if (bytes.length === row.bytes && hashBytes(bytes) === row.sha256) return true;
  if (encodingPolicy !== "accept-exact-or-lf-to-crlf-equivalent") return false;
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
