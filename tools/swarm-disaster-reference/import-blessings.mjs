#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  ACCESS_DATE,
  canonical,
  createContext,
  sha256,
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

async function localRows(relative) {
  return JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
}
function localRef(relative, row, locator) {
  return {
    source_id: `source.goal09.inherited.${slug(relative)}.${slug(locator)}`,
    repository: "starclock",
    revision: "goal03-standard-universe-v1",
    path: relative,
    locator: String(locator),
    sha256: sha256(canonical(row)),
    access_date: ACCESS_DATE,
    evidence_quality: "ExactStructured",
  };
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

const poolPolicy = await context.policyRef(
  "shared-content-pool-weight",
  "Use stable member-ID order and equal integer weight 1 for selectable Path and Blessing candidates after exact eligibility filtering. Resonances and Formations unlock deterministically from the selected Path and inherited thresholds rather than random weighting.",
  "Replace equal candidate weights and selected-Path weighting when released pool tables or reproducible engine observations provide authoritative weights.",
);
const standardBlessingRelative =
  "content-reference/standard-universe-v1/blessings.json";
const standardLevelRelative =
  "content-reference/standard-universe-v1/blessing-levels.json";
const pathRelative = "content-reference/swarm-disaster-v1/paths.json";
const resonanceRelative =
  "content-reference/swarm-disaster-v1/resonances.json";
const standardBlessings = await localRows(standardBlessingRelative);
const standardLevels = await localRows(standardLevelRelative);
const paths = await localRows(pathRelative);
const resonances = await localRows(resonanceRelative);
const manifest = await localRows(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const buffEntries = await context.table("RogueBuff");
const mazeEntries = await context.table("RogueMazeBuff");

const pathById = new Map(paths.map((row, index) => [
  row.shared_path_id,
  { row, index },
]));
const blessingById = new Map(standardBlessings.map((row) => [row.id, row]));
const buffByMazeId = new Map();
for (const entry of buffEntries) {
  const key = String(entry.row.MazeBuffID);
  if (!buffByMazeId.has(key)) buffByMazeId.set(key, entry);
}
const mazeByIdAndLevel = new Map(mazeEntries.map((entry) => [
  `${entry.row.ID}:${entry.row.Lv}`,
  entry,
]));
const requiredBlessings = new Set(manifest.categories.blessings.records
  .map(({ id }) => id));
const requiredLevels = new Set(manifest.categories.blessing_levels.records
  .map(({ id }) => id));

const blessings = standardBlessings
  .filter(({ id }) => requiredBlessings.has(id))
  .map((standardRow, index) => {
    const sourceId = String(standardRow.source_ids[0]);
    const buff = buffByMazeId.get(sourceId);
    const pathBinding = pathById.get(standardRow.path_id);
    if (!buff || !pathBinding)
      throw new Error(`incomplete shared Blessing ${standardRow.id}`);
    return {
      ...context.envelope({
        id: `swarm-disaster.blessing-binding.${sourceId}`,
        kind: "SwarmBlessingBinding",
        nameEn: standardRow.name_en,
        nameZh: standardRow.name_zh_cn,
        summaryEn:
          `Swarm Disaster inherits this ${standardRow.rarity}-star ${pathBinding.row.name_en} Blessing and both authored levels unchanged.`,
        summaryZh:
          `寰宇蝗灾原样继承该${standardRow.rarity}星${pathBinding.row.name_zh_cn}祝福及两个配置等级。`,
        ownership: "Shared",
        evidenceQuality: "ProjectPolicy",
        sourceRefs: [
          localRef(standardBlessingRelative, standardRow, index),
          context.sourceRef(buff),
          localRef(pathRelative, pathBinding.row, pathBinding.index),
          poolPolicy,
        ],
        tags: [
          "blessing-binding",
          `rarity-${standardRow.rarity}`,
          "shared",
          "project-policy",
        ],
      }),
      source_id: sourceId,
      shared_blessing_id: standardRow.id,
      path_id: standardRow.path_id,
      rarity: String(standardRow.rarity),
      level_ids: standardRow.level_ids.map((id) =>
        `swarm-disaster.blessing-level-binding.${id
          .replace(/^universe\.blessing\.([0-9]+)\.level\.([0-9]+)$/u,
            "$1.$2")}`),
      prerequisite_ids: standardRow.prerequisite_ids,
      pool_rules: {
        pool_id: "swarm-disaster.pool.blessings",
        eligibility: "EnabledSharedBlessingAndModeReachablePath",
        candidate_order: "StableMemberIdAscending",
        base_integer_weight: "1",
        selected_path_weight: "UnresolvedNoAdditionalWeight",
      },
      extra_effect_source_ids: standardRow.extra_effect_source_ids,
      inherited_rule_ids: standardRow.rule_ids,
      mechanic_tags: standardRow.mechanic_tags,
      source_description_sha256_en:
        standardRow.source_description_sha256_en,
      source_description_sha256_zh_cn:
        standardRow.source_description_sha256_zh_cn,
    };
  });
outputs.set(
  "blessings.json",
  ordered(blessings, ["path_id", "rarity", "id"]),
);
const blessingBindingBySharedId = new Map(blessings.map((row) => [
  row.shared_blessing_id,
  row.id,
]));

const levels = standardLevels
  .filter(({ id }) => requiredLevels.has(id))
  .map((standardRow, index) => {
    const blessing = blessingById.get(standardRow.blessing_id);
    const bindingId = blessingBindingBySharedId.get(standardRow.blessing_id);
    const pathBinding = pathById.get(blessing?.path_id);
    const sourceId = String(standardRow.source_ids[0]);
    const maze = mazeByIdAndLevel.get(`${sourceId}:${standardRow.level}`);
    if (!blessing || !bindingId || !pathBinding || !maze)
      throw new Error(`incomplete Blessing level ${standardRow.id}`);
    return {
      ...context.envelope({
        id: `swarm-disaster.blessing-level-binding.${sourceId}.${standardRow.level}`,
        kind: "SwarmBlessingLevelBinding",
        nameEn: standardRow.name_en,
        nameZh: standardRow.name_zh_cn,
        summaryEn:
          `Swarm Disaster inherits authored level ${standardRow.level} parameters and binding for ${blessing.name_en}.`,
        summaryZh:
          `寰宇蝗灾继承${blessing.name_zh_cn}已配置的等级${standardRow.level}参数与绑定。`,
        ownership: "Shared",
        sourceRefs: [
          localRef(standardLevelRelative, standardRow, index),
          context.sourceRef(maze),
          localRef(pathRelative, pathBinding.row, pathBinding.index),
        ],
        tags: [
          "blessing-level-binding",
          `level-${standardRow.level}`,
          "shared",
        ],
      }),
      source_id: `${sourceId}:${standardRow.level}`,
      shared_blessing_level_id: standardRow.id,
      blessing_id: bindingId,
      shared_blessing_id: standardRow.blessing_id,
      level: String(standardRow.level),
      parameter_values: standardRow.parameter_values,
      inherited_rule_ids: standardRow.rule_ids,
      effect_program: {
        modifier_name: standardRow.source_modifier_name,
        binding_type: standardRow.source_binding_type,
        binding_key: standardRow.source_binding_key,
        maze_buff_type: standardRow.source_maze_buff_type,
      },
      source_description_sha256_en:
        standardRow.source_description_sha256_en,
      source_description_sha256_zh_cn:
        standardRow.source_description_sha256_zh_cn,
    };
  });
outputs.set(
  "blessing-levels.json",
  ordered(levels, ["blessing_id", "level", "id"]),
);

function membership({
  poolId,
  memberKind,
  memberId,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  sourceRef,
  eligibility,
  weightPolicy,
}) {
  return {
    ...context.envelope({
      id: `swarm-disaster.pool-membership.${slug(poolId)}.${slug(memberId)}`,
      kind: "SwarmPoolMembership",
      nameEn,
      nameZh,
      summaryEn,
      summaryZh,
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [sourceRef, poolPolicy],
      tags: ["pool-membership", slug(memberKind), "project-policy"],
    }),
    pool_id: poolId,
    member_kind: memberKind,
    member_id: memberId,
    eligibility,
    weight_policy: weightPolicy,
  };
}

const memberships = [];
for (const [index, pathRow] of paths.entries())
  memberships.push(membership({
    poolId: "swarm-disaster.pool.paths",
    memberKind: "Path",
    memberId: pathRow.shared_path_id,
    nameEn: `${pathRow.name_en} Path Pool Membership`,
    nameZh: `${pathRow.name_zh_cn}命途池成员`,
    summaryEn:
      `${pathRow.name_en} is one of eight selectable Swarm Disaster Paths.`,
    summaryZh:
      `${pathRow.name_zh_cn}是寰宇蝗灾八个可选命途之一。`,
    sourceRef: localRef(pathRelative, pathRow, index),
    eligibility: {
      rule: "PathSelectableAndModeUnlockSatisfied",
      unlock_id: pathRow.mode_unlock_id,
    },
    weightPolicy: {
      selection: "SeededUniformIntegerWeight",
      integer_weight: "1",
      candidate_order: "StableMemberIdAscending",
    },
  }));
for (const [index, resonance] of resonances.entries())
  memberships.push(membership({
    poolId: `swarm-disaster.pool.resonances.${slug(resonance.path_id)}`,
    memberKind: resonance.kind,
    memberId: resonance.shared_resonance_id,
    nameEn: `${resonance.name_en} Pool Membership`,
    nameZh: `${resonance.name_zh_cn}池成员`,
    summaryEn:
      `${resonance.name_en} unlocks deterministically for its selected Path.`,
    summaryZh:
      `${resonance.name_zh_cn}按所选命途确定性解锁。`,
    sourceRef: localRef(resonanceRelative, resonance, index),
    eligibility: {
      rule: "SelectedPathAndInheritedThreshold",
      path_id: resonance.path_id,
      threshold: resonance.threshold,
    },
    weightPolicy: {
      selection: "DeterministicThresholdUnlock",
      integer_weight: "0",
      candidate_order: "NotApplicable",
    },
  }));
for (const blessing of blessings)
  memberships.push(membership({
    poolId: "swarm-disaster.pool.blessings",
    memberKind: "Blessing",
    memberId: blessing.shared_blessing_id,
    nameEn: `${blessing.name_en} Pool Membership`,
    nameZh: `${blessing.name_zh_cn}池成员`,
    summaryEn:
      `${blessing.name_en} is eligible through the released ${blessing.path_id} shared pool.`,
    summaryZh:
      `${blessing.name_zh_cn}通过已发布的${blessing.path_id}共享池进入候选集。`,
    sourceRef: blessing.source_refs[0],
    eligibility: {
      rule: "EnabledSharedBlessingAndModeReachablePath",
      path_id: blessing.path_id,
      rarity: blessing.rarity,
    },
    weightPolicy: {
      selection: "SeededUniformIntegerWeight",
      integer_weight: "1",
      candidate_order: "StableMemberIdAscending",
      selected_path_weight: "UnresolvedNoAdditionalWeight",
    },
  }));
outputs.set(
  "pool-membership.json",
  ordered(memberships, ["pool_id", "member_kind", "member_id"]),
);

await writeOrCheck(context, outputs, check);
console.log(
  `Swarm Disaster Blessings ${check ? "verified" : "generated"}: ` +
  `${blessings.length} Blessings, ${levels.length} levels and ` +
  `${memberships.length} pool memberships.`,
);
