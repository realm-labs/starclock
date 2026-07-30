#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-loadouts.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const decisions = json(
  "content-reference/unknowable-domain-v1/decision-components.json",
);
const choices = json(
  "content-reference/unknowable-domain-v1/component-choice-programs.json",
);
const layouts = json(
  "content-reference/unknowable-domain-v1/slot-layouts.json",
);
const loadouts = json(
  "content-reference/unknowable-domain-v1/loadouts.json",
);
const transitions = json(
  "content-reference/unknowable-domain-v1/loadout-transition-rules.json",
);
assert(decisions.length === 25, "Decision Component denominator drift");
assert(choices.length === 25, "Decision choice-program denominator drift");
assert(layouts.length === 3, "slot-layout denominator drift");
assert(loadouts.length === 72, "level-loadout denominator drift");
assert(transitions.length === 3, "loadout transition denominator drift");
const allRows = [
  ...decisions,
  ...choices,
  ...layouts,
  ...loadouts,
  ...transitions,
];
assert(unique(allRows.map(({ id }) => id)), "duplicate loadout stable ID");

for (const row of allRows) {
  assert(row.schema_revision === "starclock.unknowable-domain-row.v1"
    && row.ownership === "UnknowableDomain"
    && row.coverage_state === "DataReady",
  `${row.id} envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field] !== "",
      `${row.id} lacks ${field}`);
  assert(row.source_refs.length >= 1
    && row.source_refs.every((source) =>
      source.game_version === "4.4"
        && /^[0-9a-f]{64}$/u.test(source.sha256)),
  `${row.id} provenance drift`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
}

const componentLevels = json(
  "content-reference/unknowable-domain-v1/component-levels.json",
);
const componentLevelIds = new Set(componentLevels.map(({ id }) => id));
const componentIds = new Set(
  json("content-reference/unknowable-domain-v1/components.json")
    .filter(({ kind }) => kind === "Component")
    .map(({ id }) => id),
);
const choiceIds = new Set(choices.map(({ id }) => id));
assert(decisions.every(({ component_id: id, eligibility, scope, repetition,
  choice_program_ids: programIds, effect_program_id: effectId }) =>
  componentIds.has(id)
    && eligibility === "MagicUnitCategoryUltra"
    && scope === "Unspecified"
    && repetition === "Unspecified"
    && programIds.length === 1
    && programIds.every((programId) => choiceIds.has(programId))
    && componentLevelIds.has(effectId)),
"Decision Component binding drift");
const decisionComponentIds = new Set(decisions.map(({ component_id: id }) => id));
assert(decisionComponentIds.size === 25, "Decision Component exact-once drift");
for (const choice of choices) {
  assert(choice.candidate_set.length === 25
    && unique(choice.candidate_set)
    && choice.candidate_set.every((id) => decisionComponentIds.has(id))
    && choice.candidate_set_basis === "MagicUnitCategoryUltra",
  `${choice.id} candidate boundary drift`);
  assert(choice.offer_reachability === "Unspecified"
    && choice.ordering === "Unspecified"
    && choice.repetition === "Unspecified"
    && choice.fallback === "Unspecified",
  `${choice.id} overclaims choice semantics`);
  assert(choice.outcomes.length === 1
    && componentLevelIds.has(choice.outcomes[0].effect_program_id),
  `${choice.id} outcome does not resolve`);
}

const layoutShapes = layouts.map(({ active_count: active,
  attach_count: attach, passive_count: passive }) =>
`${active}:${attach}:${passive}`).sort();
assert(layoutShapes.join(",") === "1:1:2,1:2:2,1:2:3",
  "slot-layout shape drift");
const layoutIds = new Set(layouts.map(({ id }) => id));
const scepterLevels = json(
  "content-reference/unknowable-domain-v1/scepter-levels.json",
);
const scepterLevelIds = new Set(scepterLevels.map(({ id }) => id));
assert(loadouts.every(({ scepter_level_id: levelId, slot_layout_id: layoutId,
  locked_component_ids: locked, locked_slot_resolution: lockedResolution,
  authored_occupancy: occupancy }) =>
  scepterLevelIds.has(levelId)
    && layoutIds.has(layoutId)
    && locked.length === 1
    && locked.every((id) => componentLevelIds.has(id))
    && lockedResolution === "Unspecified"
    && occupancy.length === 0),
"loadout source binding drift");
assert(loadouts.filter(({ slot_ids: slots }) => slots.length === 4).length === 24
  && loadouts.filter(({ slot_ids: slots }) => slots.length === 5).length === 24
  && loadouts.filter(({ slot_ids: slots }) => slots.length === 6).length === 24,
"loadout slot-count progression drift");
for (const loadout of loadouts) {
  assert(unique(loadout.slot_ids)
    && loadout.slots.length === loadout.slot_ids.length
    && loadout.slots.every(({ id, slot_type: type, occupancy }) =>
      loadout.slot_ids.includes(id)
        && ["Active", "Attach", "Passive"].includes(type)
        && occupancy === "Unspecified"),
  `${loadout.id} slot expansion drift`);
}

assert(exactOnce(transitions.map(({ operation }) => operation),
  ["Insert", "Remove", "Replace"]),
"loadout transition operation drift");
for (const rule of transitions) {
  assert(rule.evidence_quality === "ProjectPolicy"
    && rule.policy_id === "loadout-transition-policy-v1"
    && rule.rejected_mutation === "PreserveAuthoritativeState"
    && rule.no_legal_candidate === "ReturnNoLegalCandidateWithoutMutation"
    && rule.source_refs.every(({ evidence_quality: quality, note,
      replacement_condition: replacement }) =>
      quality === "ProjectPolicy" && note?.length > 0 && replacement?.length > 0),
  `${rule.id} is not an explicit replaceable policy`);
}

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
assert(exactOnce(
  decisions.map(({ source_id: id }) => id),
  manifest.categories.decision_components.records.map(({ id }) => id),
), "Decision Component manifest closure drift");
assert(exactOnce(
  layouts.map(({ source_id: id }) => id),
  manifest.categories.slot_layouts.records.map(({ id }) => id),
), "slot-layout manifest closure drift");

console.log(
  "Unknowable Domain loadouts verified (25 Decision Components and " +
  "choice boundaries; 3 layouts; 72 exact level loadouts; 3 replaceable " +
  "transition policies; hidden offer/locked-slot semantics remain fail-closed).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function exactOnce(left, right) {
  const ordered = (values) => [...values].sort();
  return JSON.stringify(ordered(left)) === JSON.stringify(ordered(right));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
