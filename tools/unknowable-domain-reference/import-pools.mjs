#!/usr/bin/env node

import crypto from "node:crypto";
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
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const manifest = JSON.parse(await fs.readFile(path.join(
  root,
  "content-manifests/unknowable-domain-v1/content-manifest.json",
), "utf8"));
const components = JSON.parse(await fs.readFile(path.join(
  root,
  "content-reference/unknowable-domain-v1/components.json",
), "utf8")).filter(({ kind }) => kind === "Component");

const blessings = [];
const memberships = [
  ...components.map((component) => ({
    ...context.envelope({
      id: `unknowable-domain.pool-membership.component.${component.source_id}`,
      kind: "UnknowablePoolMembership",
      nameEn: `Mode Component Catalog Member ${component.source_id}`,
      nameZh: `玩法组件目录成员 ${component.source_id}`,
      summaryEn:
        `Component ${component.source_id} is mode-owned catalog content; no ` +
        "released Alignment offer selector or weight is claimed.",
      summaryZh:
        `组件 ${component.source_id} 是玩法专属目录内容；不声称存在已发布的` +
        "倾向提供选择器或权重。",
      sourceRefs: component.source_refs,
      tags: ["catalog-only", "component", "pool-membership"],
    }),
    source_id: `component:${component.source_id}`,
    pool_id: "unknowable-domain.pool.components.mode-owned-catalog",
    member_kind: "Component",
    member_id: component.id,
    eligibility: "CatalogOwnedNotOfferProven",
    weight: "Unspecified",
    alignment_ids: [],
    alignment_resolution: "Unspecified",
    reachability_proof: "DirectModeOwnership",
  })),
  ...await sharedMemberships({
    category: "curios",
    memberKind: "Curio",
    poolId: "unknowable-domain.pool.curios.type-260",
    stablePrefix: "unknowable-domain.curio",
    nameEn: "Shared Curio",
    nameZh: "共享奇物",
  }),
  ...await sharedMemberships({
    category: "occurrences",
    memberKind: "Occurrence",
    poolId: "unknowable-domain.pool.occurrences.type-260",
    stablePrefix: "unknowable-domain.occurrence",
    nameEn: "Shared Occurrence",
    nameZh: "共享事件",
  }),
].sort(compareIds);

await writeOrCheck(
  context,
  new Map([
    ["blessings.json", blessings],
    ["pool-membership.json", memberships],
  ]),
  check,
);
console.log(
  `Unknowable Domain pools ${check ? "verified" : "generated"}: ` +
  "0 Blessings, 109 catalog-only Components, 60 explicit type-260 Curios, " +
  "and 62 explicit type-260 Occurrences.",
);

async function sharedMemberships({
  category,
  memberKind,
  poolId,
  stablePrefix,
  nameEn,
  nameZh,
}) {
  const result = [];
  for (const record of manifest.categories[category].records) {
    const entry = await sourceEntry(record.source);
    const evidenceSha256 = crypto.createHash("sha256")
      .update(JSON.stringify(entry.row))
      .digest("hex");
    if (evidenceSha256 !== record.evidence_sha256)
      throw new Error(`${category}:${record.id} evidence digest drift`);
    const sourceRef = context.sourceRef(
      entry,
      "ExactStructured",
      { sha256: evidenceSha256 },
    );
    result.push({
      ...context.envelope({
        id:
          `unknowable-domain.pool-membership.${memberKind.toLowerCase()}.` +
          `${record.id}`,
        kind: "UnknowablePoolMembership",
        nameEn: `${nameEn} ${record.id} Membership`,
        nameZh: `${nameZh} ${record.id} 成员关系`,
        summaryEn:
          `${nameEn} ${record.id} is reachable through the explicit released ` +
          "mode-type 260 selector; no weight is published.",
        summaryZh:
          `${nameZh} ${record.id} 通过明确发布的玩法类型 260 选择器可达；` +
          "未发布权重。",
        ownership: "Shared",
        sourceRefs: [sourceRef],
        tags: ["explicit-type-260", "pool-membership", memberKind.toLowerCase()],
      }),
      source_id: `${memberKind.toLowerCase()}:${record.id}`,
      pool_id: poolId,
      member_kind: memberKind,
      member_id: `${stablePrefix}.${record.id}`,
      eligibility: "ExactReachable",
      weight: "Unspecified",
      alignment_ids: [],
      alignment_resolution: "NotApplicable",
      reachability_proof: "ExplicitModeType260",
    });
  }
  return result;
}
async function sourceEntry(source) {
  const match = /^(.*)#([0-9]+)$/u.exec(source);
  if (!match) throw new Error(`invalid manifest source ${source}`);
  const [, sourcePath, locator] = match;
  // Match the frozen manifest's raw JSON.parse digest contract exactly. The
  // normalized import reader preserves oversized TextMap hashes as strings,
  // while the manifest intentionally digests the released source row as read.
  const rows = JSON.parse(await fs.readFile(
    path.join(context.sourceRoot, sourcePath),
    "utf8",
  ));
  const row = rows[Number(locator)];
  if (!row) throw new Error(`missing manifest source ${source}`);
  return { sourcePath, locator, row };
}
function compareIds(left, right) {
  return left.id < right.id ? -1 : left.id > right.id ? 1 : 0;
}
