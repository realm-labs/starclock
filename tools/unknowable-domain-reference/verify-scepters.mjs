#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-scepters.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const files = new Map([
  ["scepters", json("content-reference/unknowable-domain-v1/scepters.json")],
  ["levels", json("content-reference/unknowable-domain-v1/scepter-levels.json")],
  ["activations", json(
    "content-reference/unknowable-domain-v1/scepter-activation-rules.json",
  )],
  ["states", json(
    "content-reference/unknowable-domain-v1/scepter-state-transitions.json",
  )],
]);
assert(files.get("scepters").length === 24, "Scepter denominator drift");
assert(files.get("levels").length === 72, "Scepter-level denominator drift");
assert(files.get("activations").length === 72,
  "Scepter activation denominator drift");
assert(files.get("states").length === 216,
  "Scepter lifecycle boundary drift");
const allRows = [...files.values()].flat();
assert(unique(allRows.map(({ id }) => id)), "duplicate Scepter stable ID");

for (const row of allRows) {
  assert(row.schema_revision === "starclock.unknowable-domain-row.v1"
    && row.ownership === "UnknowableDomain"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured",
  `${row.id} envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field] !== "",
      `${row.id} lacks ${field}`);
  assert(row.source_refs.length >= 2
    && row.source_refs.every((source) =>
      source.game_version === "4.4"
        && source.mechanism_quality === "DirectStructured"
        && /^[0-9a-f]{64}$/u.test(source.sha256)),
  `${row.id} provenance drift`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
}

const scepters = files.get("scepters");
const levels = files.get("levels");
const activations = files.get("activations");
const states = files.get("states");
const levelIds = new Set(levels.map(({ id }) => id));
const alignmentIds = new Set(
  json("content-reference/unknowable-domain-v1/alignments.json")
    .map(({ id }) => id),
);
assert(scepters.every(({ level_ids: ids }) =>
  ids.length === 3 && ids.every((id) => levelIds.has(id))),
"Scepter level references do not close");
assert(scepters.every(({ alignment_id: id }) => alignmentIds.has(id)),
  "Scepter Alignment reference does not resolve");
assert(new Set(scepters.map(({ style }) => style)).size === 4,
  "Scepter style boundary drift");
assert(scepters.filter(({ function: value }) => value === "Charge").length === 12
  && scepters.filter(({ function: value }) => value === "Speed").length === 12,
"Scepter function split drift");
assert(new Set(scepters.flatMap(({ slot_layout_ids: ids }) => ids)).size === 3,
  "Scepter slot-layout boundary drift");

assert(levels.every(({ level }) => ["1", "2", "3"].includes(level)),
  "Scepter level range drift");
assert(levels.every(({ locked_component_ids: ids }) => ids.length === 1),
  "Scepter locked-Component cardinality drift");
assert(unique(levels.flatMap(({ locked_component_ids: ids }) => ids)),
  "locked Component-level binding is not exact once");
assert(new Set(levels.map(({ power }) => power)).size === 3
  && ["150", "300", "600"].every((power) =>
    levels.some(({ power: value }) => value === power)),
"Scepter power progression drift");
assert(new Set(levels.flatMap(({ effect_ranges: ranges }) => ranges)).size === 4,
  "Scepter effect-range boundary drift");

const activationByLevel = new Map(
  activations.map((row) => [row.scepter_level_id, row]),
);
assert(activationByLevel.size === 72
  && [...levelIds].every((id) => activationByLevel.has(id)),
"Scepter activation exact-once drift");
assert(activations.filter(({ charge_or_speed: value }) =>
  value.kind === "Charge").length === 36
  && activations.filter(({ charge_or_speed: value }) =>
    value.kind === "Speed").length === 36,
"Scepter level function split drift");
for (const rule of activations) {
  assert(rule.ordered_operations.at(-1).includes("Attack"),
    `${rule.id} lacks attack dispatch`);
  assert(rule.target_selection_order === "Unspecified"
    && rule.simultaneous_trigger_order === "Unspecified",
  `${rule.id} overclaims hidden ordering`);
  if (rule.charge_or_speed.kind === "Charge")
    assert(rule.charge_or_speed.attack_threshold === "120"
      && rule.charge_or_speed.post_attack_reset === "Unspecified",
    `${rule.id} Charge boundary drift`);
  else
    assert(rule.charge_or_speed.speed === "100"
      && rule.charge_or_speed.post_attack_action_value === "Unspecified",
    `${rule.id} Speed boundary drift`);
}
const triggerKinds = new Set(activations.map(({ trigger }) => trigger));
assert(triggerKinds.size === 9 && triggerKinds.has("OwnAction"),
  "Scepter trigger taxonomy drift");

const stateGroups = Map.groupBy(states, ({ scepter_level_id: id }) => id);
assert(stateGroups.size === 72, "Scepter lifecycle grouping drift");
for (const [levelId, rows] of stateGroups) {
  assert(rows.length === 3, `${levelId} lifecycle boundary count drift`);
  assert(rows.every(({ ordinal }) =>
    Number.isSafeInteger(ordinal) && ordinal >= 0)
    && rows.map(({ ordinal }) => ordinal).sort().join(",") === "0,1,2",
    `${levelId} lifecycle order drift`);
  assert(rows.every(({ teardown }) => teardown === "Unspecified"),
    `${levelId} overclaims teardown`);
  assert(rows.some(({ from_state: from, to_state: to }) =>
    from === "Absent" && ["Charging", "TimelineWaiting"].includes(to)),
  `${levelId} initialization boundary drift`);
  assert(rows.some(({ to_state: to }) => to === "AttackDispatched")
    && rows.some(({ input }) => input === "DamagePerformFinishOnScepter"),
  `${levelId} activation/finish boundary drift`);
}

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
assert(exactOnce(
  scepters.map(({ source_id: id }) => id),
  manifest.categories.scepters.records.map(({ id }) => id),
), "Scepter manifest closure drift");
assert(exactOnce(
  levels.map(({ source_id: id }) => id),
  manifest.categories.scepter_levels.records.map(({ id }) => id),
), "Scepter-level manifest closure drift");
const lockedSourceIds = levels.map(({ source_id: id }) => `${id}:0`);
assert(exactOnce(
  lockedSourceIds,
  manifest.categories.scepter_locked_components.records.map(({ id }) => id),
), "locked Component manifest closure drift");

console.log(
  "Unknowable Domain Scepters verified (24 definitions; 72 levels and " +
  "locked Components; 72 activation rules; 216 lifecycle boundaries; " +
  "hidden reset/order/teardown remains fail-closed).",
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
