#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const paths = await normalized("blessing-paths.json");
const blessings = await normalized("blessings.json");
const groups = await normalized("blessing-groups.json");
const levels = await normalized("blessing-levels.json");
const groupById = new Map(groups.map((row) => [row.id, row]));
const levelByTag = new Map(levels.map((row) => [row.rogue_buff_tag, row.id]));
const rows = [];

for (const pathRow of paths)
  rows.push(poolRow({
    id: `divergent-universe.pool-membership.active-path.${pathRow.source_id}`,
    nameEn: `${pathRow.name_en} Active Path Membership`,
    nameZh: `${pathRow.name_zh_cn}活动命途成员关系`,
    summaryEn:
      `${pathRow.name_en} is selected by the released Tourn3 active Blessing-type list.`,
    summaryZh:
      `${pathRow.name_zh_cn}由已发布的 Tourn3 活动祝福类型列表明确选中。`,
    sourceRefs: pathRow.source_refs,
    poolId: "divergent-universe.pool.active-blessing-paths",
    memberId: pathRow.id,
    basis: "ExplicitTourn3ActiveTypeSelector",
    edgeKind: "PathSelector",
    ordinal: Number(pathRow.path_type_id),
    proofGroupIds: [],
  }));

for (const blessing of blessings)
  rows.push(poolRow({
    id: `divergent-universe.pool-membership.path.${blessing.path_type_id}.blessing.${blessing.source_id}`,
    nameEn: `${blessing.name_en} Path Membership`,
    nameZh: `${blessing.name_zh_cn}命途成员关系`,
    summaryEn:
      `${blessing.name_en} belongs to active Tourn3 Path type ${blessing.path_type_id} through its explicit Blessing row.`,
    summaryZh:
      `${blessing.name_zh_cn}通过其明确祝福记录归属活动 Tourn3 命途类型 ${blessing.path_type_id}。`,
    sourceRefs: blessing.source_refs,
    poolId: blessing.path_id,
    memberId: blessing.id,
    basis: "ExplicitTourn3BlessingType",
    edgeKind: "PathBlessing",
    ordinal: Number(blessing.source_id),
    proofGroupIds: [],
  }));

for (const group of groups) {
  for (const [ordinal, sourceId] of group.source_candidate_ids.entries()) {
    const terminalId = terminalFor(group, sourceId);
    const subgroupId =
      `divergent-universe.blessing-group.${sourceId}`;
    const memberId = terminalId
      ?? (groupById.has(subgroupId) ? subgroupId : "");
    if (!memberId)
      throw new Error(`${group.id} cannot classify candidate ${sourceId}`);
    rows.push(poolRow({
      id: `${group.id}.direct.${String(ordinal).padStart(2, "0")}`,
      nameEn: `${group.name_en} Direct Member ${ordinal + 1}`,
      nameZh: `${group.name_zh_cn}直接成员 ${ordinal + 1}`,
      summaryEn:
        `${group.name_en} directly references ${terminalId ? "mode-owned Blessing level tag" : "Tourn3 subgroup"} ${sourceId} at ordinal ${ordinal + 1}.`,
      summaryZh:
        `${group.name_zh_cn}在第 ${ordinal + 1} 位直接引用${terminalId ? "玩法专属祝福等级标签" : "Tourn3 子组"} ${sourceId}。`,
      sourceRefs: group.source_refs,
      poolId: group.id,
      memberId,
      basis: "DirectStableIdReference",
      edgeKind: terminalId ? "TerminalLevel" : "NestedGroup",
      ordinal,
      proofGroupIds: [group.id],
    }));
  }

  for (const terminal of expand(group.id))
    rows.push(poolRow({
      id: `${group.id}.terminal.${terminal.memberId}`,
      nameEn: `${group.name_en} Terminal Blessing Membership`,
      nameZh: `${group.name_zh_cn}终端祝福成员关系`,
      summaryEn:
        `${group.name_en} reaches mode-owned Blessing level ${terminal.memberId} through ${terminal.proofGroupIds.length} Tourn3 group row(s).`,
      summaryZh:
        `${group.name_zh_cn}经 ${terminal.proofGroupIds.length} 条 Tourn3 组记录到达玩法专属祝福等级 ${terminal.memberId}。`,
      sourceRefs: terminal.sourceRefs,
      poolId: group.id,
      memberId: terminal.memberId,
      basis: "TransitiveStableIdClosure",
      edgeKind: "ExpandedTerminalLevel",
      ordinal: terminal.ordinal,
      proofGroupIds: terminal.proofGroupIds,
    }));
}

rows.sort((left, right) => left.id.localeCompare(right.id));
await writeOrCheck(context, new Map([["pool-membership.json", rows]]), check);
console.log(
  `Divergent Universe pools ${check ? "verified" : "generated"}: ` +
  `${rows.length.toLocaleString("en-US")} exact membership rows.`,
);

function terminalFor(group, sourceId) {
  const terminalId = levelByTag.get(sourceId);
  return group.resolved_mode_level_ids.includes(terminalId)
    ? terminalId
    : undefined;
}

function expand(rootId) {
  const found = new Map();
  visit(rootId, []);
  return [...found.values()].sort((left, right) =>
    left.memberId.localeCompare(right.memberId));

  function visit(groupId, pathIds) {
    if (pathIds.includes(groupId))
      throw new Error(`Blessing group cycle: ${[...pathIds, groupId].join(" -> ")}`);
    const group = groupById.get(groupId);
    if (!group) throw new Error(`missing subgroup ${groupId}`);
    const nextPath = [...pathIds, groupId];
    for (const [ordinal, sourceId] of group.source_candidate_ids.entries()) {
      const terminalId = terminalFor(group, sourceId);
      if (terminalId) {
        const prior = found.get(terminalId);
        const proofGroupIds = nextPath;
        if (!prior || proofGroupIds.length < prior.proofGroupIds.length)
          found.set(terminalId, {
            memberId: terminalId,
            ordinal,
            proofGroupIds,
            sourceRefs: uniqueRefs(proofGroupIds.flatMap((id) =>
              groupById.get(id).source_refs)),
          });
        continue;
      }
      visit(`divergent-universe.blessing-group.${sourceId}`, nextPath);
    }
  }
}

function poolRow({
  id,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  sourceRefs,
  poolId,
  memberId,
  basis,
  edgeKind,
  ordinal,
  proofGroupIds,
}) {
  return {
    ...context.envelope({
      id,
      kind: "DivergentUniversePoolMembership",
      nameEn,
      nameZh,
      summaryEn,
      summaryZh,
      sourceRefs,
      tags: ["blessing", "pool-membership", edgeKind],
    }),
    pool_id: poolId,
    member_id: memberId,
    membership_basis: basis,
    module_scope: "Tourn3/6002201",
    edge_kind: edgeKind,
    ordinal,
    proof_group_ids: proofGroupIds,
    runtime_lowered: false,
  };
}

function uniqueRefs(refs) {
  const found = new Map();
  for (const ref of refs)
    found.set(`${ref.path}#${ref.locator}#${ref.sha256}`, ref);
  return [...found.values()];
}

async function normalized(name) {
  return JSON.parse(await fs.readFile(path.join(context.outputRoot, name), "utf8"));
}

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
