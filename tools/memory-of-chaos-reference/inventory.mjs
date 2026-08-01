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
const turnbasedRoot = path.join(sourceCache, "turnbasedgamedata");
const starRailRoot = path.join(sourceCache, "StarRailRes");
const output = path.join(root,
  "content-manifests/memory-of-chaos-v1/source-inventory.json");
const auditOutput = path.join(root,
  "evidence/memory-of-chaos-reference-v1/source-inventory-audit.md");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const starRailRevision = "7b349e39ee0f6f3bf814567995829b99c95e7a93";
const inherited = JSON.parse(await readFile(path.join(root,
  "content-manifests/standard-universe-v1/source-inventory.json"), "utf8"));
assert(inherited.source.revision === revision, "Goal 03 inventory revision drift");
assert(git(turnbasedRoot, ["rev-parse", "HEAD"]) === revision,
  "turnbasedgamedata revision drift");
assert(git(starRailRoot, ["rev-parse", "HEAD"]) === starRailRevision,
  "StarRailRes revision drift");

const dedicated = new Set([
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
]);
const sharedSeeds = new Set([
  "ExcelOutput/BattleEventConfig.json",
  "ExcelOutput/MapEntrance.json",
  "ExcelOutput/MapEntranceGroup.json",
  "ExcelOutput/MapEntranceUnlock.json",
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
]);
const adjacentPatterns = [
  /^ExcelOutput\/ChallengeActMark\.json$/u,
  /^ExcelOutput\/ChallengeActivityConfig\.json$/u,
  /^ExcelOutput\/ChallengeBadgeConfig\.json$/u,
  /^ExcelOutput\/ChallengeSkipConfig\.json$/u,
  /^ExcelOutput\/ChallengeBoss/u,
  /^ExcelOutput\/ChallengePeak/u,
  /^ExcelOutput\/ChallengeStory/u,
];
const treePaths = git(turnbasedRoot, ["ls-tree", "-r", "--name-only", "HEAD"])
  .split("\n").filter(Boolean);
const challengePaths = treePaths.filter((sourcePath) =>
  /^ExcelOutput\/Challenge.+\.json$/u.test(sourcePath));
const selected = new Set([...dedicated, ...sharedSeeds, ...challengePaths]);
const inheritedByPath = new Map(inherited.records.map((record) =>
  [record.path, record]));
const records = inherited.records.map((record) => ({
  repository: "turnbasedgamedata",
  ...record,
  classification: selected.has(record.path)
    ? classify(record.path)
    : "inherited-transitive-mechanic-candidate",
  selected_by: selected.has(record.path)
    ? selection(record.path)
    : "Goal 03 pinned mechanic/enemy closure retained for transitive reference audit",
  inherited_from: "content-manifests/standard-universe-v1/source-inventory.json",
}));

for (const sourcePath of [...selected].sort(compare)) {
  if (inheritedByPath.has(sourcePath)) continue;
  const bytes = await readFile(path.join(turnbasedRoot, sourcePath));
  records.push({ repository: "turnbasedgamedata", path: sourcePath,
    sha256: sha256(bytes), bytes: bytes.length,
    family: family(sourcePath), classification: classify(sourcePath),
    selected_by: selection(sourcePath) });
}
for (const sourcePath of ["info.json", "index_new/cn/characters.json",
  "index_new/en/characters.json"]) {
  const bytes = await readFile(path.join(starRailRoot, sourcePath));
  records.push({ repository: "StarRailRes", path: sourcePath,
    sha256: sha256(bytes), bytes: bytes.length,
    family: "identity-cross-check", classification: "evidence-only",
    selected_by: "pinned bilingual identity cross-check" });
}
records.sort((a, b) => compare(`${a.repository}:${a.path}`,
  `${b.repository}:${b.path}`));

const counts = {};
for (const record of records)
  counts[record.classification] = (counts[record.classification] ?? 0) + 1;
const activeStageCandidates = Array.from({ length: 12 }, (_, index) => [
  30123011 + index * 10,
  30123012 + index * 10,
]).flat().concat(30123123);
const payload = {
  schema_revision: "starclock.memory-of-chaos-source-inventory.v1",
  snapshot: { game_version: "4.4", generated_on: "2026-08-01",
    repositories: [
      { id: "turnbasedgamedata", revision },
      { id: "StarRailRes", revision: starRailRevision },
    ] },
  selector_seeds: { schedule: [201033, 201034], groups: [1033, 1034],
    ordinary_stage_rows: Array.from({ length: 12 }, (_, index) => 5201 + index),
    tierce_rows: [5213, 5313], stage_config_rows: activeStageCandidates,
    maze_buffs: [3030146, 3030147], battle_events: [30146],
    seed_is_denominator: false },
  policy: {
    active_release_required: true,
    prefix_or_adjacency_never_grants_membership: true,
    transitive_enemy_closure_frozen_in: "G17-P0-B3",
    adjacent_challenge_families: "evidence-only unless explicitly selected",
    shared_rows: "reconciliation-receipts-only",
  },
  counts: { total: records.length, by_classification: counts,
    inherited_goal03: records.filter((row) => row.inherited_from).length,
    dedicated_tables: records.filter((row) =>
      row.classification === "memory-dedicated-candidate").length,
    adjacent_exclusions: records.filter((row) =>
      row.classification === "other-challenge-evidence").length },
  records,
};
const bytes = Buffer.from(`${JSON.stringify(payload, null, 2)}\n`, "utf8");
const audit = `# Goal 17 Source Inventory Audit\n\n` +
  `- Result: passed\n- Inventory rows: ${records.length}\n` +
  `- Inherited pinned Goal 03 mechanic/enemy receipts: ` +
  `${payload.counts.inherited_goal03}\n` +
  `- Memory/Forgotten Hall dedicated table receipts: ` +
  `${payload.counts.dedicated_tables}\n` +
  `- Adjacent Challenge-family evidence/exclusions: ` +
  `${payload.counts.adjacent_exclusions}\n` +
  `- Active planning StageConfig candidates: ${activeStageCandidates.length}\n` +
  `- Classification digest: \`${sha256(canonical(counts))}\`\n` +
  `- Membership remains pending selector closure in G17-P0-B3; names, ` +
  `prefixes, schedules and adjacent IDs do not grant ownership.\n`;

if (check) {
  assert((await readFile(output)).equals(bytes), "source inventory drift");
  assert((await readFile(auditOutput, "utf8")) === audit,
    "source inventory audit drift");
  console.log(`Goal 17 source inventory verified (${records.length} rows).`);
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await mkdir(path.dirname(auditOutput), { recursive: true });
  await writeFile(output, bytes);
  await writeFile(auditOutput, audit);
  console.log(`Goal 17 source inventory generated (${records.length} rows).`);
}

function classify(sourcePath) {
  if (dedicated.has(sourcePath)) return "memory-dedicated-candidate";
  if (adjacentPatterns.some((pattern) => pattern.test(sourcePath)))
    return "other-challenge-evidence";
  if (sharedSeeds.has(sourcePath)) return "shared-reachability-candidate";
  return "challenge-adjacent-evidence";
}

function selection(sourcePath) {
  if (dedicated.has(sourcePath))
    return "Goal 17 dedicated Memory/Forgotten Hall starting oracle";
  if (adjacentPatterns.some((pattern) => pattern.test(sourcePath)))
    return "named adjacent Challenge family retained to prove exclusion";
  if (sharedSeeds.has(sourcePath))
    return "Goal 17 shared entry/stage/event/enemy/config/TextMap closure seed";
  return "Challenge-prefixed table retained for fail-closed classification";
}

function family(sourcePath) {
  if (sourcePath.startsWith("ExcelOutput/")) return "structured-table";
  if (sourcePath.startsWith("TextMap/")) return "bilingual-text-map";
  if (sourcePath.startsWith("Config/Level/")) return "level-program";
  return "ability-program";
}

function option(name) {
  const index = args.indexOf(name);
  if (index < 0) return undefined;
  assert(args[index + 1] !== undefined, `${name} requires a value`);
  return args[index + 1];
}
function git(cwd, gitArgs) {
  return execFileSync("git", ["-C", cwd, ...gitArgs], {
    encoding: "utf8", maxBuffer: 512 * 1024 * 1024,
  }).trim();
}
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") return `{${Object.keys(value)
    .sort(compare).map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
    .join(",")}}`;
  return JSON.stringify(value);
}
function sha256(value) { return createHash("sha256").update(value).digest("hex"); }
function compare(a, b) { return a < b ? -1 : a > b ? 1 : 0; }
function assert(condition, message) { if (!condition) throw new Error(message); }
function fail(message) { throw new Error(message); }
