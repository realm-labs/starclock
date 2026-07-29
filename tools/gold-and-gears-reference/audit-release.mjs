import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const write = process.argv.includes("--write");
const packRoot = path.join(root, "content-reference", "gold-and-gears-v1");
const manifestPath = path.join(
  root,
  "content-manifests",
  "gold-and-gears-v1",
  "content-manifest.json",
);
const schemaPath = path.join(
  root,
  "content-manifests",
  "gold-and-gears-v1",
  "normalized-schema.json",
);
const soraLockPath = path.join(
  root,
  "config",
  "gold-and-gears-generated",
  "schema.lock",
);
const output = path.join(
  root,
  "evidence",
  "gold-and-gears-reference-v1",
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
const fixtures = valuesByFile.get("review-fixtures.json");
const rules = valuesByFile.get("mechanic-rules.json");
const quality = new Set(schema.common_envelope.evidence_quality.enum);
const ownership = new Set(schema.common_envelope.ownership.enum);
const commonFields = schema.common_envelope.required_fields;
const excludedSourceKeys = new Set(
  manifest.exclusions.story_account_rows.map(({ source }) => source),
);
const forbiddenModePrefixes = manifest.exclusions.mode_prefixes;

assert(schema.files.length === 51, "normalized file-family denominator differs");
assert(Array.isArray(sources) && sources.length === 9082, "source denominator differs");
const sourceById = uniqueMap(sources, ({ id }) => id, "source");
for (const source of sources) {
  assert(source.id === source.source_id, `${source.id}: source identity differs`);
  assert(source.game_version === "4.4", `${source.id}: game version differs`);
  for (const field of [
    "repository_or_url",
    "revision_or_access_date",
    "relative_path_or_page",
    "row_locator",
    "access_date",
  ]) {
    assert(nonempty(source[field]), `${source.id}: ${field} is empty`);
  }
  assert(/^[0-9a-f]{64}$/u.test(source.evidence_sha256), `${source.id}: digest invalid`);
  assert(quality.has(source.evidence_quality), `${source.id}: quality invalid`);
  if (source.evidence_quality === "ProjectPolicy") {
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
  if (["sources.json", "manifest.json", "pack-index.json"].includes(file)) continue;
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
      for (const [referenceField, sourceField] of [
        ["repository", "repository_or_url"],
        ["revision", "revision_or_access_date"],
        ["path", "relative_path_or_page"],
        ["locator", "row_locator"],
        ["sha256", "evidence_sha256"],
        ["access_date", "access_date"],
        ["evidence_quality", "evidence_quality"],
      ]) {
        assert(
          sourceRef[referenceField] === source[sourceField],
          `${label}: source ${sourceRef.source_id} ${referenceField} differs`,
        );
      }
      const locator = `${sourceRef.path}#${sourceRef.locator}`;
      assert(!excludedSourceKeys.has(locator), `${label}: excluded story/account row leaked`);
      assert(
        !forbiddenModePrefixes.some((prefix) => sourceRef.path.includes(prefix)),
        `${label}: other-mode source path leaked`,
      );
      if (sourceRef.evidence_quality === "ProjectPolicy") {
        assert(nonempty(sourceRef.note), `${label}: policy source note missing`);
        assert(
          nonempty(sourceRef.replacement_condition),
          `${label}: policy replacement condition missing`,
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
assert(commonRows.length === 15031, "common normalized row denominator differs");
assert(
  ownershipCounts.GoldAndGears === 13637 && ownershipCounts.Shared === 1394,
  "normalized ownership denominator differs",
);
assert(sourceReferenceCount === 31176, "provenance binding denominator differs");
for (const source of sources) {
  assert(referencedSources.has(source.id), `${source.id}: orphan provenance row`);
}

const manifestRows = [];
const manifestKeys = new Set();
const manifestOwnership = {};
for (const [categoryId, category] of Object.entries(manifest.categories)) {
  assert(category.id === categoryId, `${categoryId}: manifest category identity differs`);
  assert(category.records.length === category.count, `${categoryId}: category count differs`);
  for (const row of category.records) {
    const key = `${categoryId}/${row.id}`;
    assert(!manifestKeys.has(key), `${key}: duplicate manifest obligation`);
    manifestKeys.add(key);
    assert(nonempty(row.source), `${key}: source locator missing`);
    assert(/^[0-9a-f]{64}$/u.test(row.evidence_sha256), `${key}: digest invalid`);
    assert(quality.has(row.evidence_quality), `${key}: evidence quality invalid`);
    assert(ownership.has(row.ownership), `${key}: ownership invalid`);
    assert(
      ["Direct", "Referenced", "InheritedSharedPool"].includes(row.reachability),
      `${key}: reachability invalid`,
    );
    manifestOwnership[row.ownership] = (manifestOwnership[row.ownership] ?? 0) + 1;
    manifestRows.push(row);
  }
}
assert(manifestRows.length === 7913, "manifest obligation denominator differs");
assert(manifest.counts.records === manifestRows.length, "manifest aggregate count differs");
assert(
  manifestOwnership.GoldAndGears === 7199 && manifestOwnership.Shared === 714,
  "frozen manifest ownership denominator differs",
);
assert(
  JSON.stringify(manifest.counts.ownership) ===
    JSON.stringify(sortedObject(manifestOwnership)),
  "manifest ownership aggregate differs",
);

const coverageByCategory = uniqueMap(
  coverage,
  ({ category_id: categoryId }) => categoryId,
  "coverage category",
);
assert(coverageByCategory.size === 42, "coverage category denominator differs");
for (const [categoryId, category] of Object.entries(manifest.categories)) {
  const row = coverageByCategory.get(categoryId);
  assert(row, `${categoryId}: coverage row missing`);
  assert(
    row.required === category.count &&
      row.accounted === category.count &&
      row.data_ready === category.count &&
      row.coverage_percent === "100" &&
      row.blocking_gap_ids.length === 0,
    `${categoryId}: exact-once coverage differs`,
  );
}

const fixtureById = uniqueMap(fixtures, ({ id }) => id, "fixture");
const fixtureFamilies = new Set(fixtures.map(({ family_id: familyId }) => familyId));
assert(fixtures.length === 18 && fixtureFamilies.size === 18, "fixture denominator differs");
let fixtureInputReferenceCount = 0;
let fixtureEvidenceReferenceCount = 0;
for (const fixture of fixtures) {
  for (const id of fixture.source_record_ids) {
    assert(contentById.has(id), `${fixture.id}: fixture input ${id} does not resolve`);
    fixtureInputReferenceCount += 1;
  }
  for (const id of fixture.evidence_refs) {
    assert(sourceById.has(id), `${fixture.id}: fixture evidence ${id} does not resolve`);
    fixtureEvidenceReferenceCount += 1;
  }
}

let ruleReferenceCount = 0;
assert(rules.length === 1224, "mechanic-rule denominator differs");
for (const rule of rules) {
  const owner = contentById.get(rule.owner_id);
  assert(owner, `${rule.id}: owner ${rule.owner_id} does not resolve`);
  assert(owner.file === rule.source_file, `${rule.id}: owner file differs`);
  assert(owner.row.ownership === rule.ownership, `${rule.id}: owner ownership differs`);
  assert(rule.execution_disposition === "ReferenceOnly", `${rule.id}: execution leaked`);
  assert(rule.runtime_handler_id === "", `${rule.id}: runtime handler leaked`);
  for (const id of rule.fixture_ids) {
    assert(fixtureById.has(id), `${rule.id}: fixture ${id} does not resolve`);
    ruleReferenceCount += 1;
  }
  ruleReferenceCount += 1;
}

let gapAffectedReferenceCount = 0;
for (const gap of gaps) {
  assert(gap.blocking === false && gap.gap_state === "PolicyBound", `${gap.id}: gap blocks`);
  assert(sourceById.has(gap.policy_source_id), `${gap.id}: policy source does not resolve`);
  assert(nonempty(gap.note), `${gap.id}: gap note missing`);
  assert(nonempty(gap.replacement_condition), `${gap.id}: replacement condition missing`);
  for (const affected of gap.affected_records) {
    const target = valuesByFile.get(affected.file);
    assert(Array.isArray(target), `${gap.id}: affected file ${affected.file} missing`);
    assert(
      target.some(({ id }) => id === affected.id),
      `${gap.id}: affected record ${affected.file}/${affected.id} does not resolve`,
    );
    gapAffectedReferenceCount += 1;
  }
}
assert(gaps.length === 16 && gapAffectedReferenceCount === 5025, "gap denominator differs");

const typedReferences = auditTypedSoraReferences();
assert(
  typedReferences.fields === 31 && typedReferences.bindings === 27425,
  "typed Sora reference denominator differs",
);
assert(
  manifest.exclusions.story_account_rows.length === 58,
  "story/account exclusion denominator differs",
);
const report = {
  schema_revision: "starclock.gold-and-gears-release-audit.v1",
  goal_id: "gold-and-gears-reference-v1",
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
    data_ready_obligations: coverage.reduce((sum, row) => sum + row.data_ready, 0),
    coverage_percent: "100",
    duplicate_obligations: 0,
    blocking_gaps: 0,
  },
  normalized: {
    files: schema.files.length,
    common_rows: commonRows.length,
    source_rows: sources.length,
    mechanic_rules: rules.length,
    semantic_fixtures: fixtures.length,
    research_gaps: gaps.length,
    ownership: sortedObject(ownershipCounts),
    evidence_quality: sortedObject(qualityCounts),
  },
  references: {
    source_ref_bindings: sourceReferenceCount,
    referenced_source_rows: referencedSources.size,
    typed_sora_fields: typedReferences.fields,
    typed_sora_bindings: typedReferences.bindings,
    rule_owner_and_fixture_bindings: ruleReferenceCount,
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
  boundary: {
    allowed_ownership: [...ownership],
    inherited_standard_source_rows:
      commonRows.filter(({ source_mode_owner: value }) => value === "Standard").length,
    standard_profile_rows: 0,
    swarm_disaster_rows: 0,
    unknowable_domain_rows: 0,
    divergent_universe_rows: 0,
    story_or_account_rows: 0,
    runtime_handlers: 0,
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
  `Gold and Gears release audit passed (${manifestRows.length} exact-once obligations; ` +
    `${commonRows.length} bilingual rows; ${sourceReferenceCount} provenance bindings; ` +
    `${typedReferences.bindings} typed references; zero leaks).`,
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
    } else if (key === "source_mode_owner") {
      assert(entry === "Standard", `${label}: source mode owner ${entry} leaked`);
      assert(value.ownership === "Shared", `${label}: Standard source is not Shared`);
    } else if (key === "mode_owner" || key === "profile_owner") {
      throw new Error(`${label}: ${key} leaked`);
    }
    auditOwnershipFields(entry, label);
  }
}

function auditTypedSoraReferences() {
  const debugRoot = path.join(root, "config", "gold-and-gears-generated", "debug-json");
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
  if (value === undefined || value === null || value.Null !== undefined) return [];
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
  return Object.fromEntries(Object.entries(value).sort(([left], [right]) =>
    left.localeCompare(right)));
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
