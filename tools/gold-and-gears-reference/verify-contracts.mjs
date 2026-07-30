#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifestPath =
  "content-manifests/gold-and-gears-v1/content-manifest.json";
const manifest = json(manifestPath);
const schema = json(
  "content-manifests/gold-and-gears-v1/normalized-schema.json",
);
const authoring = json(
  "content-manifests/gold-and-gears-v1/authoring-contract.json",
);
const fixtures = json(
  "content-manifests/gold-and-gears-v1/fixture-contract.json",
);

assert(sha256(manifestPath)
  === "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085",
"contracts are not bound to the P1-B5 amended content manifest");
assert(schema.schema_revision === "starclock.gold-and-gears-normalized-schema.v1"
  && authoring.schema_revision === "starclock.gold-and-gears-authoring-contract.v1"
  && fixtures.schema_revision === "starclock.gold-and-gears-fixture-contract.v1",
"unsupported Goal 08 contract revision");
assert([schema, authoring, fixtures].every(({ goal_id: goalId }) =>
  goalId === "gold-and-gears-reference-v1"),
"Goal 08 contract identity drift");

const files = schema.files;
assert(files.length === 51, "normalized file-family denominator drift");
assert(unique(files.map(({ file }) => file)), "duplicate normalized file name");
assert(files.every(({ file, record_kind: recordKind, phase, ordering_keys: keys }) =>
  /^[a-z0-9-]+\.json$/u.test(file)
    && /^[A-Z][A-Za-z]+$/u.test(recordKind)
    && /^P[12]-B[1-8]$/u.test(phase)
    && Array.isArray(keys)
    && keys.length > 0),
"normalized file contract is incomplete");
const categoryUses = new Map();
for (const file of files)
  for (const categoryId of file.manifest_category_inputs) {
    assert(manifest.categories[categoryId] !== undefined,
      `unknown manifest category input ${categoryId}`);
    categoryUses.set(categoryId, (categoryUses.get(categoryId) ?? 0) + 1);
  }
for (const categoryId of Object.keys(manifest.categories)) {
  const expected = categoryId === "boss_choices" ? 2 : 1;
  assert(categoryUses.get(categoryId) === expected,
    `manifest category mapping drift ${categoryId}`);
}

const requiredEnvelope = [
  "id", "schema_revision", "kind", "name_en", "name_zh_cn", "summary_en",
  "summary_zh_cn", "ownership", "coverage_state", "evidence_quality",
  "source_refs", "tags",
];
assert(JSON.stringify(schema.common_envelope.required_fields)
  === JSON.stringify(requiredEnvelope), "normalized common envelope drift");
assert(JSON.stringify(schema.common_envelope.evidence_quality.enum)
  === JSON.stringify([
    "ExactStructured",
    "ExactPublicText",
    "Observed",
    "ApproximateFromReleasedText",
    "ProjectPolicy",
  ]), "evidence-quality vocabulary drift");
assert(schema.common_envelope.ownership.enum.join(",") === "GoldAndGears,Shared",
  "ownership vocabulary drift");
assert(schema.common_envelope.coverage_state.enum.includes("Blocked")
  && !schema.common_envelope.coverage_state.enum.includes("GoldenVerified"),
"reference-only coverage vocabulary drift");

const decimalPattern = new RegExp(schema.types.canonical_decimal.pattern, "u");
for (const valid of ["0", "1", "-1", "0.5", "-0.5", "100.0001"])
  assert(decimalPattern.test(valid), `canonical decimal rejects ${valid}`);
for (const invalid of [
  "-0", "+1", "01", "1.", ".5", "1.0", "1.20", "1e2", "NaN", "Infinity",
])
  assert(!decimalPattern.test(invalid), `canonical decimal accepts ${invalid}`);
assert(schema.canonical_encoding.encoding === "UTF-8"
  && schema.canonical_encoding.line_endings === "LF"
  && schema.canonical_encoding.indent_spaces === 2
  && schema.canonical_encoding.terminal_newline === true,
"canonical byte encoding drift");
assert(schema.topology_policy.edge_evidence_quality === "ProjectPolicy"
  && schema.topology_policy.algorithm
    === "forward-nearest-column-within-one-row-v1"
  && schema.topology_policy.replacement_condition.length > 0,
"bounded topology-edge policy drift");
assert(schema.topology_policy.empty_block_type_policy.no_legal_candidate
  .includes("never choose another mode"),
"topology fallback is not fail-closed");

assert(authoring.authority.authoritative_format === "xlsx"
  && authoring.authority.editor === "python-openpyxl"
  && authoring.authority.editor_version === "3.1.5"
  && authoring.authority.schema_exporter_version === "0.3.0"
  && authoring.authority.production_artifact === "sora"
  && authoring.authority.json_role === "research-staging-debug-only"
  && authoring.authority.runtime_loading === false,
"Excel/Sora authoring authority drift");
assert(authoring.workbooks.length === 4
  && unique(authoring.workbooks.map(({ file }) => file)),
"workbook-family denominator drift");
const workbookFiles = authoring.workbooks.flatMap(({ normalized_files: normalized }) =>
  normalized);
assert(unique(workbookFiles) && workbookFiles.length === files.length,
  "normalized file appears in multiple workbook families");
assert(JSON.stringify([...workbookFiles].sort())
  === JSON.stringify(files.map(({ file }) => file).sort()),
"workbook families do not cover every normalized file");
assert(authoring.isolation.project.startsWith("config/gold-and-gears/")
  && authoring.isolation.workbook_root.startsWith("config/gold-and-gears/")
  && authoring.isolation.generated_root.startsWith(
    "config/gold-and-gears-generated",
  ),
"Goal 08 authoring paths are not isolated");
assert(authoring.generation.overwrite_existing_target === false
  && authoring.generation.patch_designer_workbook === false
  && authoring.generation.double_generation_byte_identical === true,
"workbook generation safety drift");
assert(authoring.sheet_contract.data_start_row === 8
  && authoring.sheet_contract.freeze_panes === "A8"
  && authoring.sheet_contract.canonical_decimal_cells === "text",
"workbook sheet contract drift");

const manifestFamilies = manifest.categories.semantic_fixture_families.records
  .map(({ id }) => id).sort();
const contractFamilies = fixtures.required_families.map(({ id }) => id).sort();
assert(JSON.stringify(manifestFamilies) === JSON.stringify(contractFamilies),
  "semantic fixture-family denominator drift");
assert(fixtures.minimum_cases_per_family === 1
  && fixtures.required_families.length === 18
  && fixtures.required_families.every(({ minimum_cases: minimum, must_cover: cover }) =>
    minimum >= 1 && Array.isArray(cover) && cover.length > 0),
"semantic fixture minimum contract drift");
assert(fixtures.fixture_role.includes("no runtime executability claim"),
  "review fixture contract claims runtime execution");
assert(fixtures.approximation.exact_claim_forbidden === true
  && fixtures.approximation.required_fields.join(",")
    === "note,replacement_condition",
"approximation replacement contract drift");
assert(fixtures.determinism.random_selection.includes("explicit seed")
  && fixtures.determinism.no_legal_target.includes("explicit expected fallback"),
"semantic fixture determinism contract drift");

console.log(
  "Gold and Gears normalized/authoring/fixture contracts verified " +
  "(51 files; 4 isolated workbooks; 18 semantic families).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function sha256(relative) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
