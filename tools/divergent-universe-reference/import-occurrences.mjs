#!/usr/bin/env node

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
const manifest = JSON.parse(await import("node:fs/promises").then(({ readFile }) =>
  readFile(
    path.join(
      root,
      "content-manifests/divergent-universe-v1/content-manifest.json",
    ),
    "utf8",
  )));
const occurrenceIds = new Set(
  manifest.categories.occurrences.records.map((row) => row.id),
);
const variantSourceIds = new Set(
  manifest.categories.occurrence_variants.records.map((row) => row.id),
);
const handbookEntries = (await context.table("RogueTournHandBookEvent"))
  .filter(({ row }) => occurrenceIds.has(String(row.EventHandbookID)));
const npcEntries = (await context.table("RogueTournNPC"))
  .filter(({ row }) => variantSourceIds.has(String(row.RogueNPCID)));
const npcById = new Map(npcEntries.map((entry) =>
  [String(entry.row.RogueNPCID), entry]));
const missingGraphPolicy = await context.policyRef(
  "occurrence-graphs-missing",
  "All 97 selected Tourn3 NPC rows publish a RogueNPC_410 JSON path, but none of those paths exists in the pinned turnbasedgamedata Git tree. Choice conditions, costs, outcomes, ordering and hidden weights therefore remain empty and fail closed.",
  "Replace each missing-graph boundary only when a released source revision contains the exact published path or another released table explicitly binds the variant to its mechanical option graph.",
);

const variantToHandbooks = new Map();
const handbookLinks = new Map();
for (const handbook of handbookEntries) {
  const handbookId = String(handbook.row.EventHandbookID);
  const links = [];
  for (const [ordinal, opaque] of (
    handbook.row.UnlockNPCProgressIDList ?? []
  ).entries())
    for (const [field, value] of Object.entries(opaque)) {
      const variantId = String(value);
      if (!variantSourceIds.has(variantId)) continue;
      links.push({
        ordinal,
        source_field: field,
        variant_id:
          `divergent-universe.occurrence-variant.${variantId}`,
      });
      if (!variantToHandbooks.has(variantId))
        variantToHandbooks.set(variantId, new Set());
      variantToHandbooks.get(variantId).add(handbookId);
    }
  if (links.length === 0)
    throw new Error(`Occurrence ${handbookId} has no Tourn3 variant`);
  handbookLinks.set(handbookId, links);
}
if (variantToHandbooks.size !== variantSourceIds.size)
  throw new Error("not every Tourn3 variant has a handbook reference");

const occurrences = handbookEntries.map((handbook) => {
  const sourceId = String(handbook.row.EventHandbookID);
  const links = handbookLinks.get(sourceId);
  const variantIds = [...new Set(links.map((link) => link.variant_id))].sort();
  const nameEn = context.text(handbook.row.EventTitle, "en")
    || `Occurrence ${sourceId}`;
  const nameZh = context.text(handbook.row.EventTitle, "zh_cn")
    || `事件 ${sourceId}`;
  return {
    ...context.envelope({
      id: `divergent-universe.occurrence.${sourceId}`,
      kind: "DivergentUniverseOccurrence",
      nameEn,
      nameZh,
      summaryEn:
        `${nameEn} is a released Tourn3 handbook Occurrence with ${variantIds.length} exact current variant binding(s); its option graph is absent from the fixed source tree.`,
      summaryZh:
        `${nameZh} 是已发布的 Tourn3 图鉴事件，具有 ${variantIds.length} 个精确当前变体绑定；其选项图未包含在固定源树中。`,
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [context.sourceRef(handbook), missingGraphPolicy],
      tags: ["occurrence", "tourn3", "graph-missing"],
    }),
    source_id: sourceId,
    handbook_priority: handbook.row.Priority,
    handbook_used: handbook.row.IsUsed === true,
    variant_ids: variantIds,
    unlock_rules: links,
    choice_ids: [],
    selection_policy: "OwningDomainOrServiceBindingRequired",
    unresolved_offer_behavior: "FailClosed",
    runtime_lowered: false,
  };
}).sort(compareIds);

const variants = [...variantSourceIds].map((sourceId) => {
  const npc = npcById.get(sourceId);
  if (!npc) throw new Error(`missing Tourn3 NPC ${sourceId}`);
  const handbooks = [...variantToHandbooks.get(sourceId)].sort(compare);
  const primary = handbookEntries.find(({ row }) =>
    String(row.EventHandbookID) === handbooks[0]);
  const nameEn = context.text(primary.row.EventTitle, "en")
    || `Occurrence variant ${sourceId}`;
  const nameZh = context.text(primary.row.EventTitle, "zh_cn")
    || `事件变体 ${sourceId}`;
  return {
    ...context.envelope({
      id: `divergent-universe.occurrence-variant.${sourceId}`,
      kind: "DivergentUniverseOccurrenceVariant",
      nameEn: `${nameEn} — Tourn3 Variant ${sourceId}`,
      nameZh: `${nameZh} — Tourn3 变体 ${sourceId}`,
      summaryEn:
        `Tourn3 NPC ${sourceId} binds ${handbooks.length} handbook identity/identities and publishes missing graph path ${npc.row.NPCJsonPath}.`,
      summaryZh:
        `Tourn3 NPC ${sourceId} 绑定 ${handbooks.length} 个图鉴身份，并发布缺失图路径 ${npc.row.NPCJsonPath}。`,
      coverageState: "Researched",
      evidenceQuality: "ProjectPolicy",
      sourceRefs: [
        ...handbooks.map((id) => context.sourceRef(
          handbookEntries.find(({ row }) =>
            String(row.EventHandbookID) === id),
        )),
        context.sourceRef(npc),
        missingGraphPolicy,
      ],
      tags: ["occurrence-variant", "tourn3", "graph-missing"],
    }),
    source_id: sourceId,
    occurrence_id:
      `divergent-universe.occurrence.${handbooks[0]}`,
    occurrence_ids: handbooks.map((id) =>
      `divergent-universe.occurrence.${id}`),
    graph_path: npc.row.NPCJsonPath,
    graph_resolution: "MissingAtPinnedRevision",
    entry_conditions: handbooks.map((id) => ({
      kind: "HandbookUnlockNPCProgress",
      occurrence_id: `divergent-universe.occurrence.${id}`,
    })),
    choice_ids: [],
    fallback: "RejectWithoutMutation",
    runtime_lowered: false,
  };
}).sort(compareIds);

await writeOrCheck(context, new Map([
  ["occurrences.json", occurrences],
  ["occurrence-variants.json", variants],
  ["occurrence-choices.json", []],
]), check);
console.log(
  `Divergent Universe Occurrences ${check ? "verified" : "generated"}: ` +
  `${occurrences.length} identities, ${variants.length} variants and zero ` +
  `choices because all published Tourn3 graph paths are absent.`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function compareIds(left, right) {
  return compare(left.id, right.id);
}
