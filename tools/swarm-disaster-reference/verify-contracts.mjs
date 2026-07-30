#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifestPath =
  "content-manifests/swarm-disaster-v1/content-manifest.json";
const manifest = json(manifestPath);
const schema = json(
  "content-manifests/swarm-disaster-v1/normalized-schema.json",
);
const authoring = json(
  "content-manifests/swarm-disaster-v1/authoring-contract.json",
);
const fixtures = json(
  "content-manifests/swarm-disaster-v1/fixture-contract.json",
);

assert(sha256(manifestPath)
  === "e466cae0481d93241eaadf6d894b82898d47c9d4863fea262134cbbac10b8850",
"contracts are not bound to the frozen P0-B3 content manifest");
assert(schema.bound_content_manifest_sha256 === sha256(manifestPath),
  "normalized schema manifest binding drift");
assert(schema.schema_revision === "starclock.swarm-disaster-normalized-schema.v1"
  && authoring.schema_revision === "starclock.swarm-disaster-authoring-contract.v1"
  && fixtures.schema_revision === "starclock.swarm-disaster-fixture-contract.v1",
"unsupported Goal 09 contract revision");
assert([schema, authoring, fixtures].every(({ goal_id: goalId }) =>
  goalId === "swarm-disaster-reference-v1"),
"Goal 09 contract identity drift");

const files = schema.files;
assert(files.length === 64, "normalized file-family denominator drift");
assert(unique(files.map(({ file }) => file)), "duplicate normalized file name");
assert(files.every(({
  file,
  record_kind: recordKind,
  phase,
  manifest_category_inputs: inputs,
  ordering_keys: keys,
  required_domain_fields: fields,
}) =>
  /^[a-z0-9-]+\.json$/u.test(file)
    && /^[A-Z][A-Za-z0-9]+$/u.test(recordKind)
    && /^P[12]-B[1-9]$/u.test(phase)
    && Array.isArray(inputs)
    && Array.isArray(keys)
    && keys.length > 0
    && Array.isArray(fields)
    && fields.length > 0),
"normalized file contract is incomplete");
const categoryUses = new Map();
for (const file of files)
  for (const categoryId of file.manifest_category_inputs) {
    assert(manifest.categories[categoryId] !== undefined,
      `unknown manifest category input ${categoryId}`);
    categoryUses.set(categoryId, (categoryUses.get(categoryId) ?? 0) + 1);
  }
for (const categoryId of Object.keys(manifest.categories))
  assert((categoryUses.get(categoryId) ?? 0) >= 1,
    `manifest category has no normalized mapping ${categoryId}`);

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
assert(schema.common_envelope.ownership.enum.join(",") === "SwarmDisaster,Shared",
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
assert(schema.reconciliation_policy.checkpoint_commit
  === manifest.exclusions.gold_checkpoint.commit
  && schema.reconciliation_policy.checkpoint_manifest_sha256
    === manifest.exclusions.gold_checkpoint.manifest_sha256,
"Goal 08 reconciliation checkpoint drift");
assert(schema.reconciliation_policy.join_key.join(",")
  === "source_path,row_locator,evidence_sha256",
"reconciliation join key drift");
assert(schema.reconciliation_policy.conflict_behavior.startsWith("Blocked"),
  "reconciliation conflicts do not fail closed");
assert(schema.reconciliation_policy.required_receipt_fields.join(",")
  === [
    "id", "source_path", "row_locator", "evidence_sha256",
    "goal08_checkpoint_commit", "goal08_ownership", "goal09_ownership",
    "outcome", "note",
  ].join(","),
"reconciliation receipt envelope drift");

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
assert(authoring.isolation.project.startsWith("config/swarm-disaster/")
  && authoring.isolation.workbook_root.startsWith("config/swarm-disaster/")
  && authoring.isolation.generated_root.startsWith(
    "config/swarm-disaster-generated",
  )
  && authoring.isolation.forbidden_outputs.every((output) =>
    !output.startsWith("config/swarm-disaster/")),
"Goal 09 authoring paths are not isolated");
assert(authoring.generation.overwrite_existing_target === false
  && authoring.generation.patch_designer_workbook === false
  && authoring.generation.double_generation_byte_identical === true,
"workbook generation safety drift");
assert(authoring.sheet_contract.data_start_row === 8
  && authoring.sheet_contract.freeze_panes === "A8"
  && authoring.sheet_contract.canonical_decimal_cells === "text",
"workbook sheet contract drift");
assert(authoring.reconciliation_sheet.goal08_checkpoint_commit
  === schema.reconciliation_policy.checkpoint_commit
  && authoring.reconciliation_sheet.conflict_behavior.includes("Block"),
"workbook reconciliation contract drift");

const manifestFamilies = manifest.categories.semantic_fixture_families.records
  .map(({ id }) => id).sort();
const contractFamilies = fixtures.required_families.map(({ id }) => id).sort();
assert(JSON.stringify(manifestFamilies) === JSON.stringify(contractFamilies),
  "semantic fixture-family denominator drift");
assert(fixtures.minimum_cases_per_family === 1
  && fixtures.required_families.length === 23
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
assert(fixtures.coverage_rule.reconciliation_coverage.includes("Goal 08"),
  "fixture coverage omits shared-DLC reconciliation");

console.log(
  "Swarm Disaster normalized/authoring/fixture contracts verified " +
  "(64 files; 4 isolated workbooks; 23 semantic families; Goal 08 receipts).",
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
