#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const manifestRoot = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join("tools", "galactic-baseballer-reference", "contracts.mjs"),
  "--check",
], { cwd: root, stdio: "inherit" });

const read = async (file) =>
  JSON.parse(await readFile(path.join(manifestRoot, file), "utf8"));
const schema = await read("normalized-schema.json");
const authoring = await read("authoring-contract.json");
const fixtures = await read("fixture-contract.json");
const approximationRegister = await read("approximation-register.json");

assert(
  schema.schema_revision
    === "starclock.galactic-baseballer-normalized-schema.v1",
  "normalized schema revision drift",
);
assert(schema.files.length === 40, "normalized file family count drift");
assert(
  new Set(schema.files.map(({ file }) => file)).size === schema.files.length,
  "duplicate normalized file contract",
);
assert(
  schema.types.canonical_decimal.storage === "string"
    && schema.types.source_numeric_id.storage === "string",
  "canonical numeric storage drift",
);
assert(
  schema.types.labeled_rng_stream.sampling
    === "project integer sampling only",
  "RNG sampling contract drift",
);
assert(
  authoring.authority.authoritative_format === "xlsx"
    && authoring.authority.editor_version === "3.1.5"
    && authoring.authority.schema_exporter_version === "0.3.0"
    && authoring.authority.production_artifact === "sora"
    && authoring.authority.runtime_loading === false,
  "authoring authority drift",
);
assert(authoring.workbooks.length === 4, "workbook family count drift");
const workbookFiles = authoring.workbooks.flatMap(
  ({ normalized_files: files }) => files,
);
assert(
  workbookFiles.length === schema.files.length
    && new Set(workbookFiles).size === schema.files.length,
  "normalized file/workbook exact-once mapping drift",
);
assert(
  authoring.visual_review.render_every_sheet
    && authoring.visual_review.render_every_schema_column,
  "visual review coverage drift",
);

assert(
  fixtures.required_families.length === 20,
  "semantic fixture family count drift",
);
const manifest = await read("content-manifest.json");
const manifestFamilies = manifest.categories.semantic_fixture_families.records
  .map(({ id }) => id)
  .sort();
const contractFamilies = fixtures.required_families.map(({ id }) => id);
assert(
  JSON.stringify([...contractFamilies].sort()) === JSON.stringify(manifestFamilies),
  "fixture family/manifest reconciliation drift",
);
assert(
  fixtures.required_families.every(
    ({ minimum_cases: count, must_cover: facts }) =>
      count >= 1 && facts.length >= 4,
  ),
  "fixture minimum/must-cover drift",
);
for (const requiredField of [
  "trigger_point",
  "state_owner",
  "preconditions",
  "input",
  "ordered_operations",
  "expected_facts",
  "evidence_quality",
]) {
  assert(
    fixtures.required_fields.includes(requiredField),
    `fixture field missing: ${requiredField}`,
  );
}

assert(
  approximationRegister.records.length === 8,
  "initial approximation count drift",
);
for (const record of approximationRegister.records) {
  for (const field of approximationRegister.required_fields) {
    assert(record[field] !== undefined, `missing ${field}: ${record.id}`);
  }
  assert(
    record.rejected_alternatives.length >= 2,
    `fewer than two rejected alternatives: ${record.id}`,
  );
  assert(
    record.affected_fixture_ids.length >= 1
      && record.affected_fixture_ids.every((id) =>
        contractFamilies.includes(id)),
    `invalid affected fixture: ${record.id}`,
  );
  assert(
    record.evidence_quality === "ProjectPolicy"
      && record.mechanism_quality === "PolicyBoundary",
    `policy mislabeled as evidence: ${record.id}`,
  );
  assert(
    typeof record.replacement_condition === "string"
      && record.replacement_condition.length > 0,
    `missing replacement condition: ${record.id}`,
  );
}

console.log(
  "Galactic Baseballer contracts verified: 40 files, 4 workbooks, "
  + "20 fixture families, 8 explicit ProjectPolicy boundaries",
);
