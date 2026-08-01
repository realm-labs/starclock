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
  table("G19-P3-B3", "combat", "Stages", ["Stage"]),
  table("G19-P3-B3", "combat", "BattleAreas", ["BattleArea", "BattleAreaConfig"]),
  table("G19-P3-B3", "combat", "Encounters", ["FateRinChallengeFight", "FateRinStoryFight", "FateRinHouguMapFight", "FateRinHouguMapGroup", "FateMonsterPool"]),
  table("G19-P3-B3", "combat", "Waves", ["Wave"]),
  table("G19-P3-B3", "combat", "EnemySlots", ["EnemySlot"]),
  table("G19-P3-B3", "combat", "EnemyVariants", ["EnemyVariant"]),
  table("G19-P3-B3", "combat", "EnemyTemplates", ["EnemyTemplate"]),
  table("G19-P3-B3", "combat", "EnemySkills", ["EnemySkill", "EnemyProgram"]),
  table("G19-P3-B3", "combat", "EnemyStatuses", ["FateStatusConfig"]),
  table("G19-P3-B3", "combat", "Buffs", ["FateBuff", "FateBuffSlot", "FateRinChallengeFightBuff"]),
  table("G19-P3-B3", "combat", "MazeBuffs", ["FateMazeBuff", "MazeBuff"]),
  table("G19-P3-B3", "combat", "BattleEvents", ["BattleEvent"]),
  table("G19-P3-B3", "combat", "BattleTargets", ["BattleTarget"]),
  table("G19-P3-B4", "review", "Sources", [], { derived: "sources" }),
  table("G19-P3-B4", "review", "ContentAudit", [], { derived: "content-audit" }),
  table("G19-P3-B4", "review", "Coverage", [], { derived: "coverage" }),
  table("G19-P3-B4", "review", "ResearchGaps", [], { derived: "research-gaps" }),
  table("G19-P3-B4", "review", "Reconciliation", [], { derived: "reconciliation" }),
  table("G19-P3-B4", "review", "ReviewFixtures", [], { derived: "review-fixtures" }),
  table("G19-P3-B4", "review", "PackFiles", [], { derived: "pack-files" }),
];

export function tablesThrough(batch) {
  const batchNumber = Number(batch.match(/B(\d+)$/u)?.[1]);
  if (!Number.isInteger(batchNumber)) throw new Error(`invalid batch ${batch}`);
  return TABLES.filter((entry) =>
    Number(entry.batch.match(/B(\d+)$/u)?.[1]) <= batchNumber);
}

export function rowsForTable(root, definition) {
  if (definition.derived === "profile") return [profileRow(root)];
  if (definition.derived) return reviewRows(root, definition.derived);
  const referenceRoot = resolve(root, "content-reference/fate-star-rail-night-v1");
  const files = [
    "profile-graph.json",
    "participants.json",
    "progression-traits.json",
    "noble-phantasms.json",
    "command-spells.json",
    "effects.json",
    "fight-flow.json",
    "battle-bindings.json",
    "encounters.json",
    "enemies.json",
    "enemy-programs.json",
  ];
  const familySet = new Set(definition.families);
  const records = files.flatMap((file) =>
    json(resolve(referenceRoot, file)).records ?? []);
  return records.filter(({ family }) => familySet.has(family));
}

function reviewRows(root, kind) {
  const referenceRoot = resolve(root, "content-reference/fate-star-rail-night-v1");
  if (kind === "content-audit") {
    const families = new Set([
      "FateAvatarDescription",
      "FateBroadcast",
      "FateFocusedLayout",
      "FateMasterTalk",
      "FateMiscDisplay",
      "FateRinMainMissions",
      "FateRinResidentReward",
      "FateRinSwitchDayTalk",
      "PoolAudit",
    ]);
    const records = [
      "participants.json",
      "fight-flow.json",
      "pool-audits.json",
    ].flatMap((file) => json(resolve(referenceRoot, file)).records ?? []);
    return records.filter(({ family }) => families.has(family));
  }
  const definitions = {
    sources: ["sources.json", "sources", "source_id", "SourceReceipt"],
    coverage: ["coverage.json", "rows", "obligation_id", "CoverageReceipt"],
    "research-gaps": ["research-gaps.json", "policies", "policy_id", "ResearchPolicy"],
    reconciliation: ["reconciliation.json", "receipts", "peer_goal", "ReconciliationReceipt"],
    "review-fixtures": ["review-fixtures.json", "fixtures", "fixture_id", "ReviewFixture"],
    "pack-files": ["pack-index.json", "files", "path", "PackFile"],
  };
  const [file, key, identity, family] = definitions[kind] ?? [];
  if (!file) throw new Error(`unknown derived review table ${kind}`);
  return json(resolve(referenceRoot, file))[key].map((payload, index) =>
    reviewEnvelope(family, payload[identity] ?? `${kind}.${index + 1}`, payload));
}

function reviewEnvelope(family, identity, payload) {
  const stable = String(identity).toLowerCase().replace(/[^a-z0-9._-]+/gu, "-");
  return {
    stable_id: `fate-star-rail-night.review.${family.toLowerCase()}.${stable}`,
    family,
    name_zh: `${family} ${identity}`,
    name_en: `${family} ${identity}`,
    summary_zh: "保留审计身份和规范化负载。",
    summary_en: "Retains the audit identity and canonical payload.",
    ownership: "EvidenceOnly",
    disposition: "EvidenceOnly",
    enabled: false,
    source_refs: [],
    evidence_quality: payload.evidence_quality ?? "DerivedAudit",
    mechanism_quality: payload.mechanism_quality ?? "AuditOnly",
    confidence: "EvidenceOnly",
    text_hashes: [],
    mechanic_payload: payload,
    notes: "Review-only row; excluded from runtime and the mechanical row denominator.",
  };
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
