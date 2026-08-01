#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const sourceCache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? fail("--source-cache is required"));
const root = path.resolve(".");
const manifestRoot = path.join(root, "content-manifests/memory-of-chaos-v1");
const evidenceRoot = path.join(root, "evidence/memory-of-chaos-reference-v1");
const output = path.join(manifestRoot, "foundation.json");
const auditOutput = path.join(evidenceRoot, "foundation-audit.md");
const launchCommit = "92febad080dd4cf9997718d64b3648fc198ab1f8";
const sourceSpecs = [
  {
    id: "turnbasedgamedata",
    remote: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    tree: "2df8981c1bea512e21c8c900920c63002b381056",
  },
  {
    id: "StarRailRes",
    remote: "https://github.com/Mar-7th/StarRailRes.git",
    revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
    tree: "1e6892227905e0dad002bb117d63464d7a5640a6",
  },
];
const requiredTurnbased = [
  "ExcelOutput/ChallengeGeneralConfig.json",
  "ExcelOutput/ChallengeGroupConfig.json",
  "ExcelOutput/ChallengeMazeConfig.json",
  "ExcelOutput/ChallengeMazeGroupExtra.json",
  "ExcelOutput/ChallengeMazeRewardLine.json",
  "ExcelOutput/ChallengeMazeTierce.json",
  "ExcelOutput/ChallengeTargetConfig.json",
  "ExcelOutput/ConstValueChallengeClient.json",
  "ExcelOutput/ConstValueChallengeCommon.json",
  "ExcelOutput/ScheduleDataChallengeMaze.json",
  "ExcelOutput/BattleEventConfig.json",
  "ExcelOutput/MapEntrance.json",
  "ExcelOutput/MappingInfo.json",
  "ExcelOutput/MazeBuff.json",
  "ExcelOutput/StageConfig.json",
  "ExcelOutput/MonsterConfig.json",
  "ExcelOutput/MonsterTemplateConfig.json",
  "ExcelOutput/MonsterSkillConfig.json",
  "ExcelOutput/MonsterStatusConfig.json",
  "Config/ConfigAbility/BattleEventAbility_2.json",
  "Config/ConfigAbility/Level/Level_MazeChallengeBuff_Ability.json",
  "Config/ConfigAbility/StageBattleEventAbility.json",
  "Config/Level/StageCommonTemplate.json",
  "TextMap/TextMapCHS.json",
  "TextMap/TextMapEN.json",
];
const ownershipPaths = [
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
  "content-manifests/gold-and-gears-v1/content-manifest.json",
  "content-manifests/swarm-disaster-v1/content-manifest.json",
  "content-manifests/unknowable-domain-v1/content-manifest.json",
  "content-manifests/divergent-universe-v1/content-manifest.json",
  "content-manifests/currency-wars-v1/content-manifest.json",
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
  "content-manifests/galactic-baseballer-v1/content-manifest.json",
];
const ownedRoots = [
  "content-manifests/memory-of-chaos-v1/",
  "content-reference/memory-of-chaos-v1/",
  "config/memory-of-chaos/",
  "config/memory-of-chaos-generated/",
  "tools/memory-of-chaos-reference/",
  "evidence/memory-of-chaos-reference-v1/",
];

assert(git(root, ["branch", "--show-current"]) ===
  "codex/goal17-memory-of-chaos-reference", "wrong execution branch");
assert(git(root, ["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"]) ===
  "origin/codex/goal17-memory-of-chaos-reference", "wrong upstream branch");
assert(git(root, ["merge-base", launchCommit, "HEAD"]) === launchCommit,
  "execution branch no longer descends from the frozen launch commit");
const worktrees = git(root, ["worktree", "list", "--porcelain"]);
assert((worktrees.match(/branch refs\/heads\/codex\/goal17-memory-of-chaos-reference/gu)
  ?? []).length === 1, "Goal 17 branch is not isolated to one worktree");

const releaseSnapshots = JSON.parse(await readFile(
  path.join(root, "policy/release-snapshots.json"), "utf8"));
const goal03 = releaseSnapshots.goals.find(({ goal_id: id }) =>
  id === "standard-universe-reference-v1");
assert(goal03?.completion_commit ===
  "60ca52ed98c5c83d867d33bff7f88c69e0b389de", "Goal 03 snapshot drift");
assert(git(root, ["show", "-s", "--format=%T", goal03.completion_commit]) ===
  goal03.completion_tree, "Goal 03 completion tree drift");
const goal03Status = await readFile(path.join(root,
  "docs/goals/03-standard-universe-reference-data-status.md"), "utf8");
assert(goal03Status.includes("| State | `Complete` |"), "Goal 03 is not Complete");

const repositories = [];
for (const spec of sourceSpecs) {
  const repository = path.join(sourceCache, spec.id);
  assert(git(repository, ["rev-parse", "HEAD"]) === spec.revision,
    `${spec.id} revision drift`);
  assert(git(repository, ["show", "-s", "--format=%T", "HEAD"]) === spec.tree,
    `${spec.id} tree drift`);
  assert(git(repository, ["status", "--porcelain"]) === "",
    `${spec.id} cache is dirty`);
  assert(symbolicBranch(repository) === "", `${spec.id} is not detached`);
  assert(git(repository, ["remote", "get-url", "origin"]) === spec.remote,
    `${spec.id} origin drift`);
  execFileSync("git", ["-C", repository, "cat-file", "-e", "HEAD^{commit}"]);
  repositories.push({ ...spec, clean: true, detached: true,
    commit_readable: true, connectivity_check: "passed during G17-P0-B1" });
}

const sourceFiles = [];
for (const relative of requiredTurnbased) {
  const bytes = await readFile(path.join(sourceCache, "turnbasedgamedata", relative));
  sourceFiles.push({ repository: "turnbasedgamedata", path: relative,
    bytes: bytes.length, sha256: sha256(bytes) });
}
for (const relative of ["info.json", "index_new/cn/characters.json",
  "index_new/en/characters.json"]) {
  const bytes = await readFile(path.join(sourceCache, "StarRailRes", relative));
  sourceFiles.push({ repository: "StarRailRes", path: relative,
    bytes: bytes.length, sha256: sha256(bytes) });
}
sourceFiles.sort((a, b) => compare(`${a.repository}:${a.path}`,
  `${b.repository}:${b.path}`));

const ownership = [];
for (const relative of ownershipPaths) {
  const bytes = await readFile(path.join(root, relative));
  ownership.push({ path: relative, revision: launchCommit, sha256: sha256(bytes) });
}

const payload = {
  schema_revision: "starclock.memory-of-chaos-foundation.v1",
  goal_id: "memory-of-chaos-reference-v1",
  batch: "G17-P0-B1",
  snapshot: { game_version: "4.4", structured_access_date: "2026-07-22",
    reproduced_on: "2026-08-01", launch_commit: launchCommit,
    launch_tree: git(root, ["show", "-s", "--format=%T", launchCommit]) },
  publication: { remote: "origin", branch:
    "codex/goal17-memory-of-chaos-reference", base: launchCommit,
    base_equals_origin_master_at_launch: true,
    remote_equality_command: "git ls-remote --exit-code origin refs/heads/codex/goal17-memory-of-chaos-reference" },
  source_cache: { path: sourceCache, isolated: true, repositories,
    required_files: sourceFiles, required_files_sha256: sha256(canonical(sourceFiles)) },
  goal03: { state: "Complete", ...goal03 },
  ownership_checkpoints: ownership,
  isolation: { branch: "codex/goal17-memory-of-chaos-reference",
    worktree: root, owned_roots: ownedRoots,
    protected_concurrent_roots: ["content-manifests/pure-fiction-v1/",
      "content-manifests/apocalyptic-shadow-v1/",
      "content-reference/pure-fiction-v1/",
      "content-reference/apocalyptic-shadow-v1/"],
    shared_challenge_policy: "reconciliation-receipts-only-no-cross-goal-overwrite" },
  scope: { lane: "Experimental-to-Candidate", runtime_implementation: false,
    active_selector_candidates: { schedule: 201033, group: 1033,
      ordinary_rows: "5201-5212", tierce: 5213, maze_buff: 3030146,
      objectives: [251, 252, 253], battle_event: 30146 },
    candidates_are_denominator: false, scheduled_group_1034_enabled: false },
  authoring: { format: "xlsx", editor: "python-openpyxl",
    authority: "sora-cli-0.3.0", workbooks: ["MemoryOfChaos.xlsx",
      "MemoryOfChaosBindings.xlsx", "MemoryOfChaosReview.xlsx"] },
  execution: { phases: 5, batches: 25, one_in_progress: true,
    push_each_batch: true, runtime_forbidden: true },
};
const bytes = Buffer.from(`${JSON.stringify(payload, null, 2)}\n`, "utf8");
const audit = `# Goal 17 Foundation Audit\n\n` +
  `- Result: passed\n- Snapshot: Version 4.4\n` +
  `- Launch/base commit: \`${launchCommit}\`\n` +
  `- Isolated source cache: \`${sourceCache}\`\n` +
  `- Source repositories: clean, detached, origin-verified, commit-readable; ` +
  `connectivity checks completed during G17-P0-B1\n` +
  `- Required source receipts: ${sourceFiles.length}\n` +
  `- Ownership checkpoints: ${ownership.length} committed Goal 07-13/16 artifacts\n` +
  `- Branch/worktree: dedicated and remote-backed\n` +
  `- Writable roots: six Goal 17 isolated roots; shared Challenge rows are ` +
  `reconciliation-only\n- Runtime implementation: forbidden\n`;

if (check) {
  assert((await readFile(output)).equals(bytes), "foundation.json drift");
  assert((await readFile(auditOutput, "utf8")) === audit,
    "foundation audit drift");
  console.log(`Goal 17 foundation verified (${sourceFiles.length} source receipts).`);
} else {
  await mkdir(manifestRoot, { recursive: true });
  await mkdir(evidenceRoot, { recursive: true });
  await writeFile(output, bytes);
  await writeFile(auditOutput, audit);
  console.log(`Goal 17 foundation generated (${sourceFiles.length} source receipts).`);
}

function option(name) {
  const index = args.indexOf(name);
  if (index < 0) return undefined;
  assert(args[index + 1] !== undefined, `${name} requires a value`);
  return args[index + 1];
}

function git(cwd, gitArgs) {
  return execFileSync("git", ["-C", cwd, ...gitArgs], {
    encoding: "utf8", maxBuffer: 128 * 1024 * 1024,
  }).trim();
}

function symbolicBranch(cwd) {
  try { return git(cwd, ["symbolic-ref", "--quiet", "--short", "HEAD"]); }
  catch { return ""; }
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") return `{${Object.keys(value)
    .sort(compare).map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
    .join(",")}}`;
  return JSON.stringify(value);
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function compare(a, b) { return a < b ? -1 : a > b ? 1 : 0; }
function assert(condition, message) { if (!condition) throw new Error(message); }
function fail(message) { throw new Error(message); }
