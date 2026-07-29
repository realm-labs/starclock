import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const write = process.argv.includes("--write");
const packRoot = path.join(
  root,
  "content-reference",
  "divergent-universe-v1",
);
const manifestPath = path.join(
  root,
  "content-manifests",
  "divergent-universe-v1",
  "content-manifest.json",
);
const schemaPath = path.join(
  root,
  "content-manifests",
  "divergent-universe-v1",
  "normalized-schema.json",
);
const soraLockPath = path.join(
  root,
  "config",
  "divergent-universe-generated",
  "schema.lock",
);
const output = path.join(
  root,
  "evidence",
  "divergent-universe-reference-v1",
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
const enums = new Map(sora.enums.map(({ name, values }) => [name, new Set(values)]));
const quality = enums.get("DuEvidenceQuality");
const ownership = enums.get("DuOwnership");
const coverageStates = enums.get("DuCoverageState");
const commonFields = schema.common_envelope.required_fields;

assert(schema.files.length === 80, "normalized file-family denominator differs");
assert(sora.tables.length === 80, "Sora table denominator differs");
assert(sources.length === 7_620, "source denominator differs");
const sourceById = uniqueMap(sources, ({ source_id: id }) => id, "source");
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
  assert(
    source.repository === "starclock" ||
      (
        source.repository ===
          "https://gitlab.com/Dimbreath/turnbasedgamedata.git" &&
        source.revision ===
          "fd978d6ef09f941fba644c731ab54abd6f7c3568"
      ),
    `${source.id}: source escapes the frozen/public boundary`,
  );
  if (source.evidence_quality === "ProjectPolicy") {
    assert(nonempty(source.note), `${source.id}: policy note missing`);
    assert(
      nonempty(source.replacement_condition),
      `${source.id}: policy replacement condition missing`,
    );
  }
}

const contentById = new Map();
const commonRows = [];
const referencedSources = new Set();
const qualityCounts = {};
const ownershipCounts = {};
const coverageStateCounts = {};
let sourceReferenceCount = 0;
for (const { file } of schema.files) {
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
    assert(
      /\p{Script=Han}/u.test(row.name_zh_cn),
      `${label}: Chinese name has no Han text`,
    );
    assert(
      /\p{Script=Han}/u.test(row.summary_zh_cn),
      `${label}: Chinese summary has no Han text`,
    );
    assert(ownership.has(row.ownership), `${label}: ownership invalid`);
    assert(coverageStates.has(row.coverage_state), `${label}: coverage invalid`);
    assert(quality.has(row.evidence_quality), `${label}: quality invalid`);
    assert(
      Array.isArray(row.tags) &&
        new Set(row.tags).size === row.tags.length &&
        JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
      `${label}: tags are duplicate or unordered`,
    );
    assert(
      Array.isArray(row.source_refs) && row.source_refs.length > 0,
      `${label}: provenance is empty`,
    );
    const rowSources = new Set();
    for (const sourceRef of row.source_refs) {
      assert(
        !rowSources.has(sourceRef.source_id),
        `${label}: duplicate source ref ${sourceRef.source_id}`,
      );
      rowSources.add(sourceRef.source_id);
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
      if (sourceRef.evidence_quality === "ProjectPolicy") {
        assert(nonempty(sourceRef.note), `${label}: policy source note missing`);
        assert(
          nonempty(sourceRef.replacement_condition),
          `${label}: policy source replacement missing`,
        );
      }
      referencedSources.add(sourceRef.source_id);
      sourceReferenceCount += 1;
    }
    auditOwnershipFields(row, label);
    qualityCounts[row.evidence_quality] =
      (qualityCounts[row.evidence_quality] ?? 0) + 1;
    ownershipCounts[row.ownership] =
      (ownershipCounts[row.ownership] ?? 0) + 1;
    coverageStateCounts[row.coverage_state] =
      (coverageStateCounts[row.coverage_state] ?? 0) + 1;
  }
}
assert(commonRows.length === 26_985, "normalized row denominator differs");
assert(
  JSON.stringify(sortedObject(ownershipCounts)) === JSON.stringify({
    DivergentUniverse: 23_992,
    Excluded: 16,
    OtherMode: 57,
    Shared: 2_920,
  }),
  "normalized ownership denominator differs",
);
assert(sourceReferenceCount === 95_109, "provenance denominator differs");
for (const source of sources) {
  assert(referencedSources.has(source.source_id), `${source.id}: orphan source`);
}

const manifestRows = [];
const manifestByKey = new Map();
const manifestOwnership = {};
for (const [categoryId, category] of Object.entries(manifest.categories)) {
  assert(category.id === categoryId, `${categoryId}: category identity differs`);
  assert(category.records.length === category.count, `${categoryId}: count differs`);
  for (const row of category.records) {
    const key = `${categoryId}/${row.id}`;
    assert(!manifestByKey.has(key), `${key}: duplicate manifest obligation`);
    manifestByKey.set(key, row);
    assert(nonempty(row.source), `${key}: source locator missing`);
    assert(/^[0-9a-f]{64}$/u.test(row.evidence_sha256), `${key}: digest invalid`);
    assert(quality.has(row.evidence_quality), `${key}: quality invalid`);
    assert(
      ["DivergentUniverse", "Shared", "SharedCandidate"].includes(row.ownership),
      `${key}: ownership invalid`,
    );
    manifestOwnership[row.ownership] =
      (manifestOwnership[row.ownership] ?? 0) + 1;
    manifestRows.push(row);
  }
}
assert(manifestRows.length === 6_215, "manifest denominator differs");
assert(Object.keys(manifest.categories).length === 50, "category denominator differs");
assert(
  JSON.stringify(sortedObject(manifestOwnership)) === JSON.stringify({
    DivergentUniverse: 4_507,
    Shared: 1,
    SharedCandidate: 1_707,
  }),
  "manifest ownership denominator differs",
);

const coverageByKey = uniqueMap(
  coverage,
  ({ manifest_category: category, manifest_record_id: id }) =>
    `${category}/${id}`,
  "coverage obligation",
);
assert(coverageByKey.size === 6_215, "coverage denominator differs");
let coverageDataBindings = 0;
for (const [key, obligation] of manifestByKey) {
  const row = coverageByKey.get(key);
  assert(row, `${key}: coverage row missing`);
  assert(row.source_locator === obligation.source, `${key}: locator differs`);
  assert(
    row.source_evidence_sha256 === obligation.evidence_sha256,
    `${key}: source digest differs`,
  );
  assert(row.state === "DataReady", `${key}: disposition is not DataReady`);
  assert(row.blocking_gap_ids.length === 0, `${key}: blocking gap leaked`);
  assert(
    Array.isArray(row.normalized_record_ids) &&
      row.normalized_record_ids.length > 0 &&
      new Set(row.normalized_record_ids).size === row.normalized_record_ids.length,
    `${key}: normalized bindings are empty or duplicate`,
  );
  for (const id of row.normalized_record_ids) {
    assert(contentById.has(id), `${key}: normalized ID ${id} does not resolve`);
    coverageDataBindings += 1;
  }
}
assert(coverageDataBindings === 21_424, "coverage binding denominator differs");

const familyById = uniqueMap(
  fixtureFamilies,
  ({ source_id: id }) => id,
  "fixture family",
);
const fixtureById = uniqueMap(fixtures, ({ id }) => id, "fixture");
assert(
  familyById.size === 25 && fixtureById.size === 25,
  "fixture denominator differs",
);
let familyInputBindings = 0;
let fixtureInputBindings = 0;
let fixtureEvidenceBindings = 0;
for (const family of fixtureFamilies) {
  assert(family.runtime_executable === false, `${family.id}: runtime leaked`);
  for (const id of family.selected_source_record_ids) {
    assert(contentById.has(id), `${family.id}: input ${id} missing`);
    familyInputBindings += 1;
  }
}
for (const fixture of fixtures) {
  assert(familyById.has(fixture.family_id), `${fixture.id}: family missing`);
  assert(fixture.runtime_executable === false, `${fixture.id}: runtime leaked`);
  for (const id of fixture.source_record_ids) {
    assert(contentById.has(id), `${fixture.id}: input ${id} missing`);
    fixtureInputBindings += 1;
  }
  for (const id of fixture.evidence_refs) {
    assert(sourceById.has(id), `${fixture.id}: evidence ${id} missing`);
    fixtureEvidenceBindings += 1;
  }
  for (const fact of fixture.expected_facts) {
    assert(fact.runtime_claim === false, `${fixture.id}: runtime claim leaked`);
  }
}
assert(
  familyInputBindings === 68 &&
    fixtureInputBindings === 68 &&
    fixtureEvidenceBindings === 174,
  "fixture reference denominator differs",
);

let ruleBindings = 0;
assert(rules.length === 669, "mechanic-rule denominator differs");
for (const rule of rules) {
  const source = contentById.get(rule.source_file_id);
  assert(source?.file === "mechanic-source-files.json", `${rule.id}: owner missing`);
  assert(rule.runtime_lowered === false, `${rule.id}: runtime lowering leaked`);
  ruleBindings += 1;
  for (const id of rule.fixture_ids) {
    assert(fixtureById.has(id), `${rule.id}: fixture ${id} missing`);
    ruleBindings += 1;
  }
}
assert(ruleBindings === 1_338, "mechanic-rule binding denominator differs");

let gapBindings = 0;
for (const gap of gaps) {
  assert(
    gap.blocking === false &&
      gap.state === "PolicyBound" &&
      gap.owner === "G11-P4-B2",
    `${gap.id}: gap state/owner differs`,
  );
  assert(
    Array.isArray(gap.known_facts) &&
      gap.known_facts.length > 0 &&
      nonempty(gap.selected_policy) &&
      nonempty(gap.replacement_condition),
    `${gap.id}: gap policy contract incomplete`,
  );
  for (const id of gap.affected_data_ids) {
    assert(contentById.has(id), `${gap.id}: affected ID ${id} missing`);
    gapBindings += 1;
  }
}
assert(gaps.length === 25 && gapBindings === 68, "gap denominator differs");
assert(receipts.length === 0, "P4-B1 expects reconciliation to remain pending");

const profile = valuesByFile.get("profiles.json");
const modules = valuesByFile.get("modules.json");
const entries = valuesByFile.get("entries.json");
assert(
  profile.length === 1 &&
    profile[0].module_id === "divergent-universe.module.6002201" &&
    profile[0].sub_mode === "TournRogue" &&
    profile[0].tourn_mode === "Tourn3" &&
    profile[0].runtime_enabled === false,
  "enabled profile boundary differs",
);
assert(
  modules.length === 1 &&
    modules[0].source_id === "6002201" &&
    modules[0].main_tourn_id === 3 &&
    modules[0].sub_tourn_id === 1,
  "enabled module boundary differs",
);
assert(
  entries.length === 2 &&
    entries.every(({ module_id: id }) =>
      id === "divergent-universe.module.6002201"
    ),
  "entry/module closure differs",
);
for (const { row } of contentById.values()) {
  if (["OtherMode", "Excluded"].includes(row.ownership)) {
    assert(row.coverage_state === "Excluded", `${row.id}: excluded row promoted`);
  }
}

const typedReferences = auditTypedSoraReferences();
const report = {
  schema_revision: "starclock.divergent-universe-release-audit.v1",
  goal_id: "divergent-universe-reference-v1",
  audited_at: "2026-07-29",
  result: "pass",
  inputs: {
    content_manifest_sha256: sha256(fs.readFileSync(manifestPath)),
    normalized_schema_sha256: sha256(fs.readFileSync(schemaPath)),
    sora_schema_lock_sha256: sha256(fs.readFileSync(soraLockPath)),
  },
  exact_once: {
    manifest_categories: 50,
    manifest_obligations: manifestRows.length,
    coverage_rows: coverage.length,
    coverage_data_bindings: coverageDataBindings,
    data_ready_obligations: coverage.length,
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
    coverage_state: sortedObject(coverageStateCounts),
    evidence_quality: sortedObject(qualityCounts),
  },
  references: {
    source_ref_bindings: sourceReferenceCount,
    referenced_source_rows: referencedSources.size,
    typed_sora_fields: typedReferences.fields,
    typed_sora_bindings: typedReferences.bindings,
    rule_owner_and_fixture_bindings: ruleBindings,
    family_input_bindings: familyInputBindings,
    fixture_input_bindings: fixtureInputBindings,
    fixture_evidence_bindings: fixtureEvidenceBindings,
    research_gap_affected_bindings: gapBindings,
    unresolved: 0,
  },
  bilingual: {
    rows_checked: commonRows.length,
    missing_english_or_chinese_fields: 0,
    chinese_fields_without_han_text: 0,
  },
  ownership_and_boundary: {
    enabled_module: "6002201",
    enabled_main_tournament: 3,
    enabled_sub_tournament: 1,
    runtime_enabled: false,
    manifest_ownership: sortedObject(manifestOwnership),
    normalized_ownership: sortedObject(ownershipCounts),
    other_mode_rows: ownershipCounts.OtherMode,
    excluded_rows: ownershipCounts.Excluded,
    promoted_other_mode_or_excluded_rows: 0,
    runtime_lowered_rules: 0,
    runtime_executable_fixtures: 0,
    runtime_claims: 0,
  },
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (write) {
  fs.writeFileSync(output, encoded);
} else {
  assert(fs.existsSync(output), "release audit is missing; run with --write");
  assert(fs.readFileSync(output, "utf8") === encoded, "release audit drifted");
}
console.log(
  `Divergent Universe release audit passed (${manifestRows.length} ` +
    `exact-once obligations; ${commonRows.length} bilingual rows; ` +
    `${sourceReferenceCount} provenance and ${typedReferences.bindings} ` +
    "typed-reference bindings; zero leaks).",
);

function auditTypedSoraReferences() {
  const debugRoot = path.join(
    root,
    "config",
    "divergent-universe-generated",
    "debug-json",
  );
  const idsByTable = new Map();
  const debugByTable = new Map();
  for (const table of sora.tables) {
    const rows = json(path.join(debugRoot, `${table.name}.json`)).table.rows;
    debugByTable.set(table.name, rows);
    idsByTable.set(
      table.name,
      new Set(rows.map(({ values }) => values.id.Integer)),
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
      assert(targetIds, `${table.name}.${field.name}: target table missing`);
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

function auditOwnershipFields(value, label) {
  if (Array.isArray(value)) {
    for (const item of value) auditOwnershipFields(item, label);
    return;
  }
  if (!value || typeof value !== "object") return;
  for (const [key, item] of Object.entries(value)) {
    if (key === "ownership") {
      assert(ownership.has(item), `${label}: nested ownership ${item} invalid`);
    } else if (key === "mode_owner" || key === "profile_owner") {
      throw new Error(`${label}: foreign owner field leaked`);
    }
    auditOwnershipFields(item, label);
  }
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
  throw new Error(`unsupported Sora reference ${JSON.stringify(value)}`);
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
