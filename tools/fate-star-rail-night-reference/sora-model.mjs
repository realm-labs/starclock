import { readFileSync } from "node:fs";
import { resolve } from "node:path";

export const COMMON_FIELDS = [
  ["stable_key", "string"],
  ["family", "string"],
  ["name_zh", "string"],
  ["name_en", "string"],
  ["summary_zh", "string"],
  ["summary_en", "string"],
  ["ownership", "string"],
  ["disposition", "string"],
  ["enabled", "string"],
  ["source_refs_json", "string"],
  ["evidence_quality", "string"],
  ["mechanism_quality", "string"],
  ["confidence", "string"],
  ["text_hashes_json", "string"],
  ["mechanic_payload_json", "string"],
  ["notes", "string"],
];

export const WORKBOOKS = {
  activity: "FateStarRailNight.xlsx",
  combat: "FateStarRailNightCombat.xlsx",
  bindings: "FateStarRailNightBindings.xlsx",
  review: "FateStarRailNightReview.xlsx",
};

export const TABLES = [
  table("G19-P3-B1", "activity", "Profiles", [], { derived: "profile" }),
  table("G19-P3-B1", "activity", "Areas", ["FateArea"]),
  table("G19-P3-B1", "activity", "Difficulties", ["FateDifficulty"]),
  table("G19-P3-B1", "activity", "Phases", ["FatePhase"]),
  table("G19-P3-B1", "activity", "BattleZones", ["FateBattleZone"]),
  table("G19-P3-B1", "activity", "Progress", ["FateDiffPassProgress", "FateRinDayProgress"]),
  table("G19-P3-B1", "activity", "CaseBoards", ["FateRinCaseBoard"]),
  table("G19-P3-B1", "activity", "CaseBoardNodes", ["FateRinCaseBoardInfo"]),
  table("G19-P3-B1", "activity", "Participants", ["FateClazz"]),
  table("G19-P3-B1", "activity", "Teams", ["FateRinCaseBoardTeamInfo"]),
  table("G19-P3-B1", "activity", "Owners", ["FateRinOwner"]),
  table("G19-P3-B1", "activity", "Traits", ["FateTrait"]),
  table("G19-P3-B1", "activity", "Levels", ["FateRinLevelUp", "FateExpReward"]),
  table("G19-P3-B1", "activity", "Unlocks", ["FateRinOwnerInitHougu"]),
  table("G19-P3-B2", "bindings", "Masters", ["FateMaster", "FateHandbookMaster", "FateRinAvatar"]),
  table("G19-P3-B2", "bindings", "Servants", ["FateRinCaseBoardServant"]),
  table("G19-P3-B2", "bindings", "NoblePhantasms", ["FateHougu"]),
  table("G19-P3-B2", "bindings", "NoblePhantasmLevels", ["FateRinHouguConfig"]),
  table("G19-P3-B2", "bindings", "Rarities", ["FateRinHouguRarity"]),
  table("G19-P3-B2", "bindings", "Tags", ["FateRinHouguTag"]),
  table("G19-P3-B2", "bindings", "Keywords", ["FateRinHouguKeyword"]),
  table("G19-P3-B2", "bindings", "Decks", ["FateRinDeck"]),
  table("G19-P3-B2", "bindings", "DeckRecommendations", ["FateRinDeckRecommend"]),
  table("G19-P3-B2", "bindings", "CommandSpells", ["FateReiju"]),
  table("G19-P3-B2", "bindings", "CommandSpellAffixes", ["FateReijuAffix"]),
  table("G19-P3-B2", "bindings", "Resources", ["FateConstValueClient", "FateConstValueCommon", "FateRinConstClient", "FateRinConstCommon"]),
  table("G19-P3-B2", "bindings", "RuleBindings", ["FateGameplayConfig"]),
  table("G19-P3-B2", "bindings", "LifecycleBindings", ["FateAffix", "FateTraitBuff"]),
];

export function tablesThrough(batch) {
  const batchNumber = Number(batch.match(/B(\d+)$/u)?.[1]);
  if (!Number.isInteger(batchNumber)) throw new Error(`invalid batch ${batch}`);
  return TABLES.filter((entry) =>
    Number(entry.batch.match(/B(\d+)$/u)?.[1]) <= batchNumber);
}

export function rowsForTable(root, definition) {
  if (definition.derived === "profile") return [profileRow(root)];
  const referenceRoot = resolve(root, "content-reference/fate-star-rail-night-v1");
  const files = [
    "profile-graph.json",
    "participants.json",
    "progression-traits.json",
    "noble-phantasms.json",
    "command-spells.json",
    "effects.json",
  ];
  const familySet = new Set(definition.families);
  const records = files.flatMap((file) =>
    json(resolve(referenceRoot, file)).records ?? []);
  return records.filter(({ family }) => familySet.has(family));
}

export function workbookFor(definition) {
  const workbook = WORKBOOKS[definition.workbook];
  if (!workbook) throw new Error(`unknown workbook ${definition.workbook}`);
  return workbook;
}

export function canonicalRow(row, ordinal) {
  return {
    id: ordinal + 1,
    stable_key: row.stable_id,
    family: row.family,
    name_zh: row.name_zh,
    name_en: row.name_en,
    summary_zh: row.summary_zh,
    summary_en: row.summary_en,
    ownership: row.ownership,
    disposition: row.disposition,
    enabled: String(row.enabled),
    source_refs_json: canonicalJson(row.source_refs ?? []),
    evidence_quality: row.evidence_quality,
    mechanism_quality: row.mechanism_quality,
    confidence: row.confidence,
    text_hashes_json: canonicalJson(row.text_hashes ?? []),
    mechanic_payload_json: canonicalJson(row.mechanic_payload ?? {}),
    notes: row.notes ?? "",
  };
}

function table(batch, workbook, sheet, families, extra = {}) {
  return { batch, workbook, sheet, families, ...extra };
}

function profileRow(root) {
  const pack = json(resolve(
    root,
    "content-reference/fate-star-rail-night-v1/pack-index.json",
  ));
  return {
    stable_id: "fate-star-rail-night.profile.candidate-v4.4",
    family: "FateProfile",
    name_zh: "Fate/Star Rail Night 4.4 候选资料档案",
    name_en: "Fate/Star Rail Night 4.4 Candidate Reference Profile",
    summary_zh: "绑定已发布 4.4 快照、精确覆盖分母和独立资料包摘要。",
    summary_en: "Binds the released 4.4 snapshot, exact coverage denominator and isolated pack digest.",
    ownership: "FateStarRailNight",
    disposition: "DataReady",
    enabled: true,
    source_refs: [{
      path: "content-reference/fate-star-rail-night-v1/pack-index.json",
      locator: "root",
      sha256: pack.pack_sha256,
    }],
    evidence_quality: "DerivedFromExactStructured",
    mechanism_quality: "ReferenceProfileOnly",
    confidence: "High",
    text_hashes: [],
    mechanic_payload: {
      pack_sha256: pack.pack_sha256,
      normalized_records: String(pack.counts.normalized_records),
      manifest_obligations: String(pack.counts.manifest_obligations),
      runtime_executable: "false",
    },
    notes: "Derived index row; it does not enlarge the frozen obligation denominator.",
  };
}

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function json(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}
