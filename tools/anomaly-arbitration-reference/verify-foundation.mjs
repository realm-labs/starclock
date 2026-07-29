#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceCache = sourceCacheArgument(process.argv.slice(2));
const policy = json("content-manifests/anomaly-arbitration-v1/foundation.json");

assert(policy.schema_revision === "starclock.anomaly-arbitration-foundation.v1",
  "unsupported Goal 13 foundation revision");
assert(policy.goal_id === "anomaly-arbitration-reference-v1",
  "Goal 13 identity drift");
assert(policy.batch === "G13-P0-B1"
  && policy.planned_phases === 5
  && policy.fixed_batches === 25,
"Goal 13 execution denominator drift");

verifyCommit(policy.launch_commit, policy.launch_tree, "Goal 13 launch");
runGit(root, ["merge-base", "--is-ancestor", policy.launch_commit,
  `${policy.launch_remote.remote}/${policy.launch_remote.branch}`]);
assert(policy.launch_remote.verified_commit === policy.launch_commit,
  "Goal 13 launch publication identity drift");

verifyGoal03();
verifySources();
verifyOwnershipCheckpoints();
verifySoraAuthority();
verifyParallelBoundary();
verifyGoalContract();

console.log(
  "Goal 13 foundation verified (Goal 03 snapshot; 6 ChallengePeak tables; " +
  "21 hashed source entries; Goal 07-12 checkpoints; 25 isolated batches).",
);

function verifyGoal03() {
  const snapshots = json("policy/release-snapshots.json");
  const snapshot = snapshots.goals.find(({ goal_id: goalId }) =>
    goalId === policy.required_snapshot.goal_id);
  assert(snapshot !== undefined, "Goal 03 immutable snapshot is missing");
  assert(snapshot.completion_commit
    === policy.required_snapshot.completion_commit,
  "Goal 03 completion commit drift");
  assert(snapshot.completion_tree
    === policy.required_snapshot.completion_tree,
  "Goal 03 completion tree drift");
  verifyCommit(snapshot.completion_commit, snapshot.completion_tree,
    "Goal 03 completion");
  assert(sha256(snapshot.release_policy_path)
    === policy.required_snapshot.release_policy_sha256,
  "Goal 03 release policy drift");
  assert(sha256(snapshot.release_evidence_path)
    === policy.required_snapshot.release_evidence_sha256,
  "Goal 03 release evidence drift");

  const evidence = json(snapshot.release_evidence_path);
  assert(evidence.snapshot.game_version === policy.source_snapshot.game_version
    && evidence.snapshot.access_date === policy.source_snapshot.access_date,
  "Goal 03 source snapshot drift");
  assert(evidence.digests.source_manifest_sha256
    === policy.inherited_reference.source_manifest_sha256,
  "Goal 03 source manifest evidence drift");
  assert(evidence.digests.universe_staging_bundle_sha256
    === policy.inherited_reference.universe_staging_bundle_sha256,
  "Goal 03 universe staging bundle drift");
  assert(evidence.digests.preserved_core_runtime_bundle_sha256
    === policy.inherited_reference.preserved_core_runtime_bundle_sha256,
  "Goal 03 preserved runtime bundle drift");

  for (const [relative, expected] of [
    [policy.inherited_reference.source_inventory_path,
      policy.inherited_reference.source_inventory_sha256],
    [policy.inherited_reference.source_manifest_path,
      policy.inherited_reference.source_manifest_sha256],
    [policy.inherited_reference.normalized_pack_index_path,
      policy.inherited_reference.normalized_pack_index_sha256],
  ])
    assert(sha256(relative) === expected,
      `inherited Goal 03 artifact drift ${relative}`);
}

function verifySources() {
  for (const source of policy.source_snapshot.repositories) {
    const repository = path.join(sourceCache, source.cache_name);
    assert(isDirectory(repository),
      `source cache is missing ${source.cache_name}`);
    assert(captureGit(repository, ["rev-parse", "HEAD"]).trim()
      === source.revision, `source revision drift ${source.cache_name}`);
    assert(captureGit(repository, ["show", "-s", "--format=%T", "HEAD"]).trim()
      === source.tree, `source tree drift ${source.cache_name}`);
    assert(captureGit(repository, ["remote", "get-url", "origin"]).trim()
      === source.remote, `source remote drift ${source.cache_name}`);
    assert(captureGit(repository, ["status", "--porcelain"]).trim() === "",
      `source cache has local changes ${source.cache_name}`);
    assert(!symbolicBranch(repository),
      `source cache must be detached ${source.cache_name}`);
    runGit(repository, ["cat-file", "-e", `${source.revision}^{commit}`]);
    runGit(repository, ["fsck", "--connectivity-only", "--no-dangling"]);
  }

  const entries = policy.required_source_entries;
  const turnRows = verifyRequiredRows(
    path.join(sourceCache, "turnbasedgamedata"),
    entries.turnbased_files,
    "turnbasedgamedata",
  );
  const resRows = verifyRequiredRows(
    path.join(sourceCache, "StarRailRes"),
    entries.starrailres_files,
    "StarRailRes",
  );
  assert(turnRows.length === entries.turnbased_rows,
    "turnbasedgamedata required-row count drift");
  assert(resRows.length === entries.starrailres_rows,
    "StarRailRes required-row count drift");
  assert(canonicalRowsDigest(turnRows) === entries.turnbased_rows_sha256,
    "turnbasedgamedata required-row digest drift");
  assert(canonicalRowsDigest(resRows) === entries.starrailres_rows_sha256,
    "StarRailRes required-row digest drift");

  const contract = policy.source_entry_contract;
  assert(contract.dedicated_tables.length === 6,
    "ChallengePeak dedicated-table seed drift");
  assert(contract.dedicated_tables.every((relative) =>
    entries.turnbased_files.some(({ path: sourcePath }) =>
      sourcePath === relative)),
  "ChallengePeak dedicated table lacks a required-file receipt");
  assert(contract.membership_rule
    === "explicit-active-version-selector-transitive-reference-or-inherited-stable-id-closure",
  "Goal 13 membership rule drift");
  assert(contract.seed_is_denominator === false,
    "ChallengePeak seed was promoted to a denominator");
  assert(contract.planning_candidates.admission_state
    === "planning-only-until-generated-selector-closure",
  "planning candidates were prematurely admitted");
  assert(contract.zero_pool_rule.includes("selector-closure"),
    "zero-pool proof contract drift");
}

function verifyOwnershipCheckpoints() {
  assert(policy.ownership_checkpoints.map(({ goal }) => goal).join(",")
    === "G07,G08,G09,G10,G11,G12",
  "Goal 07-12 checkpoint set drift");
  for (const checkpoint of policy.ownership_checkpoints) {
    verifyCommit(checkpoint.commit, checkpoint.tree,
      `${checkpoint.goal} checkpoint`);
    if (checkpoint.remote_reachable)
      verifyRemoteReachability(checkpoint);
    for (const file of checkpoint.files) {
      const bytes = gitBlob(checkpoint.commit, file.path);
      assert(hashBytes(bytes) === file.sha256,
        `${checkpoint.goal} checkpoint file drift ${file.path}`);
      const record = JSON.parse(bytes.toString("utf8"));
      assert(record.schema_revision === file.schema_revision,
        `${checkpoint.goal} checkpoint schema drift ${file.path}`);
    }
  }

  const g07 = policy.ownership_checkpoints[0];
  const retained = JSON.parse(gitBlob(g07.commit, g07.files[0].path));
  assert(retained.summary.records.total === g07.records
    && retained.summary.rules.total === g07.rules
    && retained.summary.fixtures.total === g07.fixtures,
  "Goal 07 checkpoint counts drift");

  for (const checkpoint of policy.ownership_checkpoints.slice(1, 4)) {
    const manifestFile = checkpoint.files.find(({ path: sourcePath }) =>
      sourcePath.endsWith("/content-manifest.json"));
    assert(manifestFile !== undefined,
      `${checkpoint.goal} ownership manifest is missing`);
    const manifest = JSON.parse(gitBlob(checkpoint.commit, manifestFile.path));
    assert(manifest.counts.records === checkpoint.records
      && manifest.counts.ownership[checkpoint.mode] === checkpoint.mode_owned
      && manifest.counts.ownership.Shared === checkpoint.shared,
    `${checkpoint.goal} ownership counts drift`);
  }

  const g11 = policy.ownership_checkpoints[4];
  const inventory = JSON.parse(gitBlob(g11.commit, g11.files[0].path));
  assert(inventory.counts.total === g11.inventory_records
    && g11.committed_ownership_manifest_available === false,
  "Goal 11 checkpoint boundary drift");
  assert(policy.ownership_checkpoints[5]
    .committed_ownership_manifest_available === false,
  "Goal 12 checkpoint boundary drift");
}

function verifyRemoteReachability(checkpoint) {
  const recorded = spawnSync("git", [
    "-C",
    root,
    "merge-base",
    "--is-ancestor",
    checkpoint.commit,
    `${checkpoint.remote}/${checkpoint.branch}`,
  ]);
  if (recorded.status === 0) return;

  const remoteRefs = lines(captureGit(root, [
    "for-each-ref",
    "--contains",
    checkpoint.commit,
    "--format=%(refname)",
    `refs/remotes/${checkpoint.remote}`,
  ]));
  assert(remoteRefs.length > 0,
    `${checkpoint.goal} checkpoint is not reachable from a remote ref`);
}

function verifySoraAuthority() {
  const soraPolicy = json(policy.sora_contract.policy_path);
  assert(sha256(policy.sora_contract.policy_path)
    === policy.sora_contract.policy_sha256,
  "Sora toolchain policy drift");
  assert(soraPolicy.schema_revision === "starclock.sora-toolchain.v1"
    && soraPolicy.version === policy.sora_contract.required_version,
  "Goal 13 Sora authority drift");
  const systemSora = spawnSync("sora", ["--version"], { encoding: "utf8" });
  if (systemSora.status === 0)
    assert(systemSora.stdout.trim()
      === `sora ${policy.sora_contract.system_path_observation}`,
    "recorded host PATH Sora observation drift");
  assert(policy.sora_contract.system_path_is_authority === false
    && policy.sora_contract.pinned_local_install_is_authority === true,
  "Goal 13 Sora resolution contract drift");
}

function verifyParallelBoundary() {
  const boundary = policy.parallel_boundary;
  assert(captureGit(root, ["branch", "--show-current"]).trim()
    === boundary.branch, "Goal 13 branch isolation is missing");
  assert(captureGit(root, ["rev-parse", "--abbrev-ref",
    "--symbolic-full-name", "@{upstream}"]).trim() === boundary.upstream,
  "Goal 13 upstream isolation is missing");

  const worktrees = captureGit(root, ["worktree", "list", "--porcelain"]);
  const branchMarker = `branch refs/heads/${boundary.branch}`;
  assert(worktrees.split(/\r?\n/u).filter((line) =>
    line === branchMarker).length === 1,
  "Goal 13 branch is not isolated to one worktree");

  for (const owned of boundary.owned_roots)
    for (const protectedRoot of boundary.protected_roots)
      assert(!overlaps(owned, protectedRoot),
        `owned and protected roots overlap ${owned} / ${protectedRoot}`);

  const changed = [
    ...lines(captureGit(root, ["diff", "--name-only", "HEAD"])),
    ...lines(captureGit(root, ["ls-files", "--others", "--exclude-standard"])),
  ];
  const allowed = [...boundary.owned_roots, ...boundary.owned_goal_documents];
  for (const changedPath of changed) {
    assert(allowed.some((entry) => changedPath === entry
      || changedPath.startsWith(entry)),
    `Goal 13 modified a path outside its boundary ${changedPath}`);
    assert(!boundary.protected_roots.some((prefix) =>
      changedPath.startsWith(prefix)),
    `Goal 13 modified protected path ${changedPath}`);
  }
}

function verifyGoalContract() {
  const status = text("docs/goals/13-anomaly-arbitration-reference-data-status.md");
  assert((status.match(/^\| `G13-P[0-4]-B\d+` \|/gmu) ?? []).length
    === policy.fixed_batches, "Goal 13 fixed batch ledger drift");
  assert(status.includes("| State | `InProgress` |"),
    "Goal 13 is not active");
  assert(/\| `G13-P0-B1` \| `(?:InProgress|Complete)` \|/u.test(status),
    "G13-P0-B1 has not started");
  for (const document of policy.documents)
    assert(isFile(document), `Goal 13 document is missing ${document}`);
  assert(Object.values(policy.contracts).every((value) => value === true),
    "Goal 13 foundation contains an unaccepted contract");
  assert(policy.authoring_contract.authoritative_format === "xlsx"
    && policy.authoring_contract.editor === "python-openpyxl"
    && policy.authoring_contract.exporter === "sora-cli-0.3.0"
    && policy.authoring_contract.workbooks.length === 3,
  "Goal 13 authoring contract drift");
}

function verifyRequiredRows(repository, required, label) {
  return required.map((row) => {
    const absolute = path.join(repository, row.path);
    assert(isFile(absolute), `${label} required file is missing ${row.path}`);
    const bytes = fs.readFileSync(absolute);
    assert(bytes.length === row.bytes,
      `${label} byte count drift ${row.path}`);
    assert(hashBytes(bytes) === row.sha256,
      `${label} SHA-256 drift ${row.path}`);
    return { path: row.path, bytes: row.bytes, sha256: row.sha256 };
  });
}

function canonicalRowsDigest(rows) {
  const ordered = [...rows].sort((left, right) =>
    left.path < right.path ? -1 : left.path > right.path ? 1 : 0);
  return hashBytes(Buffer.from(`${JSON.stringify(ordered)}\n`, "utf8"));
}

function sourceCacheArgument(values) {
  const index = values.indexOf("--source-cache");
  assert(index !== -1 && values[index + 1] !== undefined,
    "--source-cache requires an isolated cache path");
  assert(values.length === 2,
    "unsupported Goal 13 foundation arguments");
  return path.resolve(values[index + 1]);
}

function verifyCommit(commit, tree, label) {
  runGit(root, ["cat-file", "-e", `${commit}^{commit}`]);
  assert(captureGit(root, ["show", "-s", "--format=%T", commit]).trim()
    === tree, `${label} tree drift`);
}

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function json(relative) {
  return JSON.parse(text(relative));
}

function isFile(relativeOrAbsolute) {
  const absolute = path.isAbsolute(relativeOrAbsolute)
    ? relativeOrAbsolute
    : path.join(root, relativeOrAbsolute);
  return fs.statSync(absolute, { throwIfNoEntry: false })?.isFile() ?? false;
}

function isDirectory(absolute) {
  return fs.statSync(absolute, { throwIfNoEntry: false })?.isDirectory() ?? false;
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
    maxBuffer: 128 * 1024 * 1024,
  });
}

function runGit(cwd, gitArgs) {
  execFileSync("git", gitArgs, { cwd, stdio: "ignore" });
}

function captureGit(cwd, gitArgs) {
  return execFileSync("git", gitArgs, {
    cwd,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}

function symbolicBranch(cwd) {
  try {
    return captureGit(cwd, ["symbolic-ref", "--quiet", "--short", "HEAD"])
      .trim();
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
