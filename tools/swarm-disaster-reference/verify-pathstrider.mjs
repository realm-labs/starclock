#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-pathstrider.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const read = (name) => JSON.parse(fs.readFileSync(path.join(
  root,
  "content-reference/swarm-disaster-v1",
  name,
), "utf8"));
const manifest = JSON.parse(fs.readFileSync(path.join(
  root,
  "content-manifests/swarm-disaster-v1/content-manifest.json",
), "utf8"));
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function unique(values) {
  return values.length === new Set(values).size;
}
function exactOnce(rows, category, prefix) {
  const expected = new Set(manifest.categories[category].records
    .map(({ id }) => `${prefix}${id}`));
  for (const row of rows)
    assert(expected.delete(row.id), `${row.id} manifest mismatch`);
  assert(expected.size === 0, `${category} exact-once mismatch`);
}
function histogram(rows, field) {
  return Object.fromEntries([...rows.reduce((counts, row) => {
    counts.set(row[field], (counts.get(row[field]) ?? 0) + 1);
    return counts;
  }, new Map()).entries()].sort());
}

const objectives = read("pathstrider-objectives.json");
const conditions = read("pathstrider-finish-conditions.json");
const unlocks = read("pathstrider-unlocks.json");
const chapters = read("mechanical-chapter-locators.json");
assert(objectives.length === 31, "Pathstrider objective count drift");
assert(conditions.length === 102, "finish-condition count drift");
assert(unlocks.length === 110, "unlock count drift");
assert(chapters.length === 13, "mechanical chapter count drift");
for (const rows of [objectives, conditions, unlocks, chapters])
  assert(unique(rows.map(({ id }) => id)), "duplicate Pathstrider ID");

exactOnce(
  objectives,
  "pathstrider_cabinets",
  "swarm-disaster.pathstrider-objective.",
);
exactOnce(
  conditions,
  "pathstrider_finish_conditions",
  "swarm-disaster.pathstrider-finish-condition.",
);
exactOnce(
  unlocks,
  "pathstrider_unlocks",
  "swarm-disaster.pathstrider-unlock.",
);
exactOnce(
  chapters,
  "mechanical_chapter_locators",
  "swarm-disaster.mechanical-chapter.",
);

const cabinetIds = new Set(read("pathstrider-cabinets.json")
  .map(({ id }) => id));
const conditionIds = new Set(conditions.map(({ id }) => id));
const unlockIds = new Set(unlocks.map(({ id }) => id));
const dimensionIds = new Set(read("communing-dimensions.json")
  .map(({ id }) => id));
for (const objective of objectives)
  assert(cabinetIds.has(objective.cabinet_id)
    && objective.finish_condition_id.startsWith(
      "swarm-disaster.external-quest-condition.",
    )
    && objective.progress_policy.source === "ExternalQuestCompletion"
    && objective.progress_policy.description_parameters.length >= 1
    && objective.unlock_ids.every((id) => cabinetIds.has(id)),
  `${objective.id} objective binding drift`);
for (const condition of conditions)
  assert(["GreaterEqual", "ListContain", "NoPara"].includes(
    condition.comparison,
  )
    && condition.target_progress.length > 0
    && condition.unlock_ids.every((id) => unlockIds.has(id))
    && condition.enabled_for_swarm_compilation
      === (condition.mode_hint === "SwarmDisaster"),
  `${condition.id} condition semantics drift`);
for (const unlock of unlocks)
  assert(conditionIds.has(unlock.finish_condition_id)
    && unlock.evaluation_boundary === "AfterAcceptedActivityOperation"
    && unlock.unlock_consequence.enabled_for_swarm_compilation
      === (unlock.mode_hint === "SwarmDisaster")
    && !(unlock.mode_hint === "GoldAndGears"
      && unlock.unlock_consequence.enabled_for_swarm_compilation),
  `${unlock.id} unlock binding drift`);
for (const chapter of chapters)
  assert((chapter.dimension_id === ""
    || dimensionIds.has(chapter.dimension_id))
    && chapter.mechanical_unlock.bonus_payload === "",
  `${chapter.id} chapter locator drift`);

assert(JSON.stringify(histogram(conditions, "mode_hint"))
  === JSON.stringify({
    GoldAndGears: 45,
    SwarmDisaster: 15,
    UnresolvedSharedDlc: 42,
  }), "finish-condition mode-hint boundary drift");
assert(JSON.stringify(histogram(unlocks, "mode_hint"))
  === JSON.stringify({
    GoldAndGears: 51,
    SwarmDisaster: 15,
    UnresolvedSharedDlc: 44,
  }), "unlock mode-hint boundary drift");
assert(chapters.filter(({ mechanical_unlock: unlock }) =>
  unlock.bonus_declared).length === 3,
"chapter bonus-locator count drift");
assert(chapters.filter(({ dimension_id: id }) => id === "").length === 1,
"chapter without dimension threshold count drift");

console.log(
  "Swarm Disaster Pathstrider verification passed: 31 external quest " +
  "objectives, 102 exact finish conditions, 110 fail-closed unlocks and " +
  "13 mechanical chapter locators.",
);
