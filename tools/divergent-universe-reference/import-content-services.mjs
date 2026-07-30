#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const manifest = JSON.parse(await import("node:fs/promises").then(({ readFile }) =>
  readFile(
    path.join(
      root,
      "content-manifests/divergent-universe-v1/content-manifest.json",
    ),
    "utf8",
  )));
const serviceIds = new Set(
  manifest.categories.mode_service_npcs.records.map((row) => row.id),
);
const missingGraphPolicy = await context.policyRef(
  "service-npc-graphs-missing",
  "The 23 non-handbook Tourn3 NPC rows publish RogueNPC_410 graph paths, but none exists in the pinned Git tree. Service kind, choices, prices and effects cannot be assigned from numeric ID or adjacency.",
  "Replace each missing service boundary only when a released exact graph or explicit service binding identifies the NPC's operations and choices.",
);
const serviceRows = (await context.table("RogueTournNPC"))
  .filter(({ row }) => serviceIds.has(String(row.RogueNPCID)))
  .map((entry) => {
    const sourceId = String(entry.row.RogueNPCID);
    return {
      ...context.envelope({
        id: `divergent-universe.mode-service-npc.${sourceId}`,
        kind: "DivergentUniverseModeServiceNpc",
        nameEn: `Tourn3 Service NPC ${sourceId}`,
        nameZh: `Tourn3 服务 NPC ${sourceId}`,
        summaryEn:
          `Non-handbook Tourn3 NPC ${sourceId} publishes missing graph path ${entry.row.NPCJsonPath}; its service kind and choices remain unclassified.`,
        summaryZh:
          `非图鉴 Tourn3 NPC ${sourceId} 发布缺失图路径 ${entry.row.NPCJsonPath}；其服务类型与选择保持未分类。`,
        coverageState: "Researched",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(entry), missingGraphPolicy],
        tags: ["service-npc", "tourn3", "graph-missing"],
      }),
      source_id: sourceId,
      graph_path: entry.row.NPCJsonPath,
      graph_resolution: "MissingAtPinnedRevision",
      service_kind: "UnclassifiedMissingGraph",
      choice_ids: [],
      fallback: "RejectWithoutMutation",
      runtime_lowered: false,
    };
  }).sort(compareIds);

const programTables = new Map();
for (const [type, tableName] of [
  ["RogueCaptureMonster", "RogueCaptureMonster"],
  ["RogueDestroyProp", "RogueDestroyProp"],
  ["RogueTurntable", "RogueTurntable"],
  ["RogueEscapeLaser", "RogueEscapeLaser"],
  ["RogueCandyCrash", "RogueCandyCrash"],
])
  programTables.set(type, await context.table(tableName));
const adventurePolicy = await context.policyRef(
  "adventure-external-results",
  "Adventure action gameplay is outside Goal 11. Preserve only exact timing/score thresholds and treat the accepted external result as an abstract settlement input; never simulate movement, aiming, object placement or action controls.",
  "A later runtime goal may bind these abstract results to generic Activity operations without adding an Adventure action simulator.",
);
const missingWolfPolicy = await context.policyRef(
  "wolf-gun-parameter-groups",
  "RogueTournAdventureRoom references RogueWolfGun parameter groups 101 and 102, but no released parameter-group table identifies those programs. RogueWolfGunMiracleTarget has no parameter-group key and cannot prove the join.",
  "Replace the empty parameter program when a released table or config explicitly binds Wolf Gun group 101/102 to its thresholds and results.",
);
const adventureRows = (await context.table("RogueTournAdventureRoom"))
  .map((entry) => {
    const sourceId =
      `${entry.row.RoomID}:${entry.row.AdventureType}`;
    const candidates = programTables.get(entry.row.AdventureType) ?? [];
    const parameterEntries = candidates.filter(({ row }) =>
      String(row.ParamGroupID) === String(entry.row.ParamGroupID));
    const resolved = parameterEntries.length > 0;
    const outcomeKind = new Map([
      ["RogueCaptureMonster", "ScoreThresholdSettlement"],
      ["RogueDestroyProp", "ScoreThresholdSettlement"],
      ["RogueTurntable", "RewardTierSettlement"],
      ["RogueEscapeLaser", "RoundScoreSettlement"],
      ["RogueWolfGun", "ExternalAdventureResult"],
      ["RogueCandyCrash", "RoundThresholdSettlement"],
    ]).get(entry.row.AdventureType);
    if (!outcomeKind)
      throw new Error(`unknown Adventure type ${entry.row.AdventureType}`);
    return {
      ...context.envelope({
        id:
          `divergent-universe.adventure-outcome.${entry.row.RoomID}.` +
          entry.row.AdventureType,
        kind: "DivergentUniverseAdventureOutcome",
        nameEn:
          `${entry.row.AdventureType} Room ${entry.row.RoomID}`,
        nameZh:
          `${entry.row.AdventureType} 房间 ${entry.row.RoomID}`,
        summaryEn: resolved
          ? `${entry.row.AdventureType} room ${entry.row.RoomID} binds parameter group ${entry.row.ParamGroupID} to ${parameterEntries.length} exact settlement program row(s).`
          : `${entry.row.AdventureType} room ${entry.row.RoomID} publishes unresolved parameter group ${entry.row.ParamGroupID}; only the external-result boundary is retained.`,
        summaryZh: resolved
          ? `${entry.row.AdventureType} 房间 ${entry.row.RoomID} 将参数组 ${entry.row.ParamGroupID} 绑定到 ${parameterEntries.length} 条精确结算程序记录。`
          : `${entry.row.AdventureType} 房间 ${entry.row.RoomID} 发布未解析参数组 ${entry.row.ParamGroupID}；仅保留外部结果边界。`,
        coverageState: resolved ? "DataReady" : "Researched",
        evidenceQuality: resolved ? "ExactStructured" : "ProjectPolicy",
        sourceRefs: [
          context.sourceRef(entry),
          ...parameterEntries.map((parameter) =>
            context.sourceRef(parameter)),
          adventurePolicy,
          ...(!resolved ? [missingWolfPolicy] : []),
        ],
        tags: [
          "adventure",
          entry.row.AdventureType,
          "abstract-outcome",
          ...(resolved ? ["parameter-program"] : ["parameter-missing"]),
        ],
      }),
      source_id: sourceId,
      room_id: String(entry.row.RoomID),
      adventure_type: entry.row.AdventureType,
      parameter_group_id: String(entry.row.ParamGroupID),
      parameter_program: parameterEntries.map(({ row }) =>
        normalizeNumbers(row)),
      abstract_outcome: {
        kind: outcomeKind,
        input: "AcceptedExternalAdventureResult",
        ordered_operations: ["ValidateResult", "ApplyAuthoredSettlement"],
      },
      action_gameplay: "Excluded",
      fallback: "RejectWithoutMutation",
      runtime_lowered: false,
    };
  }).sort(compareIds);

await writeOrCheck(context, new Map([
  ["mode-service-npcs.json", serviceRows],
  ["adventure-outcomes.json", adventureRows],
]), check);
console.log(
  `Divergent Universe content services ${check ? "verified" : "generated"}: ` +
  `${serviceRows.length} missing-graph NPCs and ${adventureRows.length} ` +
  `abstract Adventure outcomes.`,
);

function normalizeNumbers(value) {
  if (Array.isArray(value)) return value.map(normalizeNumbers);
  if (value && typeof value === "object")
    return Object.fromEntries(Object.entries(value).map(([key, child]) =>
      [key, normalizeNumbers(child)]));
  return typeof value === "number" ? decimal(value) : value;
}

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function compareIds(left, right) {
  return left.id.localeCompare(right.id);
}
