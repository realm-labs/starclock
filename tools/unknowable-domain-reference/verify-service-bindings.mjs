#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-service-bindings.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const currencies = json(
  "content-reference/unknowable-domain-v1/currencies.json",
);
const adventures = json(
  "content-reference/unknowable-domain-v1/adventure-outcomes.json",
);
const npcs = json(
  "content-reference/unknowable-domain-v1/mode-service-npcs.json",
);
const rules = json(
  "content-reference/unknowable-domain-v1/service-rules.json",
);
assert(currencies.length === 1, "currency denominator drift");
assert(adventures.length === 9, "Adventure denominator drift");
assert(npcs.length === 5, "mode service NPC denominator drift");
assert(rules.length === 40, "service rule denominator drift");
for (const [kind, rows] of [
  ["UnknowableCurrency", currencies],
  ["UnknowableAdventureOutcome", adventures],
  ["ModeServiceNpc", npcs],
  ["UnknowableServiceRule", rules],
]) {
  assert(unique(rows.map(({ id }) => id)), `${kind} duplicate stable ID`);
  assert(rows.every((row) =>
    row.kind === kind
      && row.schema_revision === "starclock.unknowable-domain-row.v1"
      && row.coverage_state === "DataReady"
      && row.evidence_quality === "ExactStructured"
      && row.name_en
      && row.name_zh_cn
      && row.summary_en
      && row.summary_zh_cn
      && row.source_refs.length >= 1
      && row.source_refs.every((source) =>
        source.revision ===
          "fd978d6ef09f941fba644c731ab54abd6f7c3568"
          && source.game_version === "4.4"
          && source.mechanism_quality === "DirectStructured"
          && /^[0-9a-f]{64}$/u.test(source.sha256)),
  ), `${kind} envelope/provenance drift`);
}

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
assert(exactOnce(
  adventures.map(({ source_id: id }) => id.replace("adventure-outcome:", "")),
  manifest.categories.adventure_outcomes.records.map(({ id }) => id),
), "Adventure manifest closure drift");
assert(exactOnce(
  npcs.map(({ source_id: id }) => id.replace("mode-service-npc:", "")),
  manifest.categories.mode_service_npcs.records.map(({ id }) => id),
), "mode service NPC manifest closure drift");
const expectedServiceSources = [
  ...manifest.categories.workbenches.records.map(({ id }) => `workbench:${id}`),
  ...manifest.categories.workbench_functions.records
    .map(({ id }) => `workbench-function:${id}`),
  ...manifest.categories.gamble_groups.records
    .map(({ id }) => `gamble-group:${id}`),
  ...manifest.categories.gamble_units.records
    .map(({ id }) => `gamble-unit:${id}`),
  ...manifest.categories.adventure_outcomes.records
    .map(({ id }) => `adventure-outcome:${id}`),
  ...manifest.categories.mode_service_npcs.records
    .map(({ id }) => `mode-service-npc:${id}`),
];
assert(exactOnce(
  rules.map(({ source_id: id }) => id),
  expectedServiceSources,
), "40-parent service rule closure drift");

const currency = currencies[0];
assert(currency.id === "unknowable-domain.currency.cosmic-fragments"
  && currency.initial_amount === "Unspecified"
  && currency.cap === "Unspecified"
  && currency.carry_policy === "Unspecified"
  && currency.consumer_service_ids.length === 3
  && currency.runtime_lowered === false,
"Cosmic Fragment currency boundary drift");
assert(exactOnce(
  currency.consumer_service_ids,
  [
    "unknowable-domain.workbench-function.6",
    "unknowable-domain.workbench-function.7",
    "unknowable-domain.workbench-function.10",
  ],
), "currency consumer binding drift");

const adventureKinds = countBy(adventures, ({ adventure_type: type }) => type);
assert(JSON.stringify(adventureKinds) === JSON.stringify({
  RogueCaptureMonster: 3,
  RogueDestroyProp: 2,
  RogueEscapeLaser: 1,
  RogueTurntable: 1,
  RogueWolfGun: 2,
}), "Adventure type split drift");
assert(adventures.every((row) =>
  row.tier === "Unspecified"
    && row.offered_result === "Unspecified"
    && row.eligibility === "Unspecified"
    && row.runtime_lowered === false
    && row.param_group_id),
"Adventure unpublished result fields were claimed");
assert(adventures.reduce((sum, row) =>
  sum + row.parameter_records.length, 0) === 9,
"Adventure parameter-record denominator drift");
assert(adventures.every((row) =>
  row.parameter_records.every(({ values }) => numericLeavesAreStrings(values))),
"Adventure parameter decimals are not canonical strings");

const optionNodes = npcs.reduce((sum, row) => sum + row.eligibility.length, 0);
const options = npcs.flatMap(({ service_options: values }) => values);
assert(optionNodes === 6 && options.length === 7,
  "mode service option graph denominator drift");
assert(countBy(npcs, ({ dialogue_type: type }) => type).Event === 3
  && countBy(npcs, ({ dialogue_type: type }) => type).Story === 2,
"mode service dialogue-type split drift");
assert(options.filter(({ random_resolution: resolution }) =>
  resolution === "Unspecified").length === 2,
"mode service random-resolution denominator drift");
assert(options.filter(({ targets }) => targets.includes("Component")).length === 3
  && options.filter(({ targets }) => targets.includes("Scepter")).length === 1
  && options.filter(({ targets }) => targets.includes("Curio")).length === 1,
"mode service target split drift");
assert(options.every((option) =>
  option.service_id
    && option.operations.length >= 1
    && /^[0-9a-f]{64}$/u.test(option.choice_label_sha256_en)
    && /^[0-9a-f]{64}$/u.test(option.choice_label_sha256_zh_cn)
    && /^[0-9a-f]{64}$/u.test(option.result_sha256_en)
    && /^[0-9a-f]{64}$/u.test(option.result_sha256_zh_cn)),
"mode service option evidence drift");
assert(npcs.every((row) =>
  row.graph_path.startsWith(
    "Config/Level/Rogue/RogueNPC/RogueNPC_260/")
    && row.price_resolution === "Unspecified"
    && row.runtime_lowered === false
    && exactOnce(
      row.service_ids,
      row.service_options.map(({ service_id: id }) => id),
    )),
"mode service NPC graph/binding drift");

const kindCounts = countBy(rules, ({ service_kind: kind }) => kind);
assert(JSON.stringify(kindCounts) === JSON.stringify({
  AdventureOutcome: 9,
  GambleGroup: 10,
  GambleUnit: 7,
  ModeServiceNpc: 5,
  Workbench: 4,
  WorkbenchFunction: 5,
}), "service rule kind split drift");
assert(rules.every(({ runtime_lowered: lowered }) => lowered === false),
  "service rule was runtime-lowered");
assert(rules.filter(({ service_kind: kind }) => kind === "GambleGroup")
  .every(({ price, outcome }) =>
    price === "Unspecified"
      && outcome.resolution === "Unspecified"
      && outcome.unit_binding_resolution === "Unspecified"),
"unpublished gamble group binding was claimed");

const boundary = fs.readFileSync(path.join(
  root,
  "evidence/unknowable-domain-reference-v1/service-adventure-boundary.md",
), "utf8");
for (const phrase of [
  "40 source parents",
  "Cosmic Fragments",
  "nine Adventure rows",
  "five residual mode NPC",
  "seven mechanical service/entry options",
  "`Unspecified`",
  "no service, gamble or Adventure rule is",
])
  assert(boundary.includes(phrase), `service boundary omits ${phrase}`);

console.log(
  "Unknowable Domain service/Adventure bindings verified (1 currency; " +
  "40 source-parent rules; 9 Adventure rows/9 parameter records; 5 NPC " +
  "graphs/6 nodes/7 options; prices, rewards and gamble edges fail closed).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function countBy(rows, key) {
  return Object.fromEntries([...Map.groupBy(rows, key).entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([value, entries]) => [value, entries.length]));
}
function numericLeavesAreStrings(value) {
  if (Array.isArray(value)) return value.every(numericLeavesAreStrings);
  if (value && typeof value === "object")
    return Object.values(value).every(numericLeavesAreStrings);
  return typeof value !== "number";
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
