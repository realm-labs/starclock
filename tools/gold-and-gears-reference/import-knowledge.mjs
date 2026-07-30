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
function parameters(values) {
  return (values ?? []).map(({ Value: value }) => decimal(value));
}
function textHash(reference) {
  return reference?.Hash === undefined ? "" : String(reference.Hash);
}

const semantics = new Map(Object.entries({
  2006: ["CopyCurrentDomainAndApplyKnowledge", "Immediate", "SelectedNonBossDomain", "Selected", "Apply"],
  2007: ["CopySelectedDomainToAdjacentAndApplyKnowledge", "Immediate", "AdjacentNonBossDomain", "Random", "Apply"],
  2010: ["CopySelectedDomainToPlaneAndApplyKnowledge", "Immediate", "RandomNonBossPlaneDomain", "Random", "Apply"],
  2017: ["CopyCurrentDomainToPlaneAndApplyKnowledge", "Immediate", "RandomNonBossPlaneDomain", "Random", "Apply"],
  2023: ["GenerateBeaconOnKnowledgeDomain", "Immediate", "RandomKnowledgeDomain", "Random", "Query"],
  2024: ["ApplyKnowledgeToUnmarkedDomains", "Immediate", "RandomUnmarkedPlaneDomain", "Random", "Apply"],
  2026: ["PropagateKnowledgePerKnowledgeDomain", "Immediate", "AdjacentDomainPerKnowledgeDomain", "RandomPerSource", "Apply"],
  2027: ["ApplyKnowledgeToUnmarkedDomains", "Immediate", "RandomUnmarkedPlaneDomain", "Random", "Apply"],
  2030: ["PropagateKnowledgeFromSelectedDomain", "Immediate", "AllAdjacentToSelectedKnowledgeDomain", "All", "Apply"],
  2031: ["ProtectCollapsingDomainsWithKnowledge", "AfterMovementBeforeCollapse", "RandomAboutToCollapseDomain", "Random", "Apply"],
  2032: ["ApplyKnowledgeAdjacentToCurrentDomain", "AfterMovement", "RandomAdjacentToCurrentDomain", "Random", "Apply"],
  2033: ["ApplyKnowledgeAdjacentToCurrentDomain", "AfterMovement", "AllAdjacentToCurrentDomain", "All", "Apply"],
  2034: ["ProtectCollapsingDomainsWithKnowledge", "AfterMovementBeforeCollapse", "AllAboutToCollapseDomains", "All", "Apply"],
  2035: ["RewardPerKnowledgeDomainType", "Immediate", "DistinctKnowledgeDomainTypes", "CountAll", "Query"],
  2041: ["ApplyKnowledgeAfterEnteringKnowledgeDomain", "OnEnterDuringMovement", "RandomPlaneDomain", "Random", "Apply"],
  2047: ["OverrideMovementToKnowledgeDomain", "DuringMovementSelection", "AnyKnowledgeDomain", "Selected", "Query"],
  2057: ["TransformKnowledgeDomainToAdventure", "Immediate", "RandomNonBossKnowledgeDomain", "Random", "Query"],
  2073: ["PropagateKnowledgeFromSelectedDomain", "Immediate", "RandomAdjacentToSelectedKnowledgeDomain", "Random", "Apply"],
  2074: ["ApplyKnowledgeToSelectedDomain", "Immediate", "SelectedDomain", "Selected", "Apply"],
  2077: ["RemoveKnowledgeAndRewardPerRemoval", "Immediate", "SelectedDomainAndAllAdjacent", "All", "Remove"],
  2078: ["RewardPerKnowledgeDomain", "Immediate", "AllKnowledgeDomains", "CountAll", "Query"],
  2079: ["TransformToBlankAndPreserveKnowledge", "Immediate", "SelectedNonBlankNonBossKnowledgeDomain", "Selected", "Preserve"],
}));
const targetPolicyRef = await context.policyRef(
  "knowledge-target-selection",
  "Released face text proves candidate predicates and random/selected/all cardinality, but not engine RNG enumeration. Candidate domains are canonically ordered by stable node ID before seeded selection without replacement; an empty eligible set produces no effect.",
  "Replace when pinned engine code or structured data proves target enumeration, RNG draw order, replacement, or empty-set behavior.",
);
const resolutionPolicyRef = await context.policyRef(
  "knowledge-simultaneous-resolution",
  "Released text names movement and about-to-collapse boundaries but does not fully order simultaneous Knowledge mutations. The project resolves movement, after-movement face effects, Knowledge mutation, dice-specific callbacks, collapse, then rewards; ties use face and target stable IDs.",
  "Replace individual tiers when pinned engine evidence proves exact simultaneous-effect ordering.",
);
const targetPolicy = {
  policy_id: "knowledge-target-selection-v1",
  evidence_quality: "ProjectPolicy",
  candidate_order: "stable-node-id-ascending",
  random_selection: "seeded-without-replacement",
  selected_validation: "reject-outside-exact-selector",
  empty_candidate_behavior: "NoEffect",
  replacement_condition:
    "Replace when pinned engine evidence proves enumeration, RNG, replacement, or empty-set behavior.",
};
const resolutionPolicy = {
  policy_id: "knowledge-simultaneous-resolution-v1",
  evidence_quality: "ProjectPolicy",
  tiers: [
    "resolve-movement-destination",
    "resolve-after-movement-face-effects",
    "apply-knowledge-state-mutations",
    "apply-active-custom-dice-entry-or-collapse-callbacks",
    "resolve-domain-collapse",
    "award-derived-resources",
  ],
  tie_breakers: ["dice-face-id", "target-node-id"],
  replacement_condition:
    "Replace individual tiers when pinned engine evidence proves exact simultaneous-effect ordering.",
};
const customDiceInteractions = {
  countdown_dice_id: "gold-gears.custom-dice.301",
  countdown_behavior:
    "Entering a Knowledge domain recovers the released countdown amount while that dice is active.",
  collapse_prevention_dice_id: "gold-gears.custom-dice.302",
  collapse_prevention_behavior:
    "Knowledge domains do not collapse while that dice is active.",
  collapse_reward_dice_id: "gold-gears.custom-dice.303",
  collapse_reward_behavior:
    "A collapsing Knowledge domain awards the released Cosmic Fragment amount while that dice is active.",
  evidence_quality: "ExactStructured",
};
const customDiceEntries = new Map(
  (await context.table("RogueNousDiceBranch")).map((entry) => [
    String(entry.row.BranchID),
    entry,
  ]),
);
const interactionSources = ["301", "302", "303"].map((id) => {
  const entry = customDiceEntries.get(id);
  if (!entry) throw new Error(`missing Knowledge interaction dice ${id}`);
  return context.sourceRef(entry);
});

const faceEntries = (await context.table("RogueNousDiceSurface"))
  .filter(({ row }) => (row.TagList ?? []).includes("SpecialType"));
const rules = faceEntries.map((entry) => {
  const id = String(entry.row.SurfaceID);
  const semantic = semantics.get(id);
  if (!semantic) throw new Error(`Knowledge face ${id} has no typed semantic`);
  const [operation, triggerBoundary, targetScope, selectionMode, access] = semantic;
  const name = localized(
    entry.row.SurfaceName,
    `Knowledge Rule ${id}`,
    `知识规则 ${id}`,
  );
  const description = localized(
    entry.row.SurfaceDesc,
    `Released Knowledge effect for face ${id}.`,
    `骰面 ${id} 的已发布知识效果。`,
  );
  return {
    ...context.envelope({
      id: `gold-gears.knowledge-rule.${id}`,
      kind: "KnowledgeBinding",
      nameEn: name.en,
      nameZh: name.zh,
      summaryEn: description.en,
      summaryZh: description.zh,
      sourceRefs: [
        context.sourceRef(entry),
        ...interactionSources,
        targetPolicyRef,
        resolutionPolicyRef,
      ],
      tags: ["custom-dice", "knowledge"],
    }),
    source_id: id,
    dice_face_id: `gold-gears.dice-face.${id}`,
    operation,
    trigger_boundary: triggerBoundary,
    target_scope: targetScope,
    selection_mode: selectionMode,
    knowledge_access: access,
    parameters: parameters(entry.row.DescParam),
    activation_stage: entry.row.DiceActiveStage,
    effect_text_hash: textHash(entry.row.SurfaceDesc),
    target_policy: targetPolicy,
    simultaneous_resolution_policy: resolutionPolicy,
    custom_dice_interactions: customDiceInteractions,
  };
}).sort((left, right) =>
  left.dice_face_id.localeCompare(right.dice_face_id)
  || left.id.localeCompare(right.id));
outputs.set("knowledge-rules.json", rules);

await writeOrCheck(context, outputs, check);
console.log(
  `${check ? "Checked" : "Wrote"} 22 Knowledge face bindings with typed ` +
  "placement, propagation, consumption, movement, collapse, and reward scopes.",
);
