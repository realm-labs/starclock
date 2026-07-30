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
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const context = await createContext(root);
const functionEntries = await context.table("RogueMagicWorkbenchFunc");
const functionByType = new Map(functionEntries.map((entry) => [
  entry.row.FuncType,
  entry,
]));
const policyNote =
  "Reference-only deterministic transformation policy. Released Version 4.4 " +
  "text proves the function class but not input counts, exact costs, candidate " +
  "sets, ordering, failure behavior, or transition atomicity.";
const replacementCondition =
  "Replace field by field when a released transformation program or " +
  "reproducible observation proves exact inputs, costs, pools, selection " +
  "ordering, caps, mutation order, and failure behavior.";
const policyRef = await context.policyRef(
  "component-transformation-policy-v1",
  policyNote,
  replacementCondition,
);

const compose = requireFunction("MagicUnitCompose");
const reforge = requireFunction("MagicUnitReforge");
const upgrade = requireFunction("MagicScepterLevelUp");
const synthesisRules = [{
  ...policyEnvelope({
    id: "unknowable-domain.synthesis.component",
    kind: "ComponentSynthesisRule",
    nameEn: "Component Synthesis Policy",
    nameZh: "组件合成策略",
    summaryEn:
      "The released function combines Components into a new Component; exact " +
      "inputs, output pool and cost remain unspecified, with deterministic " +
      "nonmutation on failure.",
    summaryZh:
      "已发布功能将组件合成为新组件；精确输入、输出池与费用仍未指定，失败时" +
      "采用确定性不变更策略。",
    entry: compose,
    tags: ["component", "synthesis", "transformation"],
  }),
  source_id: String(compose.row.FuncID),
  function_type: compose.row.FuncType,
  input_count: "Unspecified",
  input_eligibility: ["Component"],
  input_level_relation: "Unspecified",
  output_pool: [],
  output_pool_resolution: "Unspecified",
  output_ordering: "StableComponentIdAscending",
  cost: {
    currency_id: "Unspecified",
    amount: "Unspecified",
  },
  fallback: "ReturnNoLegalCandidateWithoutMutation",
  policy_id: "component-transformation-policy-v1",
}];

const upgradeRules = [
  { input: "1", output: "2" },
  { input: "2", output: "3" },
].map(({ input, output }) => ({
  ...policyEnvelope({
    id: `unknowable-domain.scepter-upgrade.${input}-to-${output}`,
    kind: "ScepterUpgradeRule",
    nameEn: `Scepter Level ${input} to ${output} Upgrade Policy`,
    nameZh: `权杖等级 ${input} 至 ${output} 升级策略`,
    summaryEn:
      `The policy advances a released Scepter level from ${input} to ${output}; ` +
      "the exact Cosmic Fragment amount and mutation order await stronger evidence.",
    summaryZh:
      `该策略将已发布权杖等级从 ${input} 提升至 ${output}；精确宇宙碎片数量` +
      "与变更顺序等待更强证据。",
    entry: upgrade,
    tags: ["scepter", "transformation", "upgrade"],
  }),
  source_id: `${upgrade.row.FuncID}:${input}:${output}`,
  function_type: upgrade.row.FuncType,
  input_level: input,
  output_level: output,
  cost: {
    currency_id: "cosmic-fragments",
    amount: "Unspecified",
  },
  cap: "3",
  ordered_operations: [
    "ValidateOwnedScepterLevel",
    "ResolveUnspecifiedCost",
    "AdvanceExactlyOneReleasedLevel",
  ],
  fallback: "RejectWithoutMutation",
  policy_id: "component-transformation-policy-v1",
}));

const reforgeRules = [{
  ...policyEnvelope({
    id: "unknowable-domain.reforge.component",
    kind: "ComponentReforgeRule",
    nameEn: "Component Overwrite Policy",
    nameZh: "组件覆写策略",
    summaryEn:
      "The released function overwrites a Component into another Component; " +
      "the eligible pool and cost remain unspecified, with stable-ID ordering " +
      "and deterministic nonmutation when no candidate exists.",
    summaryZh:
      "已发布功能将组件覆写为其他组件；合法池与费用仍未指定，无候选时采用" +
      "稳定 ID 顺序与确定性不变更策略。",
    entry: reforge,
    tags: ["component", "reforge", "transformation"],
  }),
  source_id: String(reforge.row.FuncID),
  function_type: reforge.row.FuncType,
  input_eligibility: ["Component"],
  candidate_set: [],
  candidate_set_resolution: "Unspecified",
  exclude_input_identity: "Unspecified",
  ordering: "StableComponentIdAscending",
  cost: {
    currency_id: "Unspecified",
    amount: "Unspecified",
  },
  fallback: "ReturnNoLegalCandidateWithoutMutation",
  policy_id: "component-transformation-policy-v1",
}];

await writeOrCheck(
  context,
  new Map([
    ["synthesis-rules.json", synthesisRules],
    ["upgrade-rules.json", upgradeRules],
    ["reforge-rules.json", reforgeRules],
  ]),
  check,
);
console.log(
  `Unknowable Domain transformations ${check ? "verified" : "generated"}: ` +
  `${synthesisRules.length} synthesis, ${upgradeRules.length} upgrade, and ` +
  `${reforgeRules.length} reforge rule(s), all explicitly policy-bound.`,
);

function requireFunction(type) {
  const entry = functionByType.get(type);
  if (!entry) throw new Error(`missing workbench function ${type}`);
  return entry;
}
function policyEnvelope({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  entry,
  tags,
}) {
  return context.envelope({
    id,
    kind,
    nameEn,
    nameZh,
    summaryEn,
    summaryZh,
    evidenceQuality: "ProjectPolicy",
    sourceRefs: [context.sourceRef(entry), policyRef],
    tags,
  });
}
