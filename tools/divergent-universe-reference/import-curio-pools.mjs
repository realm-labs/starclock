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
const states = await normalized("curio-states.json");
const policy = await context.policyRef(
  "curio-offer-pools",
  "Tourn3 mode-copy rows prove the complete category catalog, while RogueTournMiracleGroup publishes no members or weights. Catalog membership is not an offer-eligibility claim.",
  "Replace each unspecified offer weight and eligibility only when a released selector exposes exact group members, weights, exclusions and fallback.",
);

const rows = states.map((state) => ({
  ...context.envelope({
    id: `divergent-universe.curio-catalog-membership.${state.source_id}`,
    kind: "DivergentUniverseCurioPoolMembership",
    nameEn: `${state.name_en} Tourn3 Catalog Membership`,
    nameZh: `${state.name_zh_cn} Tourn3 目录成员关系`,
    summaryEn:
      `Mode copy ${state.source_id} is an explicit Tourn3 ${state.category} catalog member; offer-specific membership and weight are not published.`,
    summaryZh:
      `玩法副本 ${state.source_id} 是明确的 Tourn3 ${state.category} 目录成员；特定报价成员关系与权重未发布。`,
    coverageState: "Researched",
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [state.source_refs[0], policy],
    tags: ["curio", "catalog-membership", `category-${state.category}`],
  }),
  pool_id:
    `divergent-universe.curio-catalog.${state.category.toLowerCase()}`,
  curio_state_id: state.id,
  weight: "Unspecified",
  eligibility: "Tourn3CatalogOnly;OfferSpecificEligibilityUnspecified",
  membership_basis: "ExplicitTourn3ModeAndCategory",
  source_group_ids: [],
  runtime_lowered: false,
})).sort((left, right) => left.id.localeCompare(right.id));

await writeOrCheck(
  context,
  new Map([["curio-pool-membership.json", rows]]),
  check,
);
console.log(
  `Divergent Universe Curio pools ${check ? "verified" : "generated"}: ` +
  `${rows.length} fail-closed catalog memberships.`,
);

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
