#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const decimalPattern =
  /^(0|-?(?:[1-9][0-9]*(?:\.[0-9]*[1-9])?|0\.[0-9]*[1-9]))$/u;

execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-neural-network.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const nodes = json("content-reference/gold-and-gears-v1/neural-network.json");
assert(Array.isArray(nodes) && nodes.length === 40,
  "Neural Network node count drift");
assert(unique(nodes.map(({ id }) => id)),
  "Neural Network nodes contain duplicate IDs");
assert(nodes.every((node, index) => node.topological_index === index + 1),
  "Neural Network topological indexes drift");

const nodeById = new Map(nodes.map((node) => [node.id, node]));
let edgeCount = 0;
let totalCost = 0;
const operationCounts = new Map();
for (const node of nodes) {
  assert(node.schema_revision === "starclock.gold-and-gears-row.v1"
    && node.kind === "NeuralNetworkNode",
  `${node.id} row revision drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof node[field] === "string" && node[field].trim() !== "",
      `${node.id} has empty ${field}`);
  assert(node.ownership === "GoldAndGears"
    && node.coverage_state === "DataReady"
    && node.evidence_quality === "ExactStructured"
    && node.mechanism_quality === "ExactStructured"
    && node.disposition === "MechanicallyRelevant",
  `${node.id} common envelope drift`);
  assert(JSON.stringify(node.tags) === JSON.stringify([...node.tags].sort()),
    `${node.id} tags are not canonical`);
  assert(Array.isArray(node.source_refs) && node.source_refs.length >= 1,
    `${node.id} has no provenance`);
  for (const source of node.source_refs) {
    for (const field of [
      "source_id", "repository", "revision", "path", "locator", "sha256",
      "access_date", "evidence_quality",
    ])
      assert(typeof source[field] === "string" && source[field] !== "",
        `${node.id} source ref omits ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${node.id} source digest drift`);
    if (source.evidence_quality === "ProjectPolicy")
      assert(source.note?.length > 0 && source.replacement_condition?.length > 0,
        `${node.id} project policy is not replaceable`);
  }
  assert(node.source_refs[0].path === "ExcelOutput/RogueNousTalent.json",
    `${node.id} primary source drift`);
  assert(Array.isArray(node.costs) && node.costs.length === 1
    && node.costs[0].source_item_id === "281013"
    && decimalPattern.test(node.costs[0].amount),
  `${node.id} cost drift`);
  totalCost += Number(node.costs[0].amount);
  assert(node.external_unlock_ids.length === 0 && node.important === false,
    `${node.id} unlock metadata drift`);
  assert(node.source_parameters.every(({ index, value }, parameterIndex) =>
    index === parameterIndex + 1 && decimalPattern.test(value)),
  `${node.id} source parameter drift`);
  assert(/^[0-9a-f]{64}$/u.test(node.source_description_sha256_en)
    && /^[0-9a-f]{64}$/u.test(node.source_description_sha256_zh_cn),
  `${node.id} description digest drift`);
  for (const reference of [...node.prerequisite_ids, ...node.next_ids])
    assert(nodeById.has(reference), `${node.id} has unresolved graph reference`);
  for (const prerequisiteId of node.prerequisite_ids) {
    const prerequisite = nodeById.get(prerequisiteId);
    assert(prerequisite.topological_index < node.topological_index,
      `${node.id} is not topologically ordered`);
    assert(prerequisite.next_ids.includes(node.id),
      `${node.id} prerequisite reverse edge drift`);
  }
  for (const nextId of node.next_ids)
    assert(nodeById.get(nextId).prerequisite_ids.includes(node.id),
      `${node.id} next reverse edge drift`);
  edgeCount += node.next_ids.length;
  assert(Array.isArray(node.effect_contributions)
    && node.effect_contributions.length === 1,
  `${node.id} effect contribution drift`);
  const contribution = node.effect_contributions[0];
  assert(contribution.scope === node.effect_domain
    && contribution.mechanism_quality === "ExactStructured",
  `${node.id} effect domain drift`);
  operationCounts.set(
    contribution.operation,
    (operationCounts.get(contribution.operation) ?? 0) + 1,
  );
  assert(node.rule_contribution_id
    === `gold-gears.rule.neural-network.${node.source_id}`,
  `${node.id} rule contribution identity drift`);
}

assert(edgeCount === 53, "Neural Network prerequisite edge count drift");
assert(totalCost === 31250, "Neural Network aggregate cost drift");
assert(nodes.filter(({ prerequisite_ids: ids }) => ids.length === 0).length === 3,
  "Neural Network root count drift");
assert(nodes.filter(({ next_ids: ids }) => ids.length === 0).length === 1,
  "Neural Network terminal count drift");
assert(nodes.filter(({ effect_domain: domain }) => domain === "Battle").length
  === 30, "Neural Network battle classification drift");
assert(nodes.filter(({ effect_domain: domain }) => domain === "Activity").length
  === 9, "Neural Network activity classification drift");
assert(nodes.filter(({ effect_domain: domain }) =>
  domain === "ActivityAndBattle").length === 1,
"Neural Network cross-boundary classification drift");
assert(operationCounts.get("AddBattleStatRatio") === 30,
  "Neural Network battle-stat operation count drift");
for (const operation of [
  "ApplyFixedEntryDamage",
  "AddInitialCountdown",
  "AddBlessingStoreOfferCount",
  "AddRerollAttempts",
  "ExcludePreviousRerollResult",
])
  assert(operationCounts.get(operation) === 1,
    `${operation} operation count drift`);
assert(operationCounts.get("UpgradeDiceFaceSlot") === 3,
  "Neural Network slot-upgrade count drift");
assert(operationCounts.get("UnlockTrailblazeBonus") === 2,
  "Neural Network Trailblaze Bonus unlock count drift");

const statNodes = nodes.filter(({ effect_domain: domain }) => domain === "Battle");
assert(statNodes.every((node) =>
  node.effect_contributions[0].value === node.source_parameters[0]?.value
  && node.effect_contributions[0].unit === "Ratio"
  && node.effect_contributions[0].stacking === "AdditiveContribution"),
"Neural Network stat parameter transport drift");
assert(statNodes.filter((node) =>
  node.effect_contributions[0].target === "path-resonance.damage_ratio").length
  === 10, "Neural Network Path Resonance node count drift");

const reboot = sourceNode("201");
assert(reboot.effect_domain === "ActivityAndBattle"
  && reboot.effect_contributions[0].value === "0.99"
  && reboot.effect_contributions[0].eligible_battle_limit === "4"
  && JSON.stringify(reboot.source_parameters.map(({ value }) => value))
    === JSON.stringify(["4", "0.99"]),
"Neural Network Reboot Plane transport drift");
const countdown = sourceNode("801");
assert(countdown.effect_contributions[0].value === "1",
  "Neural Network countdown drift");
const store = sourceNode("1001");
assert(store.effect_contributions[0].value === "3",
  "Neural Network store count drift");
const rerolls = sourceNode("1201");
assert(rerolls.effect_contributions[0].value === "1",
  "Neural Network reroll gain drift");

const slots = new Map(
  json("content-reference/gold-and-gears-v1/dice-slots.json")
    .map((slot) => [slot.id, slot]),
);
for (const [sourceId, slotId] of [
  ["301", "gold-gears.dice-slot.5"],
  ["1401", "gold-gears.dice-slot.3"],
  ["2001", "gold-gears.dice-slot.6"],
]) {
  const node = sourceNode(sourceId);
  const contribution = node.effect_contributions[0];
  assert(slots.has(slotId) && contribution.target === slotId,
    `${node.id} slot target drift`);
  assert(contribution.target_policy.policy_id
    === "neural-network-slot-upgrade-target-v1"
    && contribution.target_policy.evidence_quality === "ProjectPolicy"
    && node.quality_overrides.length === 1
    && node.source_refs.length === 3,
  `${node.id} slot policy drift`);
}
const reroll = sourceNode("1701");
assert(reroll.effect_contributions[0].selection_policy.policy_id
  === "neural-network-reroll-empty-candidate-v1"
  && reroll.effect_contributions[0].selection_policy.empty_candidate_behavior
    === "KeepPreviousAndConsumeAttempt"
  && reroll.quality_overrides.length === 1
  && reroll.source_refs.length === 2,
"Neural Network reroll policy drift");

const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const actual = nodes.map(({ source_id: sourceId }) => sourceId).sort();
const required = manifest.categories.neural_network_nodes.records
  .map(({ id }) => id).sort();
assert(JSON.stringify(actual) === JSON.stringify(required),
  "Neural Network manifest exact-once drift");
for (const bonusId of ["204", "205"]) {
  assert(manifest.categories.trailblaze_bonuses.records
    .some(({ id }) => id === bonusId),
  `Neural Network Trailblaze Bonus ${bonusId} manifest reference drift`);
}

console.log(
  "Gold and Gears Neural Network verified (40 nodes; 53 edges; 31,250 " +
  "Neural Impulses; 30 battle, 9 activity and 1 cross-boundary contribution).",
);

function sourceNode(sourceId) {
  const node = nodes.find(({ source_id: id }) => id === sourceId);
  assert(node, `missing Neural Network source node ${sourceId}`);
  return node;
}
function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
