#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceCache = argument("--source-cache");
const check = args.includes("--check");
const output = path.join(root,
  "content-manifests/fate-star-rail-night-v1/source-inventory.json");
const turnRoot = path.join(sourceCache, "turnbasedgamedata");
const resRoot = path.join(sourceCache, "StarRailRes");

const records = [
  ...walk(turnRoot).map((relative) => sourceRecord(
    "turnbasedgamedata", turnRoot, relative, classifyTurn(relative))),
  ...walk(resRoot).map((relative) => sourceRecord(
    "StarRailRes", resRoot, relative, classifyRes(relative))),
].sort((left, right) => compareText(
  `${left.repository}/${left.path}`, `${right.repository}/${right.path}`));
const turnTree = treePaths(turnRoot);
const namedExclusions = [
  exclusion("rtbattle-adjacent", "Config/Activity/RtBattle/",
    turnTree.filter((entry) => entry.startsWith("Config/Activity/RtBattle/")),
    "Adjacent real-time battle configuration; no FateRin-originating selector."),
  exclusion("currency-wars-fate-bonds", "ExcelOutput/GridFight*Trait*.json",
    turnTree.filter((entry) => entry.startsWith("ExcelOutput/GridFight") &&
      entry.includes("Trait") && entry.endsWith(".json")),
    "Currency Wars-owned Bond/Trait tables; Fate terminology inside rows is not reachability."),
  exclusion("reward-and-talk", "ExcelOutput/FateRin{ResidentReward,SwitchDayTalk}.json",
    records.filter(({ path: relative }) =>
      relative === "ExcelOutput/FateRinResidentReward.json" ||
      relative === "ExcelOutput/FateRinSwitchDayTalk.json")
      .map(({ path: relative }) => relative),
    "Evidence-only unless a row proves a mechanical unlock or graph locator."),
];

const categoryCounts = countBy(records, "category");
const dispositionCounts = countBy(records, "disposition");
const document = {
  schema_revision: "starclock.fate-star-rail-night-source-inventory.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  batch: "G19-P0-B2",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-08-01",
    turnbased_revision: git(turnRoot, ["rev-parse", "HEAD"]),
    starrailres_revision: git(resRoot, ["rev-parse", "HEAD"]),
  },
  contract: {
    inventory_is_denominator: false,
    membership_rule: "explicit-faterin-selector-and-transitive-reference-closure",
    row_locator: "zero-based-top-level-index-or-object-key",
    long_prose_committed: false,
  },
  counts: {
    files: records.length,
    json_files: records.filter(({ json }) => json).length,
    top_level_rows: records.reduce((sum, row) => sum + row.top_level_rows, 0),
    categories: categoryCounts,
    dispositions: dispositionCounts,
    named_exclusion_paths: namedExclusions.reduce(
      (sum, row) => sum + row.path_count, 0),
  },
  records,
  named_exclusions: namedExclusions,
};
document.canonical_records_sha256 = digest(
  `${JSON.stringify({ records, named_exclusions: namedExclusions })}\n`);

const serialized = `${JSON.stringify(document, null, 2)}\n`;
if (check) {
  assert(fs.existsSync(output), `missing ${path.relative(root, output)}`);
  assert(fs.readFileSync(output, "utf8") === serialized,
    "Goal 19 source inventory drift");
  console.log(summary("verified"));
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, serialized);
  console.log(summary("wrote"));
}

function sourceRecord(repository, repositoryRoot, relative, classification) {
  const bytes = fs.readFileSync(path.join(repositoryRoot, relative));
  const json = relative.endsWith(".json");
  let topLevelRows = 0;
  let jsonShape = "non-json";
  if (json) {
    const value = JSON.parse(bytes.toString("utf8"));
    jsonShape = Array.isArray(value) ? "array" : value === null ? "null" : typeof value;
    topLevelRows = Array.isArray(value) ? value.length
      : value !== null && typeof value === "object" ? Object.keys(value).length : 1;
  }
  return {
    repository,
    path: relative,
    bytes: bytes.length,
    sha256: digest(bytes),
    json,
    json_shape: jsonShape,
    top_level_rows: topLevelRows,
    ...classification,
  };
}

function classifyTurn(relative) {
  if (relative.startsWith("ExcelOutput/FateRin")) {
    const evidenceOnly = relative.endsWith("ResidentReward.json") ||
      relative.endsWith("SwitchDayTalk.json") ||
      relative.endsWith("MainMissions.json");
    return { category: "FateRinTable",
      disposition: evidenceOnly ? "EvidenceOnly" : "SelectorSeed" };
  }
  if (relative.startsWith("ExcelOutput/Fate"))
    return { category: "FateTable", disposition: "SelectorSeed" };
  if (relative.startsWith("Config/Gameplays/Fate/"))
    return { category: "FateGameplayConfig", disposition: "SelectorSeed" };
  if (relative.startsWith("Config/") && relative.includes("FateRin"))
    return { category: "FateFocusedConfig", disposition: "ClosureCandidate" };
  if (relative.startsWith("ExcelOutput/"))
    return { category: "SharedTable", disposition: "ClosureCandidate" };
  if (relative.startsWith("TextMap/"))
    return { category: "BilingualTextMap", disposition: "EvidenceOnly" };
  return { category: "SourceMetadata", disposition: "EvidenceOnly" };
}

function classifyRes(relative) {
  if (relative.startsWith("index_new/"))
    return { category: "IdentityCrossCheck", disposition: "EvidenceOnly" };
  return { category: "SourceMetadata", disposition: "EvidenceOnly" };
}

function exclusion(id, selector, paths, reason) {
  const sorted = [...paths].sort(compareText);
  return { id, selector, disposition: "NamedExcludedCandidate",
    path_count: sorted.length, paths: sorted, reason };
}

function walk(directory, prefix = "") {
  const rows = [];
  for (const entry of fs.readdirSync(path.join(directory, prefix),
    { withFileTypes: true }).sort((left, right) => compareText(left.name, right.name))) {
    if (entry.name === ".git") continue;
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) rows.push(...walk(directory, relative));
    else if (entry.isFile()) rows.push(relative);
  }
  return rows;
}

function treePaths(repositoryRoot) {
  return git(repositoryRoot, ["ls-tree", "-r", "--name-only", "HEAD"])
    .split("\n").filter(Boolean);
}

function countBy(rows, field) {
  return Object.fromEntries([...new Set(rows.map((row) => row[field]))]
    .sort(compareText).map((value) => [value,
      rows.filter((row) => row[field] === value).length]));
}

function git(cwd, command) {
  return execFileSync("git", ["-C", cwd, ...command], {
    encoding: "utf8", maxBuffer: 64 * 1024 * 1024,
  }).trim();
}

function argument(name) {
  const index = args.indexOf(name);
  assert(index !== -1 && args[index + 1], `${name} requires a value`);
  return path.resolve(args[index + 1]);
}

function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function summary(verb) {
  return `Goal 19 source inventory ${verb} (${records.length} files, ` +
    `${document.counts.top_level_rows} top-level rows, ` +
    `${document.canonical_records_sha256}).`;
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
