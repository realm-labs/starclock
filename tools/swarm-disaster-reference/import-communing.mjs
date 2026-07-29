#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const outputs = new Map();

function localized(reference, fallbackEn, fallbackZh) {
  return {
    en: context.text(reference, "en") || fallbackEn,
    zh: context.text(reference, "zh_cn") || fallbackZh,
  };
}
function common(values) {
  return context.envelope(values);
}
function ordered(records, fields = ["id"]) {
  return records.sort((left, right) => {
    for (const field of fields) {
      if (left[field] < right[field]) return -1;
      if (left[field] > right[field]) return 1;
    }
    return 0;
  });
}

const displays = await context.table("RogueAeonDisplay");
const displayById = new Map(displays.map((display) => [
  display.row.DisplayID,
  display,
]));
function pathIdentity(aeonId) {
  const display = displayById.get(aeonId);
  if (!display) throw new Error(`missing RogueAeonDisplay ${aeonId}`);
  const name = localized(
    display.row.RogueAeonPathName2,
    `Path ${aeonId}`,
    `命途 ${aeonId}`,
  );
  return {
    display,
    name,
    pathId: `universe.path.${slug(name.en.replace(/^The /u, ""))}`,
  };
}

const choicePolicy = await context.policyRef(
  "communing-choices",
  "A released MainStoryBranch Aeon choice increments its per-Aeon choice counter once; it does not directly change permanent Communing points without an authored increment row.",
  "Replace only if released structured evidence binds a branch choice directly to an AeonDimension point delta.",
);
const choiceEntries = (await context.table("RogueDLCMainStoryBranch"))
  .filter(({ row }) => row.AeonID !== undefined);
const choices = choiceEntries.map((choice) => {
  const branchId = choice.row.MainStoryBranchID;
  const identity = pathIdentity(choice.row.AeonID);
  const storyStage = Math.floor(branchId / 100);
  return {
    ...common({
      id: `swarm-disaster.communing-choice.${branchId}`,
      kind: "CommuningChoice",
      nameEn: `${identity.name.en} Choice ${branchId}`,
      nameZh: `${identity.name.zh}选择 ${branchId}`,
      summaryEn:
        `Story-stage ${storyStage} branch records one ${identity.name.en} alignment choice through NPC ${choice.row.RogueNPCID}.`,
      summaryZh:
        `故事阶段 ${storyStage} 分支通过 NPC ${choice.row.RogueNPCID} 记录一次${identity.name.zh}倾向选择。`,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(choice),
        context.sourceRef(identity.display),
        choicePolicy,
      ],
      tags: ["communing-choice", slug(identity.name.en), "project-policy"],
    }),
    source_id: String(branchId),
    story_stage: storyStage,
    aeon_id: String(choice.row.AeonID),
    path_id: identity.pathId,
    eligibility: {
      branch_available: true,
      story_stage: storyStage,
    },
    point_deltas: [],
    ordered_operations: [{
      order: 0,
      operation: "IncrementAeonChoiceCounter",
      counter_id: `swarm-disaster.aeon-choice-counter.${choice.row.AeonID}`,
      delta: "1",
      once_scope: `MainStoryBranch:${branchId}`,
    }],
    rogue_npc_id: String(choice.row.RogueNPCID),
  };
});
outputs.set("communing-choices.json", ordered(
  choices,
  ["story_stage", "aeon_id", "id"],
));

const cabinetEntries = await context.table("RogueDLCAeonCabinet");
const prerequisites = new Map(cabinetEntries.map(({ row }) => [
  row.CabinetID,
  [],
]));
for (const cabinet of cabinetEntries)
  for (const unlockedId of cabinet.row.UnlockCabinetID ?? []) {
    if (!prerequisites.has(unlockedId))
      throw new Error(`cabinet ${cabinet.row.CabinetID} unlocks missing ${unlockedId}`);
    prerequisites.get(unlockedId).push(cabinet.row.CabinetID);
  }
for (const values of prerequisites.values())
  values.sort((left, right) => left - right);
const cabinetPolicy = await context.policyRef(
  "pathstrider-cabinets",
  "Interpret UnlockCabinetID as outgoing unlock edges and invert them into prerequisite IDs. Apply authored point increments in source-list order, clamping each dimension to its released maximum.",
  "Replace edge direction or simultaneous increment order if released engine evidence demonstrates different semantics.",
);
const cabinets = cabinetEntries.map((cabinet) => {
  const cabinetId = cabinet.row.CabinetID;
  const name = localized(
    cabinet.row.CabinetName,
    `Pathstrider Cabinet ${cabinetId}`,
    `行者之道柜节点 ${cabinetId}`,
  );
  const description = localized(
    cabinet.row.CabinetMissionDesc,
    `Complete objective ${cabinet.row.QuestID}.`,
    `完成目标 ${cabinet.row.QuestID}。`,
  );
  return {
    ...common({
      id: `swarm-disaster.pathstrider-cabinet.${cabinetId}`,
      kind: "PathstriderCabinet",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: description.en,
      summaryZh: description.zh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(cabinet), cabinetPolicy],
      tags: ["pathstrider-cabinet", cabinet.row.CabinetType, "project-policy"],
    }),
    source_id: String(cabinetId),
    sort: cabinet.row.Sort,
    cabinet_type: cabinet.row.CabinetType,
    prerequisite_ids: prerequisites.get(cabinetId).map((id) =>
      `swarm-disaster.pathstrider-cabinet.${id}`),
    unlocks_cabinet_ids: (cabinet.row.UnlockCabinetID ?? []).map((id) =>
      `swarm-disaster.pathstrider-cabinet.${id}`),
    objective_id: String(cabinet.row.QuestID),
    point_deltas: (cabinet.row.FinishAeonDimensionPointList ?? [])
      .map(({ DimensionID: dimensionId, Increments: increment }) => ({
        dimension_id: `swarm-disaster.communing-dimension.${dimensionId}`,
        delta: String(increment),
      })),
    description_parameters: (cabinet.row.DescParam ?? []).map(String),
  };
});
outputs.set("pathstrider-cabinets.json", ordered(
  cabinets,
  ["sort", "id"],
));

const dimensionPolicy = await context.policyRef(
  "communing-dimensions",
  "Communing dimensions are persistent account progression, clamp after each ordered increment to the released maximum, and do not decrease at run boundaries.",
  "Replace carry or clamp timing if released persistence evidence establishes a different lifecycle.",
);
const dimensionEntries = await context.table("RogueDLCAeonDimension");
const dimensions = dimensionEntries.map((dimension) => {
  const dimensionId = dimension.row.AeonDimensionID;
  const identity = pathIdentity(dimensionId);
  const description = localized(
    dimension.row.PlayShortDesc,
    `${identity.name.en} Communing dimension.`,
    `${identity.name.zh}觐见维度。`,
  );
  return {
    ...common({
      id: `swarm-disaster.communing-dimension.${dimensionId}`,
      kind: "CommuningDimension",
      nameEn: `${identity.name.en} Communing Dimension`,
      nameZh: `${identity.name.zh}觐见维度`,
      summaryEn: description.en,
      summaryZh: description.zh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        context.sourceRef(dimension),
        context.sourceRef(identity.display),
        dimensionPolicy,
      ],
      tags: ["communing-dimension", slug(identity.name.en), "project-policy"],
    }),
    source_id: String(dimensionId),
    path_id: identity.pathId,
    max_points: String(dimension.row.AeonDimensionMaxPoint),
    carry_policy: "PersistentAcrossRuns",
    clamp_policy: "ClampAfterEachOrderedIncrement",
  };
});
outputs.set("communing-dimensions.json", ordered(dimensions));

const adjustments = [];
for (const cabinet of cabinetEntries)
  for (const [ordinal, point] of (
    cabinet.row.FinishAeonDimensionPointList ?? []
  ).entries()) {
    const cabinetId = cabinet.row.CabinetID;
    adjustments.push({
      ...common({
        id:
          `swarm-disaster.communing-adjustment.cabinet.${cabinetId}.${ordinal}`,
        kind: "CommuningPointAdjustment",
        nameEn:
          `Cabinet ${cabinetId} Dimension ${point.DimensionID} Adjustment`,
        nameZh:
          `柜节点 ${cabinetId} 维度 ${point.DimensionID} 调整`,
        summaryEn:
          `Completing cabinet ${cabinetId} adds ${point.Increments} point(s) to Communing dimension ${point.DimensionID}.`,
        summaryZh:
          `完成柜节点 ${cabinetId} 后，为觐见维度 ${point.DimensionID} 增加 ${point.Increments} 点。`,
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [context.sourceRef(cabinet), cabinetPolicy],
        tags: ["communing-adjustment", "cabinet", "project-policy"],
      }),
      source_kind: "PathstriderCabinet",
      source_id: String(cabinetId),
      ordinal,
      dimension_id: `swarm-disaster.communing-dimension.${point.DimensionID}`,
      delta: String(point.Increments),
      clamp_policy: "ClampToDimensionMaximumAfterOperation",
      operation_order: ordinal,
    });
  }
outputs.set("communing-point-adjustments.json", ordered(
  adjustments,
  ["source_kind", "source_id", "ordinal"],
));

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster Communing ${check ? "verified" : "generated"}: ` +
  `${choices.length} choices, ${cabinets.length} cabinets, ` +
  `${dimensions.length} dimensions and ${adjustments.length} adjustments.`,
);
