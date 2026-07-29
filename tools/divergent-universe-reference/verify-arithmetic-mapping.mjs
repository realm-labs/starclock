#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(
  valueAfter("--source-cache")
    ?? path.join(root, ".cache/content-reference/turnbasedgamedata"),
);
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/import-arithmetic-mapping.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const eligibility = json(path.join(
  outputRoot,
  "arithmetic-mapping-eligibility.json",
));
const builds = json(path.join(outputRoot, "arithmetic-mapping-builds.json"));
const rules = json(path.join(outputRoot, "arithmetic-mapping-rules.json"));
assert(eligibility.length === 84, "eligibility denominator drift");
assert(builds.length === 95, "build/role union denominator drift");
assert(rules.length === 7, "Arithmetic Mapping lifecycle rule drift");
assert(unique(eligibility.map(({ id }) => id))
  && unique(builds.map(({ id }) => id))
  && unique(rules.map(({ id }) => id)),
"Arithmetic Mapping IDs are not unique");

const manifest = json(path.join(
  root,
  "content-manifests/divergent-universe-v1/content-manifest.json",
));
const expectedSources = new Map();
for (const categoryId of [
  "arithmetic_mapping_avatars",
  "arithmetic_mapping_build_refs",
  "arithmetic_mapping_roles",
])
  for (const record of manifest.categories[categoryId].records)
    expectedSources.set(record.source, {
      digest: record.evidence_sha256,
      categoryId,
    });
const actualSources = new Map();
for (const row of [...eligibility, ...builds])
  for (const ref of row.source_refs)
    if (expectedSources.has(`${ref.path}#${ref.locator}`))
      actualSources.set(`${ref.path}#${ref.locator}`, ref.sha256);
assert(actualSources.size === 258,
  "Arithmetic Mapping manifest obligations are not all accounted");
for (const [locator, expected] of expectedSources)
  assert(actualSources.get(locator) === expected.digest,
    `${expected.categoryId}/${locator} source receipt drift`);

const sourceEligibility = sourceRows("ExcelOutput/RogueTournBuildRefAvatar.json");
assert(setEqual(
  new Set(eligibility.map(({ avatar_id: id }) => id)),
  new Set(sourceEligibility.map(({ AvatarID }) => String(AvatarID))),
), "eligibility stable-ID closure drift");
const roleRows = sourceRows("ExcelOutput/RogueTournRole.json");
assert(setEqual(
  new Set(builds.map(({ avatar_id: id }) => id)),
  new Set(roleRows.map(({ AvatarID }) => String(AvatarID))),
), "role/build stable-ID closure drift");
assert(builds.filter(({ special_avatar_id: id }) => id).length === 79,
  "SpecialAvatar mapping denominator drift");
assert(builds.every(({ role_buff_id: id }) => id),
  "a role/build row lacks its exact role buff");
assert(builds.every((row) =>
  row.role_buff_binding_key
    && row.role_buff_modifier_name
    && row.role_buff_parameters.length >= 4
    && row.role_buff_parameters.length <= 7),
"role buff binding/parameter closure drift");
assert(JSON.stringify(Object.fromEntries(
  [...Map.groupBy(builds, (row) => row.role_buff_parameters.length)]
    .map(([length, rows]) => [length, rows.length]),
)) === JSON.stringify({ 4: 41, 5: 34, 6: 18, 7: 2 }),
"role buff parameter arity distribution drift");

const unresolvedIdentity = builds.filter((row) =>
  row.public_identity_resolution === "MissingReleasedAvatarConfig");
assert(unresolvedIdentity.map(({ avatar_id: id }) => id).sort(compare)
  .join(",") === "1014,1015,1508,1509",
"unreleased/unresolved AvatarConfig boundary drift");
assert(unresolvedIdentity.every((row) =>
  row.coverage_state === "Cataloged"
    && row.tags.includes("unresolved-public-identity")),
"unresolved public identities were promoted");
assert(builds.filter((row) => row.coverage_state === "DataReady").length === 91,
  "resolved mapping build DataReady count drift");

const ruleById = new Map(rules.map((row) => [row.id, row]));
for (const id of [
  "scope",
  "character-level",
  "traces",
  "relics",
  "light-cone",
  "refresh",
  "teardown",
])
  assert(ruleById.has(`divergent-universe.mapping-rule.${id}`),
    `missing mapping rule ${id}`);
assert(ruleById.get("divergent-universe.mapping-rule.character-level")
  .stronger_build_rule === "PreserveWhenConditionIsFalse"
  && ruleById.get("divergent-universe.mapping-rule.traces")
    .stronger_build_rule === "PreserveWhenConditionIsFalse"
  && ruleById.get("divergent-universe.mapping-rule.relics")
    .stronger_build_rule === "PreserveWhenConditionIsFalse",
"already-sufficient build preservation drift");
assert(ruleById.get("divergent-universe.mapping-rule.light-cone")
  .condition === "Unspecified",
"unpublished Light Cone condition was invented");
assert(ruleById.get("divergent-universe.mapping-rule.teardown")
  .account_mutation === false
  && rules.every(({ runtime_lowered: lowered }) => lowered === false),
"teardown/account/runtime boundary drift");

for (const row of [...eligibility, ...builds, ...rules])
  assert(validEnvelope(row), `${row.id} has invalid envelope`);
const digest = crypto.createHash("sha256")
  .update(fs.readFileSync(path.join(
    outputRoot,
    "arithmetic-mapping-eligibility.json",
  )))
  .update(fs.readFileSync(path.join(
    outputRoot,
    "arithmetic-mapping-builds.json",
  )))
  .update(fs.readFileSync(path.join(
    outputRoot,
    "arithmetic-mapping-rules.json",
  )))
  .digest("hex");
console.log(
  `Divergent Universe Arithmetic Mapping verified ` +
  `(${eligibility.length + builds.length + rules.length} rows; ` +
  `258 manifest receipts; digest ${digest}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function sourceRows(relative) {
  return json(path.join(sourceRoot, relative));
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function validEnvelope(row) {
  return row.schema_revision === "starclock.divergent-universe-row.v1"
    && row.name_en
    && row.name_zh_cn
    && row.summary_en
    && row.summary_zh_cn
    && ["DivergentUniverse", "Shared"].includes(row.ownership)
    && ["Cataloged", "Researched", "DataReady", "Blocked"].includes(
      row.coverage_state,
    )
    && row.source_refs.length > 0
    && row.source_refs.every((ref) => /^[0-9a-f]{64}$/u.test(ref.sha256));
}

function unique(values) {
  return new Set(values).size === values.length;
}

function setEqual(left, right) {
  return left.size === right.size && [...left].every((value) => right.has(value));
}

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
