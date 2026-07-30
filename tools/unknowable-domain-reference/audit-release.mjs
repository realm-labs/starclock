import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const write = process.argv.includes("--write");
const packRoot = path.join(root, "content-reference", "unknowable-domain-v1");
const manifestPath = path.join(
  root,
  "content-manifests",
  "unknowable-domain-v1",
  "content-manifest.json",
);
const schemaPath = path.join(
  root,
  "content-manifests",
  "unknowable-domain-v1",
  "normalized-schema.json",
);
const soraLockPath = path.join(
  root,
  "config",
  "unknowable-domain-generated",
  "schema.lock",
);
const output = path.join(
  root,
  "evidence",
  "unknowable-domain-reference-v1",
  "release-audit.json",
);

const manifest = json(manifestPath);
const schema = json(schemaPath);
const sora = json(soraLockPath).schema;
const valuesByFile = new Map(
  schema.files.map(({ file }) => [file, json(path.join(packRoot, file))]),
);
const sources = valuesByFile.get("sources.json");
const coverage = valuesByFile.get("coverage.json");
const gaps = valuesByFile.get("research-gaps.json");
const fixtureFamilies = valuesByFile.get("semantic-fixture-families.json");
const fixtures = valuesByFile.get("review-fixtures.json");
const rules = valuesByFile.get("mechanic-rules.json");
const receipts = valuesByFile.get("reconciliation-receipts.json");
const quality = new Set(schema.common_envelope.evidence_quality.enum);
const ownership = new Set(schema.common_envelope.ownership.enum);
const commonFields = schema.common_envelope.required_fields;
const excludedSourceKeys = new Set([
  ...manifest.exclusions.named_mode_source_files,
  ...manifest.exclusions.presentation_account_source_files,
].map(({ source, evidence_sha256: digest }) => `${source}\0${digest}`));
const forbiddenModePrefixes = manifest.exclusions.mode_prefixes;

assert(schema.files.length === 65, "normalized file-family denominator differs");
assert(Array.isArray(sources) && sources.length === 4473, "source denominator differs");
const sourceById = uniqueMap(sources, ({ source_id: sourceId }) => sourceId, "source");
for (const source of sources) {
  assert(source.game_version === "4.4", `${source.id}: game version differs`);
  for (const field of [
    "repository",
    "revision",
    "path",
    "locator",
    "access_date",
    "mechanism_quality",
  ]) {
    assert(nonempty(source[field]), `${source.id}: ${field} is empty`);
  }
  assert(/^[0-9a-f]{64}$/u.test(source.sha256), `${source.id}: digest invalid`);
  assert(quality.has(source.evidence_quality), `${source.id}: quality invalid`);
  if (
    source.evidence_quality === "ApproximateFromReleasedText" ||
    source.evidence_quality === "ProjectPolicy"
  ) {
    assert(nonempty(source.note), `${source.id}: policy note missing`);
    assert(
      nonempty(source.replacement_condition),
      `${source.id}: policy replacement condition missing`,
    );
  }
}

const commonRows = [];
const contentById = new Map();
const referencedSources = new Set();
const qualityCounts = {};
const ownershipCounts = {};
let sourceReferenceCount = 0;
for (const { file } of schema.files) {
  if (["manifest.json", "pack-index.json"].includes(file)) continue;
  const rows = valuesByFile.get(file);
  assert(Array.isArray(rows), `${file}: expected an array`);
  for (const row of rows) {
    const label = `${file}/${row.id}`;
    for (const field of commonFields) {
      assert(Object.hasOwn(row, field), `${label}: common field ${field} missing`);
    }
    assert(
      row.schema_revision === schema.common_envelope.schema_revision.value,
      `${label}: row schema revision differs`,
    );
    assert(!contentById.has(row.id), `${label}: duplicate global stable ID`);
    contentById.set(row.id, { file, row });
    commonRows.push(row);
    for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"]) {
      assert(nonempty(row[field]), `${label}: bilingual field ${field} is empty`);
    }
    assert(/\p{Script=Han}/u.test(row.name_zh_cn), `${label}: Chinese name has no Han text`);
    assert(
      /\p{Script=Han}/u.test(row.summary_zh_cn),
      `${label}: Chinese summary has no Han text`,
    );
    assert(ownership.has(row.ownership), `${label}: ownership is invalid`);
    assert(row.coverage_state === "DataReady", `${label}: row is not DataReady`);
    assert(quality.has(row.evidence_quality), `${label}: evidence quality is invalid`);
    assert(
      Array.isArray(row.tags) &&
        new Set(row.tags).size === row.tags.length &&
        JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
      `${label}: tags are not unique and lexicographically ordered`,
    );
    assert(
      Array.isArray(row.source_refs) && row.source_refs.length > 0,
      `${label}: provenance is empty`,
    );
    const rowSourceIds = new Set();
    for (const sourceRef of row.source_refs) {
      assert(
        !rowSourceIds.has(sourceRef.source_id),
        `${label}: duplicate source ref ${sourceRef.source_id}`,
      );
      rowSourceIds.add(sourceRef.source_id);
      const source = sourceById.get(sourceRef.source_id);
      assert(source, `${label}: source ${sourceRef.source_id} is not registered`);
      for (const field of [
        "repository",
        "revision",
        "path",
        "locator",
        "sha256",
        "access_date",
        "game_version",
        "evidence_quality",
        "mechanism_quality",
      ]) {
        assert(
          sourceRef[field] === source[field],
          `${label}: source ${sourceRef.source_id} ${field} differs`,
        );
      }
      assert(
        !excludedSourceKeys.has(`${sourceRef.path}\0${sourceRef.sha256}`),
        `${label}: excluded source evidence leaked`,
      );
      assert(
        !forbiddenModePrefixes.some((prefix) => sourceRef.path.includes(prefix)),
        `${label}: other-mode source path leaked`,
      );
      if (
        sourceRef.evidence_quality === "ApproximateFromReleasedText" ||
        sourceRef.evidence_quality === "ProjectPolicy"
      ) {
        assert(nonempty(sourceRef.note), `${label}: policy source note missing`);
        assert(
          nonempty(sourceRef.replacement_condition),
          `${label}: policy source replacement condition missing`,
        );
        assert(sourceRef.note === source.note, `${label}: policy source note differs`);
        assert(
          sourceRef.replacement_condition === source.replacement_condition,
          `${label}: policy source replacement condition differs`,
        );
      }
      referencedSources.add(sourceRef.source_id);
      sourceReferenceCount += 1;
    }
    if (row.evidence_quality === "ProjectPolicy") {
      assert(
        row.source_refs.some(({ evidence_quality: value }) => value === "ProjectPolicy"),
        `${label}: policy row has no policy provenance`,
      );
    }
    auditOwnershipFields(row, label);
    qualityCounts[row.evidence_quality] = (qualityCounts[row.evidence_quality] ?? 0) + 1;
    ownershipCounts[row.ownership] = (ownershipCounts[row.ownership] ?? 0) + 1;
  }
}
assert(commonRows.length === 17147, "common normalized row denominator differs");
assert(
  ownershipCounts.UnknowableDomain === 16602 && ownershipCounts.Shared === 545,
  "normalized ownership denominator differs",
);
assert(sourceReferenceCount === 23167, "provenance binding denominator differs");
for (const source of sources) {
  assert(referencedSources.has(source.source_id), `${source.id}: orphan provenance row`);
}

const manifestRows = [];
const manifestByKey = new Map();
const manifestOwnership = {};
for (const [categoryId, category] of Object.entries(manifest.categories)) {
  assert(category.id === categoryId, `${categoryId}: manifest category identity differs`);
  assert(category.records.length === category.count, `${categoryId}: category count differs`);
  for (const row of category.records) {
    const key = `${categoryId}/${row.id}`;
    assert(!manifestByKey.has(key), `${key}: duplicate manifest obligation`);
    manifestByKey.set(key, row);
    assert(nonempty(row.source), `${key}: source locator missing`);
    assert(/^[0-9a-f]{64}$/u.test(row.evidence_sha256), `${key}: digest invalid`);
    assert(quality.has(row.evidence_quality), `${key}: evidence quality invalid`);
    assert(ownership.has(row.ownership), `${key}: ownership invalid`);
    assert(
      ["Direct", "ExplicitModeSelector", "Referenced"].includes(row.reachability),
      `${key}: reachability invalid`,
    );
    manifestOwnership[row.ownership] = (manifestOwnership[row.ownership] ?? 0) + 1;
    manifestRows.push(row);
  }
}
assert(manifestRows.length === 5377, "manifest obligation denominator differs");
assert(manifest.counts.categories === 43, "manifest category aggregate differs");
assert(manifest.counts.records === manifestRows.length, "manifest aggregate count differs");
assert(
  manifestOwnership.UnknowableDomain === 5243 && manifestOwnership.Shared === 134,
  "frozen manifest ownership denominator differs",
);
assert(
  JSON.stringify(manifest.counts.ownership) ===
    JSON.stringify(sortedObject(manifestOwnership)),
  "manifest ownership aggregate differs",
);

const coverageByKey = uniqueMap(
  coverage,
  ({ manifest_category: category, manifest_record_id: recordId }) =>
    `${category}/${recordId}`,
  "coverage obligation",
);
assert(coverageByKey.size === 5377, "coverage obligation denominator differs");
let coverageDataBindings = 0;
for (const [key, obligation] of manifestByKey) {
  const row = coverageByKey.get(key);
  assert(row, `${key}: coverage row missing`);
  assert(row.source_locator === obligation.source, `${key}: source locator differs`);
  assert(
    row.source_evidence_sha256 === obligation.evidence_sha256,
    `${key}: source digest differs`,
  );
  assert(row.state === "DataReady", `${key}: coverage is not DataReady`);
  assert(row.blocking_gap_ids.length === 0, `${key}: blocking gap leaked`);
  assert(
    Array.isArray(row.data_ids) &&
      row.data_ids.length > 0 &&
      new Set(row.data_ids).size === row.data_ids.length &&
      JSON.stringify(row.data_ids) === JSON.stringify([...row.data_ids].sort()),
    `${key}: data IDs are empty, duplicate or unordered`,
  );
  for (const id of row.data_ids) {
    assert(contentById.has(id), `${key}: data ID ${id} does not resolve`);
    coverageDataBindings += 1;
  }
}
assert(coverageDataBindings === 6909, "coverage data-binding denominator differs");

const fixtureById = uniqueMap(fixtures, ({ id }) => id, "fixture");
const familyById = uniqueMap(
  fixtureFamilies,
  ({ source_id: familyId }) => familyId,
  "fixture family",
);
assert(
  fixtures.length === 24 &&
    fixtureById.size === 24 &&
    fixtureFamilies.length === 24 &&
    familyById.size === 24,
  "fixture denominator differs",
);
let familyInputReferenceCount = 0;
for (const family of fixtureFamilies) {
  assert(family.runtime_executable === false, `${family.id}: runtime fixture leaked`);
  for (const id of family.selected_source_record_ids) {
    assert(contentById.has(id), `${family.id}: selected input ${id} does not resolve`);
    familyInputReferenceCount += 1;
  }
}
let fixtureInputReferenceCount = 0;
let fixtureEvidenceReferenceCount = 0;
for (const fixture of fixtures) {
  assert(familyById.has(fixture.family_id), `${fixture.id}: family does not resolve`);
  for (const id of fixture.source_record_ids) {
    assert(contentById.has(id), `${fixture.id}: fixture input ${id} does not resolve`);
    fixtureInputReferenceCount += 1;
  }
  for (const sourceRef of fixture.source_refs) {
    assert(sourceById.has(sourceRef.source_id), `${fixture.id}: evidence does not resolve`);
    fixtureEvidenceReferenceCount += 1;
  }
  for (const fact of fixture.expected_facts) {
    assert(fact.runtime_claim === false, `${fixture.id}: runtime fact leaked`);
  }
}
assert(
  familyInputReferenceCount === 66 &&
    fixtureInputReferenceCount === 66 &&
    fixtureEvidenceReferenceCount === 126,
  "fixture reference denominator differs",
);

let ruleReferenceCount = 0;
assert(rules.length === 41, "mechanic-rule denominator differs");
for (const rule of rules) {
  const sourceFile = contentById.get(rule.source_file_id);
  assert(sourceFile, `${rule.id}: source file ${rule.source_file_id} does not resolve`);
  assert(sourceFile.file === "mechanic-source-files.json", `${rule.id}: source file differs`);
  assert(rule.runtime_lowered === false, `${rule.id}: runtime lowering leaked`);
  for (const id of rule.fixture_ids) {
    assert(fixtureById.has(id), `${rule.id}: fixture ${id} does not resolve`);
    ruleReferenceCount += 1;
  }
  ruleReferenceCount += 1;
}
assert(ruleReferenceCount === 82, "mechanic rule reference denominator differs");

let gapAffectedReferenceCount = 0;
for (const gap of gaps) {
  assert(
    gap.blocking === false && gap.state === "PolicyBound",
    `${gap.id}: research gap blocks`,
  );
  assert(gap.owner === "G10-P4-B2", `${gap.id}: research owner differs`);
  assert(nonempty(gap.known_fact), `${gap.id}: known fact missing`);
  assert(nonempty(gap.policy), `${gap.id}: policy missing`);
  assert(nonempty(gap.replacement_condition), `${gap.id}: replacement condition missing`);
  for (const id of gap.affected_data_ids) {
    assert(contentById.has(id), `${gap.id}: affected record ${id} does not resolve`);
    gapAffectedReferenceCount += 1;
  }
}
assert(gaps.length === 24 && gapAffectedReferenceCount === 66, "gap denominator differs");

const reconciliationOutcomes = {};
for (const receipt of receipts) {
  assert(receipt.blocking === false, `${receipt.id}: reconciliation receipt blocks`);
  assert(
    ["MatchedShared", "DivergentRepresentation"].includes(receipt.outcome),
    `${receipt.id}: unresolved reconciliation outcome`,
  );
  const key = `${receipt.goal10_category}/${receipt.goal10_record_id}`;
  const obligation = manifestByKey.get(key);
  assert(obligation, `${receipt.id}: Goal 10 obligation ${key} does not resolve`);
  assert(
    obligation.source === `${receipt.source_path}#${receipt.row_locator}`,
    `${receipt.id}: Goal 10 source locator differs`,
  );
  assert(
    obligation.evidence_sha256 === receipt.evidence_sha256,
    `${receipt.id}: Goal 10 source digest differs`,
  );
  assert(
    obligation.ownership === receipt.goal10_ownership,
    `${receipt.id}: Goal 10 ownership differs`,
  );
  reconciliationOutcomes[receipt.outcome] =
    (reconciliationOutcomes[receipt.outcome] ?? 0) + 1;
}
assert(receipts.length === 155, "reconciliation receipt denominator differs");
assert(
  reconciliationOutcomes.MatchedShared === 143 &&
    reconciliationOutcomes.DivergentRepresentation === 12,
  "reconciliation outcome denominator differs",
);

const typedReferences = auditTypedSoraReferences();
assert(
  typedReferences.fields === 34 && typedReferences.bindings === 3082,
  "typed Sora reference denominator differs",
);
assert(
  manifest.exclusions.named_mode_source_files.length ===
    manifest.exclusions.named_mode_source_count,
  "named other-mode exclusion denominator differs",
);
assert(
  manifest.exclusions.presentation_account_source_files.length ===
    manifest.exclusions.presentation_account_source_count,
  "presentation/account exclusion denominator differs",
);

const report = {
  schema_revision: "starclock.unknowable-domain-release-audit.v1",
  goal_id: "unknowable-domain-reference-v1",
  audited_at: "2026-07-29",
  result: "pass",
  inputs: {
    content_manifest_sha256: sha256(fs.readFileSync(manifestPath)),
    normalized_schema_sha256: sha256(fs.readFileSync(schemaPath)),
    sora_schema_lock_sha256: sha256(fs.readFileSync(soraLockPath)),
  },
  exact_once: {
    manifest_categories: Object.keys(manifest.categories).length,
    manifest_obligations: manifestRows.length,
    coverage_rows: coverage.length,
    coverage_data_bindings: coverageDataBindings,
    data_ready_obligations: coverage.filter(({ state }) => state === "DataReady").length,
    coverage_percent: "100",
    duplicate_obligations: 0,
    blocking_gaps: 0,
  },
  normalized: {
    files: schema.files.length,
    common_rows: commonRows.length,
    source_rows: sources.length,
    mechanic_rules: rules.length,
    semantic_fixture_families: fixtureFamilies.length,
    semantic_review_fixtures: fixtures.length,
    research_gaps: gaps.length,
    reconciliation_receipts: receipts.length,
    ownership: sortedObject(ownershipCounts),
    evidence_quality: sortedObject(qualityCounts),
  },
  references: {
    source_ref_bindings: sourceReferenceCount,
    referenced_source_rows: referencedSources.size,
    typed_sora_fields: typedReferences.fields,
    typed_sora_bindings: typedReferences.bindings,
    rule_owner_and_fixture_bindings: ruleReferenceCount,
    family_input_bindings: familyInputReferenceCount,
    fixture_input_bindings: fixtureInputReferenceCount,
    fixture_evidence_bindings: fixtureEvidenceReferenceCount,
    research_gap_affected_bindings: gapAffectedReferenceCount,
    unresolved: 0,
  },
  bilingual: {
    rows_checked: commonRows.length,
    missing_english_or_chinese_fields: 0,
    chinese_fields_without_han_text: 0,
  },
  ownership_and_boundary: {
    allowed_ownership: [...ownership],
    manifest_ownership: sortedObject(manifestOwnership),
    normalized_ownership: sortedObject(ownershipCounts),
    reconciliation_outcomes: sortedObject(reconciliationOutcomes),
    named_other_mode_exclusions: manifest.exclusions.named_mode_source_files.length,
    presentation_account_exclusions:
      manifest.exclusions.presentation_account_source_files.length,
    other_mode_source_rows: 0,
    presentation_account_rows: 0,
    runtime_lowered_rules: 0,
    runtime_executable_fixture_families: 0,
    runtime_claims: 0,
  },
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, encoded);
} else {
  assert(fs.existsSync(output), "release audit evidence is missing; run with --write");
  assert(fs.readFileSync(output, "utf8") === encoded, "release audit evidence drifted");
}
console.log(
  `Unknowable Domain release audit passed (${manifestRows.length} exact-once ` +
    `obligations; ${commonRows.length} bilingual rows; ${sourceReferenceCount} ` +
    `provenance bindings; ${typedReferences.bindings} typed references; zero leaks).`,
);

function auditOwnershipFields(value, label) {
  if (Array.isArray(value)) {
    for (const entry of value) auditOwnershipFields(entry, label);
    return;
  }
  if (!value || typeof value !== "object") return;
  for (const [key, entry] of Object.entries(value)) {
    if (key === "ownership") {
      assert(ownership.has(entry), `${label}: nested ownership ${entry} leaked`);
    } else if (key === "mode_owner" || key === "profile_owner") {
      throw new Error(`${label}: ${key} leaked`);
    }
    auditOwnershipFields(entry, label);
  }
}

function auditTypedSoraReferences() {
  const debugRoot = path.join(root, "config", "unknowable-domain-generated", "debug-json");
  const idsByTable = new Map();
  const debugByTable = new Map();
  for (const table of sora.tables) {
    const debug = json(path.join(debugRoot, `${table.name}.json`)).table.rows;
    debugByTable.set(table.name, debug);
    idsByTable.set(
      table.name,
      new Set(debug.map(({ values }) => values.id.Integer)),
    );
  }
  let fields = 0;
  let bindings = 0;
  for (const table of sora.tables) {
    for (const field of table.fields) {
      const target = referenceTarget(field.ty);
      if (!target) continue;
      fields += 1;
      const targetIds = idsByTable.get(target);
      assert(targetIds, `${table.name}.${field.name}: target table ${target} missing`);
      for (const row of debugByTable.get(table.name)) {
        for (const id of referenceValues(row.values[field.name])) {
          assert(
            targetIds.has(id),
            `${table.name}.${field.name}: target ${target}/${id} missing`,
          );
          bindings += 1;
        }
      }
    }
  }
  return { fields, bindings };
}

function referenceTarget(type) {
  if (!type || typeof type !== "object") return undefined;
  if (type.Ref) return type.Ref.table;
  if (type.Optional) return referenceTarget(type.Optional);
  if (type.List) return referenceTarget(type.List);
  return undefined;
}

function referenceValues(value) {
  if (
    value === undefined ||
    value === null ||
    value === "Null" ||
    value.Null !== undefined
  ) {
    return [];
  }
  if (value.Integer !== undefined) return [value.Integer];
  if (value.List) return value.List.flatMap(referenceValues);
  throw new Error(`unsupported Sora reference value ${JSON.stringify(value)}`);
}

function uniqueMap(values, keyOf, label) {
  const result = new Map();
  for (const value of values) {
    const key = keyOf(value);
    assert(!result.has(key), `duplicate ${label} ${key}`);
    result.set(key, value);
  }
  return result;
}

function sortedObject(value) {
  return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) => left.localeCompare(right)),
  );
}

function nonempty(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
