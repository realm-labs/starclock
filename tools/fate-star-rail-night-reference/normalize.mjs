#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceCache = path.resolve(argument("--source-cache"));
const batch = argument("--batch");
const check = args.includes("--check");
const turnRoot = path.join(sourceCache, "turnbasedgamedata");
const inventory = json(path.join(root,
  "content-manifests/fate-star-rail-night-v1/source-inventory.json"));
const manifest = json(path.join(root,
  "content-manifests/fate-star-rail-night-v1/content-manifest.json"));

const batches = {
  "G19-P1-B1": {
    output: "profile-graph.json",
    purpose: "profile, area, difficulty, phase, progress, Case Board and fight graph",
    tables: [
      "FateArea", "FateDifficulty", "FatePhase", "FateBattleZone",
      "FateDiffPassProgress", "FateRinDayProgress", "FateRinCaseBoard",
      "FateRinCaseBoardInfo", "FateRinChallengeFight", "FateRinStoryFight",
      "FateRinHouguMapGroup", "FateRinHouguMapFight",
    ],
    configPrefixes: [],
  },
  "G19-P1-B2": {
    output: "participants.json",
    purpose: "Masters, Servants, avatars, teams and participant ownership",
    tables: [
      "FateAvatarDescription", "FateClazz", "FateHandbookMaster", "FateMaster",
      "FateRinAvatar", "FateRinCaseBoardServant", "FateRinCaseBoardTeamInfo",
      "FateRinOwner",
    ],
    configPrefixes: ["Config/Gameplays/Fate/MasterConfig/"],
  },
  "G19-P1-B3": {
    output: "noble-phantasms.json",
    purpose: "Noble Phantasm identities, rarity, tags, keywords and decks",
    tables: [
      "FateHougu", "FateRinHouguConfig", "FateRinHouguRarity",
      "FateRinHouguTag", "FateRinHouguKeyword", "FateRinDeck",
      "FateRinDeckRecommend",
    ],
    configPrefixes: [],
  },
  "G19-P1-B4": {
    output: "effects.json",
    purpose: "buff, status, target/effect and challenge contribution facts",
    tables: [
      "FateBuff", "FateBuffSlot", "FateMazeBuff", "FateStatusConfig",
      "FateTraitBuff", "FateRinChallengeFightBuff",
    ],
    configPrefixes: [],
  },
  "G19-P1-B5": {
    output: "command-spells.json",
    purpose: "Command Spells, Reiju affixes, constants and resource transitions",
    tables: [
      "FateReiju", "FateReijuAffix", "FateRinConstClient",
      "FateRinConstCommon", "FateConstValueClient", "FateConstValueCommon",
    ],
    configPrefixes: ["Config/Gameplays/Fate/ReijuConfig/"],
  },
  "G19-P1-B6": {
    output: "progression-traits.json",
    purpose: "affixes, experience, traits, levels and initial Noble Phantasms",
    tables: [
      "FateAffix", "FateExpReward", "FateTrait", "FateRinLevelUp",
      "FateRinOwnerInitHougu",
    ],
    configPrefixes: ["Config/Gameplays/Fate/TraitConfig/"],
  },
  "G19-P1-B7": {
    output: "fight-flow.json",
    purpose: "monster pools, mechanically relevant locators and bounded presentation evidence",
    tables: [
      "FateMonsterPool", "FateBroadcast", "FateMasterTalk", "FateMiscDisplay",
      "FateRinMainMissions", "FateRinResidentReward", "FateRinSwitchDayTalk",
    ],
    configPrefixes: ["Config/ConfigAI/", "Config/ConfigAbility/",
      "Config/ConfigAnimEvents/", "Config/ConfigCharacter/"],
  },
};
const definition = batches[batch];
assert(definition !== undefined, `unsupported normalization batch ${batch}`);

const zhText = losslessJson(fs.readFileSync(path.join(turnRoot,
  "TextMap/TextMapCHS.json")));
const enText = losslessJson(fs.readFileSync(path.join(turnRoot,
  "TextMap/TextMapEN.json")));
const records = [];
for (const table of definition.tables) {
  const relative = `ExcelOutput/${table}.json`;
  const source = sourceReceipt(relative);
  const rows = losslessJson(fs.readFileSync(path.join(turnRoot, relative)));
  rows.forEach((row, index) => records.push(normalizeRow(table, source, row, index)));
}
for (const prefix of definition.configPrefixes)
  for (const source of inventory.records.filter(({ repository, path: relative }) =>
    repository === "turnbasedgamedata" && relative.startsWith(prefix) &&
      (relative.startsWith("Config/Gameplays/Fate/") || relative.includes("FateRin"))))
    records.push(normalizeConfig(source));

records.sort((left, right) => compareText(left.stable_id, right.stable_id));
const document = {
  schema_revision: "starclock.fate-star-rail-night-normalized.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  batch,
  purpose: definition.purpose,
  manifest_binding: manifest.canonical_obligations_sha256,
  counts: {
    records: records.length,
    enabled: records.filter(({ enabled }) => enabled).length,
    evidence_only: records.filter(({ disposition }) => disposition === "EvidenceOnly").length,
    families: countBy(records, "family"),
  },
  records,
};
document.canonical_records_sha256 = digest(canonical(records));
const output = path.join(root, "content-reference/fate-star-rail-night-v1",
  definition.output);
const serialized = `${JSON.stringify(document, null, 2)}\n`;

if (check) {
  assert(fs.existsSync(output), `missing normalized output ${definition.output}`);
  assert(fs.readFileSync(output, "utf8") === serialized,
    `normalized output drift ${definition.output}`);
  verifyRecords(records);
  console.log(summary("verified"));
} else {
  fs.writeFileSync(output, serialized);
  console.log(summary("wrote"));
}

function normalizeRow(family, source, row, index) {
  const obligation = manifest.obligations.find(({ source_path: sourcePath, locator }) =>
    sourcePath === source.path && locator === `index:${index}`);
  assert(obligation !== undefined, `missing manifest row ${source.path}#${index}`);
  const hashes = [...new Set(textHashes(row))].sort(compareText);
  const name = firstShortBilingual(hashes);
  const ordinal = String(index + 1);
  return {
    stable_id: `fate-star-rail-night.${slug(family)}.${String(index + 1).padStart(4, "0")}`,
    family,
    name_zh: name?.zh ?? `${family} 条目 ${ordinal}`,
    name_en: name?.en ?? `${family} entry ${ordinal}`,
    summary_zh: `保留 ${family} 第 ${ordinal} 条已发布机械事实；字段语义待类型化阶段复核。`,
    summary_en: `Released ${family} mechanic record ${ordinal}; typed field meaning remains reviewable.`,
    ownership: obligation.ownership,
    disposition: obligation.disposition,
    enabled: obligation.disposition === "DataReady",
    source_refs: [{ path: source.path, locator: `index:${index}`, sha256: source.sha256 }],
    evidence_quality: "ExactStructured",
    mechanism_quality: "TransportedExact",
    confidence: obligation.disposition === "DataReady" ? "High" : "EvidenceOnly",
    text_hashes: hashes,
    mechanic_payload: canonicalValue(stripPresentation(row)),
    notes: "Obfuscated upstream field names are transported without invented semantics; long text and assets are omitted.",
  };
}

function normalizeConfig(source) {
  const obligation = manifest.obligations.find(({ source_path: sourcePath, locator }) =>
    sourcePath === source.path && locator === "file");
  assert(obligation !== undefined, `missing manifest config ${source.path}`);
  const stem = path.basename(source.path, ".json").replace(/\.layout$/u, "");
  return {
    stable_id: `fate-star-rail-night.config.${slug(source.path)}`,
    family: obligation.family,
    name_zh: `${stem} 配置`,
    name_en: `${stem} configuration`,
    summary_zh: "保留配置文件身份和摘要，不复制或执行上游程序。",
    summary_en: "Retains configuration identity and digest without copying or executing the upstream program.",
    ownership: obligation.ownership,
    disposition: obligation.disposition,
    enabled: obligation.disposition === "DataReady",
    source_refs: [{ path: source.path, locator: "file", sha256: source.sha256 }],
    evidence_quality: "ExactStructured",
    mechanism_quality: source.path.endsWith(".layout.json") ? "IdentityOnly" : "ProgramDigestExact",
    confidence: source.path.endsWith(".layout.json") ? "EvidenceOnly" : "High",
    text_hashes: [],
    mechanic_payload: { bytes: String(source.bytes), program_sha256: source.sha256 },
    notes: "Runtime lowering is excluded.",
  };
}

function stripPresentation(value, key = "") {
  if (Array.isArray(value)) return value.map((entry) => stripPresentation(entry, key));
  if (value && typeof value === "object") {
    if (Object.keys(value).length === 1 && Object.hasOwn(value, "Hash")) return undefined;
    return Object.fromEntries(Object.entries(value)
      .map(([childKey, child]) => [childKey, stripPresentation(child, childKey)])
      .filter(([, child]) => child !== undefined));
  }
  if (typeof value === "string" && (/path|icon|sprite|prefab|image|audio/iu.test(key)
    || /^(?:SpriteOutput|UI|Characters|Audio|Texture|Effects)\//u.test(value)))
    return undefined;
  return value;
}

function canonicalValue(value) {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value && typeof value === "object") return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) => compareText(left, right))
      .map(([key, child]) => [key, canonicalValue(child)]));
  if (typeof value === "number") return String(value);
  return value;
}

function textHashes(value) {
  if (Array.isArray(value)) return value.flatMap(textHashes);
  if (value && typeof value === "object") {
    if (Object.keys(value).length === 1 && Object.hasOwn(value, "Hash"))
      return [String(value.Hash)];
    return Object.values(value).flatMap(textHashes);
  }
  return [];
}

function firstShortBilingual(hashes) {
  for (const hash of hashes) {
    const zh = zhText[hash];
    const en = enText[hash];
    if (typeof zh === "string" && typeof en === "string" && zh.length > 0 &&
      en.length > 0 && zh.length <= 40 && en.length <= 80 &&
      !zh.includes("\n") && !en.includes("\n")) return { zh, en };
  }
  return undefined;
}

function verifyRecords(rows) {
  assert(new Set(rows.map(({ stable_id: id }) => id)).size === rows.length,
    "duplicate normalized stable id");
  for (const row of rows) {
    assert(row.name_zh && row.name_en && row.summary_zh && row.summary_en,
      `missing bilingual field ${row.stable_id}`);
    assert(row.source_refs.length === 1 && row.source_refs[0].sha256.length === 64,
      `invalid provenance ${row.stable_id}`);
  }
}

function sourceReceipt(relative) {
  const source = inventory.records.find(({ repository, path: sourcePath }) =>
    repository === "turnbasedgamedata" && sourcePath === relative);
  assert(source !== undefined, `missing source inventory receipt ${relative}`);
  return source;
}

function losslessJson(bytes) {
  return JSON.parse(bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu, '$1"$2"'));
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") return `{${Object.keys(value)
    .sort(compareText).map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

function countBy(rows, field) {
  return Object.fromEntries([...new Set(rows.map((row) => row[field]))]
    .sort(compareText).map((value) => [value,
      rows.filter((row) => row[field] === value).length]));
}

function json(absolute) {
  return JSON.parse(fs.readFileSync(absolute, "utf8"));
}

function argument(name) {
  const index = args.indexOf(name);
  assert(index !== -1 && args[index + 1], `${name} requires a value`);
  return args[index + 1];
}

function slug(value) {
  return value.replace(/([a-z0-9])([A-Z])/gu, "$1-$2")
    .replace(/[^A-Za-z0-9]+/gu, "-").replace(/^-|-$/gu, "").toLowerCase();
}

function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function summary(verb) {
  return `${batch} normalized pack ${verb} (${records.length} records, ${document.canonical_records_sha256}).`;
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
