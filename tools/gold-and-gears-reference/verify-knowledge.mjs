#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const decimalPattern =
  /^(0|-?(?:[1-9][0-9]*(?:\.[0-9]*[1-9])?|0\.[0-9]*[1-9]))$/u;
const operations = new Set([
  "CopyCurrentDomainAndApplyKnowledge",
  "CopySelectedDomainToAdjacentAndApplyKnowledge",
  "CopySelectedDomainToPlaneAndApplyKnowledge",
  "CopyCurrentDomainToPlaneAndApplyKnowledge",
  "GenerateBeaconOnKnowledgeDomain",
  "ApplyKnowledgeToUnmarkedDomains",
  "PropagateKnowledgePerKnowledgeDomain",
  "PropagateKnowledgeFromSelectedDomain",
  "ProtectCollapsingDomainsWithKnowledge",
  "ApplyKnowledgeAdjacentToCurrentDomain",
  "RewardPerKnowledgeDomainType",
  "ApplyKnowledgeAfterEnteringKnowledgeDomain",
  "OverrideMovementToKnowledgeDomain",
  "TransformKnowledgeDomainToAdventure",
  "ApplyKnowledgeToSelectedDomain",
  "RemoveKnowledgeAndRewardPerRemoval",
  "RewardPerKnowledgeDomain",
  "TransformToBlankAndPreserveKnowledge",
]);
const triggerBoundaries = new Set([
  "Immediate",
  "AfterMovement",
  "AfterMovementBeforeCollapse",
  "OnEnterDuringMovement",
  "DuringMovementSelection",
]);
const targetScopes = new Set([
  "SelectedNonBossDomain",
  "AdjacentNonBossDomain",
  "RandomNonBossPlaneDomain",
  "RandomKnowledgeDomain",
  "RandomUnmarkedPlaneDomain",
  "AdjacentDomainPerKnowledgeDomain",
  "AllAdjacentToSelectedKnowledgeDomain",
  "RandomAboutToCollapseDomain",
  "RandomAdjacentToCurrentDomain",
  "AllAdjacentToCurrentDomain",
  "AllAboutToCollapseDomains",
  "DistinctKnowledgeDomainTypes",
  "RandomPlaneDomain",
  "AnyKnowledgeDomain",
  "RandomNonBossKnowledgeDomain",
  "RandomAdjacentToSelectedKnowledgeDomain",
  "SelectedDomain",
  "SelectedDomainAndAllAdjacent",
  "AllKnowledgeDomains",
  "SelectedNonBlankNonBossKnowledgeDomain",
]);
const selectionModes = new Set([
  "Selected", "Random", "RandomPerSource", "All", "CountAll",
]);
const knowledgeAccess = new Set(["Apply", "Query", "Remove", "Preserve"]);
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-knowledge.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const rules = json("content-reference/gold-and-gears-v1/knowledge-rules.json");
assert(Array.isArray(rules) && rules.length === 22,
  "Knowledge rule count drift");
assert(unique(rules.map(({ id }) => id)), "Knowledge rules contain duplicate IDs");
const faces = json("content-reference/gold-and-gears-v1/dice-faces.json");
const faceById = new Map(faces.map((face) => [face.id, face]));
const diceById = new Map(json(
  "content-reference/gold-and-gears-v1/dice-definitions.json",
).map((dice) => [dice.id, dice]));
const operationIds = new Map();
for (const rule of rules) {
  assert(rule.schema_revision === "starclock.gold-and-gears-row.v1",
    `${rule.id} row revision drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof rule[field] === "string" && rule[field].trim() !== "",
      `${rule.id} has empty ${field}`);
  assert(rule.ownership === "GoldAndGears"
    && rule.coverage_state === "DataReady"
    && rule.evidence_quality === "ExactStructured",
  `${rule.id} common envelope drift`);
  assert(JSON.stringify(rule.tags) === JSON.stringify([...rule.tags].sort()),
    `${rule.id} tags are not canonical`);
  assert(Array.isArray(rule.source_refs) && rule.source_refs.length === 6,
    `${rule.id} provenance count drift`);
  for (const source of rule.source_refs) {
    for (const field of [
      "source_id", "repository", "revision", "path", "locator", "sha256",
      "access_date", "evidence_quality",
    ])
      assert(typeof source[field] === "string" && source[field] !== "",
        `${rule.id} source ref omits ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${rule.id} source digest drift`);
    if (source.evidence_quality === "ProjectPolicy")
      assert(source.note?.length > 0 && source.replacement_condition?.length > 0,
        `${rule.id} policy is not replaceable`);
  }
  const face = faceById.get(rule.dice_face_id);
  assert(face && face.source_id === rule.source_id,
    `${rule.id} face reference drift`);
  assert(face.mechanical_tag_codes.includes("SpecialType"),
    `${rule.id} face lacks exact Knowledge tag`);
  assert(rule.summary_en === face.summary_en
    && rule.summary_zh_cn === face.summary_zh_cn
    && rule.effect_text_hash === face.effect_text_hash
    && JSON.stringify(rule.parameters) === JSON.stringify(face.parameters),
  `${rule.id} face semantics drift`);
  assert(rule.parameters.every((parameter) => decimalPattern.test(parameter)),
    `${rule.id} parameter drift`);
  assert(operations.has(rule.operation)
    && triggerBoundaries.has(rule.trigger_boundary)
    && targetScopes.has(rule.target_scope)
    && selectionModes.has(rule.selection_mode)
    && knowledgeAccess.has(rule.knowledge_access),
  `${rule.id} typed Knowledge semantic drift`);
  assert(rule.target_policy.policy_id === "knowledge-target-selection-v1"
    && rule.target_policy.evidence_quality === "ProjectPolicy"
    && rule.target_policy.empty_candidate_behavior === "NoEffect"
    && rule.target_policy.replacement_condition.length > 0,
  `${rule.id} target policy drift`);
  assert(rule.simultaneous_resolution_policy.policy_id
    === "knowledge-simultaneous-resolution-v1"
    && rule.simultaneous_resolution_policy.evidence_quality === "ProjectPolicy"
    && rule.simultaneous_resolution_policy.tiers.length === 6
    && rule.simultaneous_resolution_policy.replacement_condition.length > 0,
  `${rule.id} resolution policy drift`);
  for (const diceId of [
    rule.custom_dice_interactions.countdown_dice_id,
    rule.custom_dice_interactions.collapse_prevention_dice_id,
    rule.custom_dice_interactions.collapse_reward_dice_id,
  ])
    assert(diceById.has(diceId), `${rule.id} custom-dice interaction drift`);
  assert(rule.custom_dice_interactions.evidence_quality === "ExactStructured",
    `${rule.id} interaction evidence drift`);
  operationIds.set(
    rule.operation,
    [...(operationIds.get(rule.operation) ?? []), rule.source_id],
  );
}

assert(JSON.stringify(operationIds.get("ProtectCollapsingDomainsWithKnowledge"))
  === JSON.stringify(["2031", "2034"]),
"about-to-collapse Knowledge closure drift");
assert(JSON.stringify(operationIds.get("OverrideMovementToKnowledgeDomain"))
  === JSON.stringify(["2047"]), "Knowledge movement override drift");
assert(JSON.stringify(operationIds.get("RemoveKnowledgeAndRewardPerRemoval"))
  === JSON.stringify(["2077"]), "Knowledge consumption closure drift");
assert(JSON.stringify(operationIds.get("TransformToBlankAndPreserveKnowledge"))
  === JSON.stringify(["2079"]), "Knowledge preservation closure drift");
assert(rules.filter(({ knowledge_access: access }) => access === "Apply").length
  === 15, "Knowledge apply-rule count drift");
assert(rules.filter(({ knowledge_access: access }) => access === "Query").length
  === 5, "Knowledge query-rule count drift");
assert(rules.filter(({ trigger_boundary: boundary }) =>
  boundary === "AfterMovement"
  || boundary === "AfterMovementBeforeCollapse"
  || boundary === "OnEnterDuringMovement").length === 5,
"Knowledge movement-boundary count drift");

const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const actual = rules.map(({ source_id: sourceId }) => sourceId).sort();
const required = manifest.categories.knowledge_bindings.records
  .map(({ id }) => id).sort();
assert(JSON.stringify(actual) === JSON.stringify(required),
  "Knowledge manifest exact-once drift");
assert(manifest.categories.knowledge_bindings.records.every(({ binding }) =>
  binding === "SpecialType"), "Knowledge manifest binding drift");

console.log(
  "Gold and Gears Knowledge verified (22 exact face bindings; typed apply, " +
  "query, remove and preserve operations; replaceable target/order policies).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
