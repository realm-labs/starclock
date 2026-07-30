#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(".");
const manifestRoot = path.join(
  root,
  "content-manifests",
  "anomaly-arbitration-v1",
);
const paths = {
  manifest: path.join(manifestRoot, "content-manifest.json"),
  schema: path.join(manifestRoot, "normalized-schema.json"),
  authoring: path.join(manifestRoot, "authoring-contract.json"),
  fixture: path.join(manifestRoot, "fixture-contract.json"),
};
const encoded = Object.fromEntries(await Promise.all(
  Object.entries(paths).map(async ([key, filePath]) => [
    key,
    await readFile(filePath),
  ]),
));
const documents = Object.fromEntries(Object.entries(encoded).map(
  ([key, bytes]) => [key, JSON.parse(bytes)],
));
const { manifest, schema, authoring, fixture } = documents;

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function sortedUnique(values, label) {
  assert(
    values.every((value, index) =>
      index === 0 || values[index - 1] < value),
    `${label} is not uniquely sorted`,
  );
}

assert(
  schema.schema_revision
    === "starclock.anomaly-arbitration-normalized-schema.v1",
  "normalized schema revision drift",
);
assert(
  schema.row_schema_revision === "starclock.anomaly-arbitration-row.v1",
  "row schema revision drift",
);
assert(
  authoring.schema_revision
    === "starclock.anomaly-arbitration-authoring-contract.v1",
  "authoring contract revision drift",
);
assert(
  fixture.schema_revision
    === "starclock.anomaly-arbitration-fixture-contract.v1",
  "fixture contract revision drift",
);
for (const document of [schema, authoring, fixture])
  assert(
    document.goal_id === "anomaly-arbitration-reference-v1",
    "contract goal ID drift",
  );

const envelopeFields = schema.common_envelope.required_fields;
assert(
  JSON.stringify(envelopeFields) === JSON.stringify([
    "id",
    "schema_revision",
    "kind",
    "name_en",
    "name_zh_cn",
    "summary_en",
    "summary_zh_cn",
    "ownership",
    "coverage_state",
    "evidence_quality",
    "mechanism_quality",
    "manifest_record_ids",
    "source_refs",
    "tags",
  ]),
  "common bilingual/provenance envelope drift",
);
assert(
  JSON.stringify(schema.common_envelope.ownership.enum)
    === JSON.stringify(["AnomalyArbitration", "Shared"]),
  "normalized ownership enum drift",
);
assert(
  JSON.stringify(schema.common_envelope.evidence_quality.enum)
    === JSON.stringify([
      "ExactStructured",
      "ExactPublicText",
      "Observed",
      "ApproximateFromReleasedText",
      "ProjectPolicy",
    ]),
  "evidence quality enum drift",
);
assert(
  schema.common_envelope.mechanism_quality.enum.length === 6,
  "mechanism quality enum drift",
);

const decimalPattern = new RegExp(schema.types.canonical_decimal.pattern, "u");
for (const valid of ["0", "1", "-1", "0.5", "-0.5", "10.25"])
  assert(decimalPattern.test(valid), `canonical decimal rejected: ${valid}`);
for (const invalid of ["-0", "+1", "01", "1.0", "1e3", ".5", "0.50"])
  assert(!decimalPattern.test(invalid), `noncanonical decimal accepted: ${invalid}`);
assert(
  schema.types.source_ref.required_fields.includes("locator")
    && schema.types.source_ref.required_fields.includes("sha256")
    && schema.types.source_ref.required_fields.includes("evidence_quality")
    && schema.types.source_ref.required_fields.includes("mechanism_quality")
    && schema.types.source_ref.required_fields.includes("note"),
  "row-level source reference contract drift",
);
assert(
  schema.types.approximation.required_fields.includes("alternatives")
    && schema.types.approximation.required_fields
      .includes("affected_fixture_ids")
    && schema.types.approximation.required_fields
      .includes("replacement_condition"),
  "field-level approximation contract drift",
);
assert(
  JSON.stringify(schema.types.reconciliation_receipt.required_fields)
    .includes("source_path")
    && schema.types.reconciliation_receipt.required_fields
      .includes("row_locator")
    && schema.types.reconciliation_receipt.required_fields
      .includes("evidence_sha256")
    && schema.types.reconciliation_receipt.conflict_rule
      .includes("never rewrite another Goal artifact"),
  "shared-source reconciliation contract drift",
);
assert(
  schema.canonical_encoding.encoding === "UTF-8"
    && schema.canonical_encoding.line_endings === "LF"
    && schema.canonical_encoding.unicode_normalization === "NFC"
    && schema.canonical_encoding.terminal_newline === true
    && schema.canonical_encoding.decimal_policy
      === "canonical_decimal strings only",
  "canonical encoding contract drift",
);
assert(schema.lifecycle_contract.runtime_claim === false,
  "schema improperly claims runtime executability");

const expectedFiles = [
  "ability-bindings.json",
  "aggregations.json",
  "battle-events.json",
  "clocks.json",
  "coverage.json",
  "encounter-waves.json",
  "encounters.json",
  "enemies.json",
  "enemy-skills.json",
  "enemy-slots.json",
  "enemy-statuses.json",
  "king-protection.json",
  "king-states.json",
  "loadout-records.json",
  "manifest.json",
  "maze-buff-bindings.json",
  "mechanic-contributions.json",
  "mechanic-rules.json",
  "objectives.json",
  "pack-index.json",
  "participant-policies.json",
  "periods.json",
  "pool-audits.json",
  "profiles.json",
  "progress-records.json",
  "quadrant-options.json",
  "quadrant-selections.json",
  "reconciliation.json",
  "research-gaps.json",
  "review-fixtures.json",
  "sources.json",
  "stage-results.json",
  "stages.json",
  "targets.json",
  "team-slots.json",
  "terminal-outcomes.json",
  "traits.json",
];
const schemaFiles = schema.files.map(({ file }) => file).sort();
assert(
  JSON.stringify(schemaFiles) === JSON.stringify(expectedFiles),
  "normalized file set drift",
);
sortedUnique(schemaFiles, "normalized file set");
for (const file of schema.files) {
  assert(/^P[12]-B[1-6]$/u.test(file.phase),
    `invalid owning phase: ${file.file}`);
  assert(file.ordering_keys.length > 0,
    `missing ordering keys: ${file.file}`);
  for (const categoryId of file.manifest_category_inputs)
    assert(manifest.categories[categoryId] !== undefined,
      `unknown manifest input ${categoryId}: ${file.file}`);
}
const coveredCategories = new Set(schema.files.flatMap(
  ({ manifest_category_inputs: inputs }) => inputs,
));
for (const categoryId of Object.keys(manifest.categories))
  assert(coveredCategories.has(categoryId),
    `manifest category has no normalized file contract: ${categoryId}`);

assert(
  authoring.authority.authoritative_format === "xlsx"
    && authoring.authority.editor === "python-openpyxl"
    && authoring.authority.editor_version === "3.1.5"
    && authoring.authority.schema_exporter_version === "0.3.0"
    && authoring.authority.production_artifact === "sora"
    && authoring.authority.runtime_loading === false,
  "Excel/openpyxl/Sora authority drift",
);
assert(
  authoring.isolation.project === "config/anomaly-arbitration/project.toml"
    && authoring.isolation.generated_root
      === "config/anomaly-arbitration-generated"
    && authoring.isolation.normalized_root
      === "content-reference/anomaly-arbitration-v1",
  "isolated authoring path drift",
);
assert(
  authoring.generation.overwrite_existing_target === false
    && authoring.generation.patch_designer_workbook === false
    && authoring.generation.double_generation_byte_identical === true,
  "no-overwrite/deterministic workbook contract drift",
);
assert(
  JSON.stringify(authoring.workbooks.map(({ file }) => file))
    === JSON.stringify([
      "AnomalyArbitration.xlsx",
      "AnomalyArbitrationBindings.xlsx",
      "AnomalyArbitrationReview.xlsx",
    ]),
  "three-workbook set drift",
);
const workbookFiles = authoring.workbooks.flatMap(
  ({ normalized_files: files }) => files,
);
assert(workbookFiles.length === expectedFiles.length,
  "workbook normalized-file exact-once count drift");
assert(
  JSON.stringify([...workbookFiles].sort()) === JSON.stringify(expectedFiles),
  "workbook normalized-file partition drift",
);
assert(new Set(workbookFiles).size === workbookFiles.length,
  "normalized file appears in multiple workbooks");
assert(
  authoring.sheet_contract.data_start_row === 8
    && authoring.sheet_contract.freeze_panes === "A8"
    && authoring.sheet_contract.canonical_decimal_cells === "text"
    && authoring.sheet_contract.source_numeric_id_cells === "text",
  "Sora/openpyxl sheet contract drift",
);
assert(authoring.visual_review.render_every_sheet === true,
  "visual review does not cover every sheet");

const manifestFamilies = manifest.categories.semantic_fixture_families.records
  .map(({ id }) => id);
const contractFamilies = fixture.required_families.map(({ id }) => id);
assert(
  JSON.stringify(contractFamilies) === JSON.stringify(manifestFamilies),
  "fixture family IDs do not match the frozen manifest",
);
sortedUnique(contractFamilies, "fixture family IDs");
assert(contractFamilies.length === 18, "fixture family denominator drift");
for (const family of fixture.required_families) {
  assert(
    Number.isSafeInteger(family.minimum_cases)
      && family.minimum_cases >= fixture.minimum_cases_per_family,
    `invalid fixture minimum: ${family.id}`,
  );
  assert(family.must_cover.length >= 3,
    `fixture family lacks semantic boundaries: ${family.id}`);
}
assert(
  fixture.required_families.find(({ id }) =>
    id === "empty-pool-proof")?.minimum_cases === 6,
  "empty-pool fixture minimum drift",
);
assert(
  fixture.determinism.battle_boundary.includes("teardown")
    && fixture.determinism.cross_battle_projection.includes("retained best"),
  "battle/cross-battle fixture boundary drift",
);
assert(
  fixture.fixture_role.includes("no runtime executability"),
  "fixture contract improperly claims runtime executability",
);

console.log(
  "Anomaly Arbitration contracts verified: " +
  `${schema.files.length} normalized files, ` +
  `${authoring.workbooks.length} workbooks, ` +
  `${fixture.required_families.length} fixture families; ` +
  `schema ${createHash("sha256").update(encoded.schema).digest("hex")}, ` +
  `authoring ${createHash("sha256").update(encoded.authoring).digest("hex")}, ` +
  `fixtures ${createHash("sha256").update(encoded.fixture).digest("hex")}.`,
);
