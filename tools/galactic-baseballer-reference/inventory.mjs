#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(".");
const sourceCache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source"));
const output = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "source-inventory.json",
);
const locatorOutput = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "localization-locators.json",
);
const sources = [
  {
    id: "turnbasedgamedata",
    repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    root: path.join(sourceCache, "turnbasedgamedata"),
  },
  {
    id: "starrailres",
    repository: "https://github.com/Mar-7th/StarRailRes.git",
    revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
    root: path.join(sourceCache, "StarRailRes"),
  },
];
const sharedSeeds = new Set([
  "ExcelOutput/StageConfig.json",
  "ExcelOutput/BattleEventConfig.json",
  "ExcelOutput/BattleTargetConfig.json",
  "ExcelOutput/MazeBuff.json",
  "ExcelOutput/MonsterConfig.json",
  "ExcelOutput/MonsterTemplateConfig.json",
  "ExcelOutput/MonsterSkillConfig.json",
  "ExcelOutput/MonsterStatusConfig.json",
  "TextMap/TextMapCHS.json",
  "TextMap/TextMapEN.json",
]);
const crossCheckPaths = new Set([
  "LICENSE",
  "README.md",
  "info.json",
  "index_new/cn/achievements.json",
  "index_new/cn/characters.json",
  "index_new/en/achievements.json",
  "index_new/en/characters.json",
]);

function option(name) {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  if (args[index + 1] === undefined || args[index + 1].startsWith("--"))
    throw new Error(`${name} requires a path`);
  return args[index + 1];
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function git(source, gitArgs, encoding = "utf8") {
  return execFileSync("git", ["-C", source.root, ...gitArgs], {
    encoding,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
    maxBuffer: 256 * 1024 * 1024,
  });
}

function selectedPaths(source) {
  const lines = git(source, ["ls-tree", "-r", "--full-tree", "HEAD"])
    .trim().split("\n").filter(Boolean);
  const tree = new Map(lines.map((line) => {
    const match = /^100644 blob ([0-9a-f]{40})\t(.+)$/u.exec(line);
    if (match === null) throw new Error(`unexpected tree row: ${line}`);
    return [match[2], match[1]];
  }));
  const paths = [...tree.keys()].filter((relativePath) => {
    if (source.id === "starrailres") return crossCheckPaths.has(relativePath);
    return sharedSeeds.has(relativePath)
      || /^ExcelOutput\/(?:EvolveBuild|EvoBdSC).+\.json$/u.test(relativePath)
      || /^Config\/ConfigAbility\/BattleEvent\/EvolveBuild.+\.json$/u.test(relativePath)
      || /^Config\/ConfigCharacter\/BattleEvent\/EvolveBuild.+\.json$/u.test(relativePath);
  }).sort(compareText);
  return { paths, tree };
}

function classify(sourceId, relativePath) {
  if (sourceId === "starrailres") return "identity-cross-check";
  if (sharedSeeds.has(relativePath)) return "shared-closure-seed";
  if (/EvoBdSC|EvolveBuildSC_|EvolveBuild_.+_SC/u.test(relativePath))
    return "demon-king-candidate";
  if (/EvolveBuild/u.test(relativePath))
    return "departure-or-shared-candidate";
  throw new Error(`unclassified source path: ${relativePath}`);
}

const records = [];
const sourceMetadata = [];
const modeHashSources = new Map();
for (const source of sources) {
  if (git(source, ["rev-parse", "HEAD"]).trim() !== source.revision)
    throw new Error(`${source.id}: revision drift`);
  if (git(source, ["status", "--porcelain"]).trim() !== "")
    throw new Error(`${source.id}: source cache is dirty`);
  const remote = git(source, ["remote", "get-url", "origin"]).trim();
  if (remote !== source.repository)
    throw new Error(`${source.id}: origin drift`);
  const { paths, tree } = selectedPaths(source);
  for (const relativePath of paths) {
    const bytes = readFileSync(path.join(source.root, relativePath));
    const computedOid = createHash("sha1")
      .update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
    if (computedOid !== tree.get(relativePath))
      throw new Error(`${source.id}:${relativePath}: checked-out blob drift`);
    const classification = classify(source.id, relativePath);
    const record = {
      repository: source.id,
      revision: source.revision,
      path: relativePath,
      git_blob_oid: computedOid,
      bytes: bytes.length,
      sha256: sha256(bytes),
      classification,
    };
    if (relativePath.endsWith(".json")) {
      const parsed = JSON.parse(bytes.toString("utf8"));
      record.json_shape = Array.isArray(parsed) ? "array" : "object";
      record.json_rows = Array.isArray(parsed)
        ? parsed.length
        : Object.keys(parsed).length;
      if (Array.isArray(parsed) && parsed.length > 0
          && parsed[0] !== null && typeof parsed[0] === "object") {
        record.first_row_fields = Object.keys(parsed[0]).sort(compareText);
      }
    }
    records.push(record);
    if (source.id === "turnbasedgamedata"
        && classification !== "shared-closure-seed") {
      const text = bytes.toString("utf8");
      for (const match of text.matchAll(/"Hash"\s*:\s*(-?[0-9]+)/gu)) {
        const hash = BigInt.asUintN(64, BigInt(match[1])).toString();
        const owners = modeHashSources.get(hash) ?? new Set();
        owners.add(relativePath);
        modeHashSources.set(hash, owners);
      }
    }
  }
  sourceMetadata.push({
    id: source.id,
    repository: source.repository,
    revision: source.revision,
    tree: git(source, ["rev-parse", "HEAD^{tree}"]).trim(),
    selected_files: paths.length,
  });
}
records.sort((left, right) =>
  compareText(`${left.repository}\0${left.path}`, `${right.repository}\0${right.path}`));

const turnRoot = sources[0].root;
const titlePatterns = {
  chs: /银河球棒侠传说|银河球棒侠|启程篇|魔王篇/iu,
  en: /Legend of (?:the )?Galactic Baseballer|Galactic Baseballer/iu,
};
const localizationRecords = [];
const presentByLocale = {};
for (const [locale, relativePath] of [
  ["chs", "TextMap/TextMapCHS.json"],
  ["en", "TextMap/TextMapEN.json"],
]) {
  const textMap = JSON.parse(await readFile(path.join(turnRoot, relativePath), "utf8"));
  let referenced = 0;
  let titleMatched = 0;
  for (const [hash, value] of Object.entries(textMap)) {
    const owners = modeHashSources.get(hash);
    const directTitleMatch = titlePatterns[locale].test(value);
    if (owners === undefined && !directTitleMatch) continue;
    if (owners !== undefined) referenced += 1;
    if (directTitleMatch) titleMatched += 1;
    localizationRecords.push({
      locale,
      hash,
      source_path: relativePath,
      source_row_locator: `TextMap.${hash}`,
      value_bytes: Buffer.byteLength(value, "utf8"),
      value_sha256: sha256(Buffer.from(value, "utf8")),
      referenced_by: owners === undefined ? [] : [...owners].sort(compareText),
      direct_title_match: directTitleMatch,
    });
  }
  presentByLocale[locale] = { referenced, title_matched: titleMatched };
}
localizationRecords.sort((left, right) =>
  compareText(`${left.locale}\0${left.hash}`, `${right.locale}\0${right.hash}`));
const referencedHashes = [...modeHashSources.keys()].sort(compareText);
const localizationPayload = {
  schema_revision: "starclock.galactic-baseballer-localization-locators.v1",
  source_revision: sources[0].revision,
  hash_owners: referencedHashes.length,
  records: localizationRecords,
  counts: {
    total: localizationRecords.length,
    by_locale: presentByLocale,
  },
};
localizationPayload.canonical_sha256 = sha256(Buffer.from(
  `${JSON.stringify(localizationPayload.records)}\n`,
));

const inventory = {
  schema_revision: "starclock.galactic-baseballer-source-inventory.v1",
  goal_id: "galactic-baseballer-reference-v1",
  game_version: "4.4",
  access_date: "2026-07-30",
  sources: sourceMetadata,
  records,
  counts: {
    total: records.length,
    by_repository: Object.fromEntries(sources.map(({ id }) => [
      id,
      records.filter((record) => record.repository === id).length,
    ])),
    by_classification: Object.fromEntries(
      [...new Set(records.map(({ classification }) => classification))]
        .sort(compareText).map((classification) => [
          classification,
          records.filter((record) => record.classification === classification).length,
        ]),
    ),
    mode_hash_owners: referencedHashes.length,
    localization_locators: localizationRecords.length,
    candidate_json_rows: records
      .filter(({ classification }) =>
        classification === "departure-or-shared-candidate"
        || classification === "demon-king-candidate")
      .reduce((total, record) => total + (record.json_rows ?? 0), 0),
  },
  boundary: {
    inventory_is_not_membership_denominator: true,
    profile_membership_freezes_in: "G16-P0-B3",
    prefixes_and_ids_do_not_prove_reachability: true,
    story_assets_and_account_rewards_are_excluded: true,
  },
};
inventory.canonical_sha256 = sha256(Buffer.from(
  `${JSON.stringify(inventory.records)}\n`,
));

const expectedCounts = {
  total: 81,
  turnbasedgamedata: 74,
  starrailres: 7,
  departure: 41,
  demonKing: 23,
  shared: 10,
  crossCheck: 7,
  candidateRows: 1653,
  hashOwners: 1739,
  localizationLocators: 3403,
  referencedChs: 1510,
  referencedEn: 1510,
};
const actualCounts = inventory.counts;
if (actualCounts.total !== expectedCounts.total
    || actualCounts.by_repository.turnbasedgamedata
      !== expectedCounts.turnbasedgamedata
    || actualCounts.by_repository.starrailres !== expectedCounts.starrailres
    || actualCounts.by_classification["departure-or-shared-candidate"]
      !== expectedCounts.departure
    || actualCounts.by_classification["demon-king-candidate"]
      !== expectedCounts.demonKing
    || actualCounts.by_classification["shared-closure-seed"]
      !== expectedCounts.shared
    || actualCounts.by_classification["identity-cross-check"]
      !== expectedCounts.crossCheck
    || actualCounts.candidate_json_rows !== expectedCounts.candidateRows
    || actualCounts.mode_hash_owners !== expectedCounts.hashOwners
    || actualCounts.localization_locators !== expectedCounts.localizationLocators
    || localizationPayload.counts.by_locale.chs.referenced
      !== expectedCounts.referencedChs
    || localizationPayload.counts.by_locale.en.referenced
      !== expectedCounts.referencedEn) {
  throw new Error(
    `Goal 16 focused inventory denominator drift: ${JSON.stringify(actualCounts)}`,
  );
}

if (check) {
  const expectedInventory = JSON.parse(await readFile(output, "utf8"));
  const expectedLocators = JSON.parse(await readFile(locatorOutput, "utf8"));
  if (JSON.stringify(expectedInventory) !== JSON.stringify(inventory))
    throw new Error("Galactic Baseballer source inventory drift");
  if (JSON.stringify(expectedLocators) !== JSON.stringify(localizationPayload))
    throw new Error("Galactic Baseballer localization locator drift");
  console.log(
    `Galactic Baseballer inventory verified (${records.length} files, ` +
    `${localizationRecords.length} localization locators).`,
  );
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, `${JSON.stringify(inventory, null, 2)}\n`);
  await writeFile(locatorOutput, `${JSON.stringify(localizationPayload, null, 2)}\n`);
  console.log(
    `Wrote Galactic Baseballer inventory (${records.length} files, ` +
    `${localizationRecords.length} localization locators).`,
  );
}
