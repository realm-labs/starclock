#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-curios.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const curios = json("content-reference/unknowable-domain-v1/curios.json");
const states = json("content-reference/unknowable-domain-v1/curio-states.json");
const groups = json("content-reference/unknowable-domain-v1/curio-groups.json");
const rules = json("content-reference/unknowable-domain-v1/curio-rules.json");
assert(curios.length === 60, "Curio identity denominator drift");
assert(states.length === 81, "Curio state denominator drift");
assert(groups.length === 47, "Curio group denominator drift");
assert(rules.length === 128, "Curio rule denominator drift");
for (const [kind, rows] of [
  ["UnknowableCurio", curios],
  ["UnknowableCurioState", states],
  ["UnknowableCurioGroup", groups],
  ["UnknowableCurioRule", rules],
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
  curios.map(({ source_id: id }) => id.replace("curio:", "")),
  manifest.categories.curios.records.map(({ id }) => id),
), "Curio manifest closure drift");
assert(exactOnce(
  states.map(({ source_id: id }) => id.replace("curio-state:", "")),
  manifest.categories.curio_states.records.map(({ id }) => id),
), "Curio state manifest closure drift");
assert(exactOnce(
  groups.map(({ source_id: id }) => id.replace("curio-group:", "")),
  manifest.categories.curio_groups.records.map(({ id }) => id),
), "Curio group manifest closure drift");

const poolMembers = json(
  "content-reference/unknowable-domain-v1/pool-membership.json",
).filter(({ member_kind: kind }) => kind === "Curio");
assert(exactOnce(
  curios.map(({ source_id: id }) => id.replace("curio:", "")),
  poolMembers.map(({ source_id: id }) => id.replace("curio:", "")),
), "type-260 Curio membership drift");
const stateById = new Map(states.map((row) => [row.id, row]));
const groupById = new Map(groups.map((row) => [row.id, row]));
const stateReferences = curios.flatMap(({ state_ids: ids }) => ids);
assert(stateReferences.length === 81 && unique(stateReferences),
  "Curio-to-state exact-once binding drift");
assert(stateReferences.every((id) => stateById.has(id)),
  "Curio references an unknown mode copy");
for (const curio of curios) {
  assert(curio.ownership === "Shared"
    && curio.reachability_proof === "ExplicitModeType260"
    && curio.account_reward_excluded === true
    && curio.pool_ids.includes("unknowable-domain.pool.curios.type-260"),
  `${curio.id} reachability/exclusion drift`);
  assert(curio.pool_ids.filter((id) => id.includes("curio-group."))
    .every((id) => groupById.has(id)),
  `${curio.id} references an unknown group`);
}
const multiplicities = new Map(curios.map((curio) =>
  [curio.handbook_id, curio.state_ids.length]));
assert(multiplicities.get("320") === 8
  && multiplicities.get("108") === 4
  && multiplicities.get("315") === 12
  && [...multiplicities.values()].filter((count) => count === 1).length === 57,
"mode-copy multiplicity drift");

const edges = groups.flatMap(({ weighted_members: members }) => members);
assert(edges.length === 691, "Curio group edge denominator drift");
assert(new Set(edges.map(({ weight }) => weight)).size === 3
  && exactOnce([...new Set(edges.map(({ weight }) => weight))], ["1", "2", "4"]),
"Curio group weight values drift");
assert(edges.every(({ state_id: id }) => stateById.has(id)),
  "Curio group contains unknown mode copy");
assert(exactOnce(
  [...new Set(edges.map(({ state_id: id }) => id))],
  states.map(({ id }) => id),
), "Curio group union does not cover every copy");
assert(groups.every(({ eligibility, ordering }) =>
  eligibility === "Unspecified" && ordering === "Unspecified"),
"unpublished group selection semantics were claimed");
for (const state of states) {
  assert(state.curio_id && state.source_state_id
    && Array.isArray(state.effect_program.parameter_values)
    && state.effect_program.runtime_lowered === false
    && /^[0-9a-f]{64}$/u.test(state.effect_program.description_sha256_en)
    && /^[0-9a-f]{64}$/u.test(state.effect_program.description_sha256_zh_cn)
    && state.pool_ids.every((id) => groupById.has(id)),
  `${state.id} effect/pool binding drift`);
}

const stateRules = rules.filter(({ curio_state_id: id }) => id);
const groupRules = rules.filter(({ curio_group_id: id }) => id);
assert(stateRules.length === 81 && groupRules.length === 47,
  "Curio rule kind split drift");
assert(exactOnce(
  stateRules.map(({ curio_state_id: id }) => id),
  states.map(({ id }) => id),
), "Curio state rule exact-once drift");
assert(exactOnce(
  groupRules.map(({ curio_group_id: id }) => id),
  groups.map(({ id }) => id),
), "Curio group rule exact-once drift");
assert(rules.every(({ runtime_lowered: lowered }) => lowered === false),
  "Curio reference rule was runtime-lowered");
assert(groupRules.every(({ eligibility, ordering, fallback }) =>
  eligibility === "Unspecified"
    && ordering === "Unspecified"
    && fallback === "Unspecified"),
"unpublished group consumer/order/fallback was claimed");

const expectedCharges = new Map(Object.entries({
  7101: "1", 7125: "4", 7212: "5", 7312: "4", 7315: "4",
  7316: "4", 7317: "4", 7318: "3", 7319: "4", 7320: "3",
  7321: "2", 7322: "3", 7323: "2", 7324: "3", 7325: "1",
  7326: "3", 7405: "9", 7406: "5", 7407: "2", 7504: "2",
}));
const finite = stateRules.filter(({ lifecycle }) =>
  lifecycle?.resolution === "ExactLocalized");
assert(finite.length === 20, "finite Curio lifecycle denominator drift");
for (const rule of finite) {
  const stateId = rule.curio_state_id.split(".").at(-1);
  assert(rule.lifecycle.charges === expectedCharges.get(stateId),
    `${rule.id} finite charge drift`);
}
const conditional = stateRules.filter(({ lifecycle }) =>
  lifecycle?.resolution === "ExactLocalizedConditional");
assert(conditional.length === 3
  && exactOnce(
    conditional.map(({ curio_state_id: id }) => id.split(".").at(-1)),
    ["7123", "7303", "7501"],
  ), "conditional Curio destruction drift");
const repairRules = stateRules.filter(({ repair }) =>
  repair?.resolution === "ExactLocalized");
const replacementRules = stateRules.filter(({ replacement }) =>
  replacement?.resolution === "ExactLocalized");
assert(repairRules.length === 1
  && repairRules[0].curio_state_id.endsWith(".7116")
  && repairRules[0].repair.selection_order === "Unspecified",
"Curio repair boundary drift");
assert(replacementRules.length === 1
  && replacementRules[0].curio_state_id.endsWith(".7110")
  && replacementRules[0].replacement.candidate_pool === "Unspecified",
"Curio replacement boundary drift");

const boundary = fs.readFileSync(path.join(
  root,
  "evidence/unknowable-domain-reference-v1/curio-boundary.md",
), "utf8");
for (const phrase of [
  "60 shared Curio",
  "81 `RogueMagicMiracle`",
  "691 exact weighted edges",
  "20 finite-use",
  "`Unspecified`",
  "not lowered into runtime handlers",
])
  assert(boundary.includes(phrase), `Curio boundary omits ${phrase}`);

console.log(
  "Unknowable Domain Curios verified (60 shared identities; 81 mode copies; " +
  "47 groups/691 weighted edges; 20 finite and 3 conditional destruction " +
  "rules; exact repair/replacement boundaries; hidden selection stays " +
  "Unspecified).",
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
