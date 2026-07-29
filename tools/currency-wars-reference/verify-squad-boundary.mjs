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
  "tools/currency-wars-reference/import-squad-boundary.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "squad-hp-rules.json": 1,
  "action-value-limits.json": 2,
  "battle-result-projections.json": 2,
  "run-failure-rules.json": 1,
};
const rowsByFile = Object.fromEntries(Object.entries(expected).map(
  ([file, count]) => {
    const rows = json(path.join(outputRoot, file));
    assert(rows.length === count, `${file} row count drift`);
    assert(unique(rows.map(({ id }) => id)), `${file} duplicate IDs`);
    assert(rows.every(validEnvelope), `${file} invalid envelope`);
    return [file, rows];
  },
));

const schema = json(path.join(
  root,
  "content-manifests/currency-wars-v1/normalized-schema.json",
));
for (const [file, rows] of Object.entries(rowsByFile)) {
  const contract = schema.files.find((entry) => entry.file === file);
  assert(contract, `${file} lacks normalized contract`);
  for (const row of rows)
    assert(contract.required_domain_fields.every((field) =>
      Object.hasOwn(row, field)),
    `${file}/${row.id} lacks required domain field`);
}

const squad = rowsByFile["squad-hp-rules.json"][0];
assert(squad.initial_hp === "100"
  && squad.minimum_hp === "0"
  && squad.maximum_hp.initial_value === "100"
  && squad.maximum_hp.resolution === "ProjectPolicy",
"Squad HP initialization boundary drift");
assert(squad.loss_rules.length === 1
  && squad.loss_rules[0].amount === "ConfiguredByNodeOrDifficulty"
  && squad.recovery_rules.some(({ trigger, amount }) =>
    trigger === "NodeVictory" && amount === "0"),
"Squad HP loss/recovery boundary drift");

const actionLimits = rowsByFile["action-value-limits.json"];
const finite = actionLimits.find(({ limit_kind: kind }) =>
  kind === "FiniteNodeConfigured");
const unlimited = actionLimits.find(({ limit_kind: kind }) =>
  kind === "Unlimited");
assert(finite?.initial_value === "ConfiguredByNodeOrDifficulty"
  && finite.timeout_boundary.battle_outcome === "NonVictory",
"finite action-value boundary drift");
assert(unlimited?.initial_value === "Infinite"
  && unlimited.decrement_rules.length === 0
  && unlimited.coverage_state === "DataReady",
"unlimited action-value exception drift");

const projections = new Map(
  rowsByFile["battle-result-projections.json"].map((row) =>
    [row.battle_outcome, row]),
);
assert(projections.get("Victory")?.squad_hp_projection
  === "PreserveBeforeContentContributions"
  && projections.get("NonVictory")?.run_disposition
    === "FailAtZeroOtherwiseContinue",
"battle-result projection drift");

const failure = rowsByFile["run-failure-rules.json"][0];
assert(failure.failure_condition === "ProjectedSquadHpEqualsZero"
  && failure.same_boundary_order.join(",")
    === [
      "DetermineVictoryBeforeTimeoutLoss",
      "ProjectVictoryOrConfiguredNonVictoryLoss",
      "ClampSquadHpToZero",
      "FailRunAtZeroOtherwiseContinue",
    ].join(",")
  && failure.source_refs.some(({ replacement_condition: condition }) =>
    condition?.includes("same-boundary precedence")),
"run-failure same-boundary policy drift");

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every(({ runtime_lowered: runtimeLowered }) =>
  runtimeLowered === false),
"reference rows must never claim runtime lowering");
assert(allRows.filter(({ coverage_state: state }) =>
  state === "DataReady").length === 1,
"only the exact unlimited-node exception may be DataReady in P1-B2");
assert(allRows.every((row) => row.source_refs.some(({ repository }) =>
  repository.includes("turnbasedgamedata"))),
"every row requires released source evidence");

for (const hash of [
  "1912261755023964838",
  "6971100623138337968",
  "7693488975416237801",
  "5626677263404827289",
  "4983101780975847570",
  "7940111314490605947",
])
  for (const locale of ["EN", "CHS"]) {
    const textMap = json(path.join(sourceRoot, `TextMap/TextMap${locale}.json`));
    assert(typeof textMap[hash] === "string" && textMap[hash].length > 0,
      `missing released TextMap${locale} row ${hash}`);
  }

const digest = crypto.createHash("sha256");
for (const file of Object.keys(expected).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars Squad boundary verified (${allRows.length} rows; digest ` +
  `${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  assert(args[index + 1] && !args[index + 1].startsWith("--"),
    `${flag} requires a value`);
  return args[index + 1];
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function validEnvelope(row) {
  return row
    && /^[a-z0-9][a-z0-9._:-]*$/u.test(row.id)
    && row.schema_revision === "starclock.currency-wars-row.v1"
    && typeof row.name_en === "string" && row.name_en.length > 0
    && typeof row.name_zh_cn === "string" && row.name_zh_cn.length > 0
    && typeof row.summary_en === "string" && row.summary_en.length > 0
    && typeof row.summary_zh_cn === "string" && row.summary_zh_cn.length > 0
    && Array.isArray(row.source_refs) && row.source_refs.length > 0
    && Array.isArray(row.tags);
}

function unique(values) {
  return new Set(values).size === values.length;
}

function compare(left, right) {
  return left.localeCompare(right);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
