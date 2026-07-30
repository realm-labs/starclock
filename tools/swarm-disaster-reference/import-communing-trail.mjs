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
function effectDomain(id) {
  if ([104, 204, 304, 404, 704].includes(id)) return "Activity";
  if ([504, 604].includes(id)) return "ActivityAndBattle";
  return "Battle";
}

const entries = await context.table("RogueDLCAeonTalent");
const byDimension = new Map();
for (const node of entries) {
  const dimensionId = node.row.AeonDimensionID;
  if (!byDimension.has(dimensionId)) byDimension.set(dimensionId, []);
  byDimension.get(dimensionId).push(node);
}
for (const nodes of byDimension.values())
  nodes.sort((left, right) =>
    left.row.UnlockAeonDimensionPoint - right.row.UnlockAeonDimensionPoint
      || left.row.AeonTalentID - right.row.AeonTalentID);

const graphPolicy = await context.policyRef(
  "communing-trail-prerequisites",
  "Within each Communing dimension, order nodes by released point threshold then stable Talent ID; each non-root node requires the immediately preceding node.",
  "Replace the derived predecessor edge if released structured graph data supplies an explicit prerequisite relation.",
);
const nodes = [];
const prerequisites = [];
const effects = [];
for (const [dimensionId, dimensionNodes] of [...byDimension.entries()]
  .sort(([left], [right]) => left - right)) {
  for (const [index, node] of dimensionNodes.entries()) {
    const talentId = node.row.AeonTalentID;
    const name = localized(
      node.row.EffectTitle,
      `Communing Trail ${talentId}`,
      `觐见行迹 ${talentId}`,
    );
    const description = localized(
      node.row.EffectDesc,
      `Apply gameplay effect ${talentId}.`,
      `应用玩法效果 ${talentId}。`,
    );
    const previous = index === 0 ? undefined : dimensionNodes[index - 1];
    const prerequisiteIds = previous
      ? [`swarm-disaster.communing-prerequisite.${talentId}.0`]
      : [];
    nodes.push({
      ...common({
        id: `swarm-disaster.communing-trail.${talentId}`,
        kind: "CommuningTrailNode",
        nameEn: name.en,
        nameZh: name.zh,
        summaryEn: description.en,
        summaryZh: description.zh,
        evidenceQuality: previous ? "ProjectPolicy" : "ExactStructured",
        sourceRefs: previous
          ? [context.sourceRef(node), context.sourceRef(previous), graphPolicy]
          : [context.sourceRef(node)],
        tags: [
          "communing-trail",
          node.row.IsImportant ? "important" : "incremental",
          ...(previous ? ["project-policy"] : []),
        ],
      }),
      source_id: String(talentId),
      dimension_id: `swarm-disaster.communing-dimension.${dimensionId}`,
      threshold: String(node.row.UnlockAeonDimensionPoint),
      prerequisite_ids: prerequisiteIds,
      effect_ids: (node.row.GamePlayEffectList ?? []).map((effectId) =>
        `swarm-disaster.communing-effect.${talentId}.${effectId}`),
      is_important: Boolean(node.row.IsImportant),
    });
    if (previous)
      prerequisites.push({
        ...common({
          id: `swarm-disaster.communing-prerequisite.${talentId}.0`,
          kind: "CommuningTrailPrerequisite",
          nameEn: `${name.en} Prerequisite`,
          nameZh: `${name.zh}先决条件`,
          summaryEn:
            `Dimension ${dimensionId} node ${talentId} follows node ${previous.row.AeonTalentID} at released threshold ${node.row.UnlockAeonDimensionPoint}.`,
          summaryZh:
            `维度 ${dimensionId} 节点 ${talentId} 在已发布阈值 ${node.row.UnlockAeonDimensionPoint} 时承接节点 ${previous.row.AeonTalentID}。`,
          evidenceQuality: "ProjectPolicy",
          sourceRefs: [
            context.sourceRef(node),
            context.sourceRef(previous),
            graphPolicy,
          ],
          tags: ["communing-prerequisite", "project-policy"],
        }),
        node_id: `swarm-disaster.communing-trail.${talentId}`,
        ordinal: 0,
        required_node_id:
          `swarm-disaster.communing-trail.${previous.row.AeonTalentID}`,
        required_points: String(node.row.UnlockAeonDimensionPoint),
      });
    for (const [ordinal, effectId] of (
      node.row.GamePlayEffectList ?? []
    ).entries()) {
      const domain = effectDomain(effectId);
      effects.push({
        ...common({
          id: `swarm-disaster.communing-effect.${talentId}.${effectId}`,
          kind: "CommuningTrailEffect",
          nameEn: name.en,
          nameZh: name.zh,
          summaryEn: description.en,
          summaryZh: description.zh,
          sourceRefs: [context.sourceRef(node)],
          tags: ["communing-effect", domain.toLowerCase()],
        }),
        node_id: `swarm-disaster.communing-trail.${talentId}`,
        ordinal,
        domain,
        ordered_operations: [{
          order: 0,
          operation: "ApplyReleasedGameplayEffect",
          effect_ref: `source-effect.${effectId}`,
          parameters: (node.row.EffectDescParamList ?? []).map(decimal),
        }],
        battle_projection: {
          enabled: domain !== "Activity",
          boundary: domain === "Activity"
            ? "NotApplicable"
            : "BattleSpecCreation",
          effect_ref: domain === "Activity" ? "" : `source-effect.${effectId}`,
        },
      });
    }
  }
}
outputs.set("communing-trail-nodes.json", ordered(
  nodes,
  ["dimension_id", "threshold", "id"],
));
outputs.set("communing-trail-prerequisites.json", ordered(
  prerequisites,
  ["node_id", "ordinal"],
));
outputs.set("communing-trail-effects.json", ordered(
  effects,
  ["node_id", "ordinal"],
));

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster Communing Trail ${check ? "verified" : "generated"}: ` +
  `${nodes.length} nodes, ${prerequisites.length} prerequisite edges and ` +
  `${effects.length} effects.`,
);
