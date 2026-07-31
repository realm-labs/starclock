#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const packRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
);
const manifestRoot = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
);
function option(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}
function digest(value) {
  return createHash("sha256").update(canonical(value)).digest("hex");
}
function run(script, args = []) {
  execFileSync(process.execPath, [
    path.join("tools", "galactic-baseballer-reference", script),
    ...args,
  ], { cwd: root, stdio: "inherit" });
}
run("normalize-departure-progression.mjs", [
  "--check",
  "--source-cache",
  sourceCache,
]);
run("assemble-reference-pack.mjs", ["--check"]);

const schema = JSON.parse(await readFile(
  path.join(manifestRoot, "normalized-schema.json"),
  "utf8",
));
const sourceManifest = JSON.parse(await readFile(
  path.join(manifestRoot, "content-manifest.json"),
  "utf8",
));
const files = schema.files.map(({ file }) => file);
assert(files.length === 40, "normalized file denominator drift");
assert(new Set(files).size === files.length, "duplicate normalized file");
const data = new Map();
for (const file of files) {
  const rows = JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
  assert(Array.isArray(rows), `normalized file is not an array: ${file}`);
  assert(
    new Set(rows.map(({ id }) => id)).size === rows.length,
    `duplicate row ID in ${file}`,
  );
  data.set(file, rows);
}

const profiles = data.get("profiles.json");
assert(
  profiles.length === 2
    && profiles.map(({ id }) => id).join(",")
      === "galactic-baseballer.demon-king.v3_3,"
        + "galactic-baseballer.departure.v2_2",
  "profile closure drift",
);
assert(
  profiles.every(({ runtime_enabled: enabled }) => enabled === false),
  "a reference profile became runtime enabled",
);

const manifestRows = Object.entries(sourceManifest.categories)
  .flatMap(([categoryId, { records }]) =>
    records.map((record) => ({ categoryId, record })));
assert(manifestRows.length === 2232, "source obligation denominator drift");
const manifestById = new Map(manifestRows.map(({ categoryId, record }) => [
  record.id,
  { categoryId, record },
]));
assert(manifestById.size === 2232, "duplicate source obligation ID");

const coverage = data.get("coverage.json");
const reconciliation = data.get("reconciliation.json");
assert(coverage.length === 2232, "coverage row count drift");
assert(reconciliation.length === 2232, "reconciliation row count drift");
assert(
  coverage.filter(({ coverage_state: state }) => state === "DataReady").length
    === 2207,
  "DataReady denominator drift",
);
assert(
  coverage.filter(({ coverage_state: state }) => state === "EvidenceOnly")
    .length === 25,
  "EvidenceOnly denominator drift",
);
assert(
  coverage.every(({ closure_kind: kind }) =>
    ["NormalizedRecordReference", "EvidenceOnlyExcluded"].includes(kind)),
  "an obligation lacks an explicit normalized/excluded owner",
);
const contentIds = new Set(
  [...data.entries()]
    .filter(([file]) => ![
      "coverage.json",
      "reconciliation.json",
      "sources.json",
      "manifest.json",
      "pack-index.json",
    ].includes(file))
    .flatMap(([, rows]) => rows.map(({ id }) => id)),
);
for (const row of coverage) {
  const source = manifestById.get(row.record_id);
  assert(source !== undefined, `unknown coverage record: ${row.record_id}`);
  assert(
    source.categoryId === row.category_id
      && source.record.evidence_sha256 === row.evidence_sha256
      && source.record.source_path === row.source_path
      && source.record.row_locator === row.row_locator,
    `coverage evidence drift: ${row.record_id}`,
  );
  if (row.coverage_state === "DataReady") {
    assert(
      row.normalized_record_ids.length >= 1
        && row.normalized_record_ids.every((id) => contentIds.has(id)),
      `DataReady row lacks a real normalized owner: ${row.record_id}`,
    );
  } else {
    assert(
      source.record.runtime_disposition === "EvidenceOnly"
        && row.normalized_record_ids.length === 0,
      `EvidenceOnly row leaked into normalized mechanics: ${row.record_id}`,
    );
  }
}
const reconciledSourceIds = reconciliation.map(
  ({ source_record_id: id }) => id,
);
assert(
  new Set(reconciledSourceIds).size === 2232
    && reconciledSourceIds.every((id) => manifestById.has(id)),
  "reconciliation is not exact-once",
);
assert(
  reconciliation.every((row) =>
    row.exact_once === true && row.inferred_from_name_or_id_range === false),
  "reconciliation inference/exact-once policy drift",
);

const sources = data.get("sources.json");
const sourceIds = new Set(sources.map(({ id }) => id));
assert(
  sources.every((row) =>
    row.repository_or_url.length > 0
    && row.revision_or_access_date.length > 0
    && row.game_version.length > 0
    && row.path_or_pages.length >= 1
    && row.locators.length >= 1
    && row.sha256_values.every((value) => /^[0-9a-f]{64}$/u.test(value))
    && row.evidence_qualities.length >= 1),
  "source receipt completeness drift",
);
for (const [, rows] of data) {
  for (const row of rows) {
    for (const source of row.source_refs ?? []) {
      assert(
        sourceIds.has(source.source_id),
        `unregistered source reference: ${source.source_id}`,
      );
    }
  }
}

const approximations = data.get("approximations.json");
const gaps = data.get("research-gaps.json");
assert(approximations.length === 12, "approximation denominator drift");
assert(gaps.length === 12, "research-gap denominator drift");
assert(
  approximations.every((row) =>
    ["ProjectPolicy", "ApproximateFromReleasedText"]
      .includes(row.evidence_quality)
    && row.unavailable_fact.length > 0
    && row.selected_policy.length > 0
    && row.rejected_alternatives.length >= 2
    && row.rationale.length > 0
    && row.affected_fixture_ids.length >= 1
    && row.confidence.length > 0
    && row.replacement_condition.length > 0),
  "approximation replacement contract drift",
);
assert(
  gaps.every(({ state, terminal_blocker: blocker }) =>
    state === "ReplaceableNonBlocking" && blocker === false),
  "a research boundary became blocking or untracked",
);

const rules = data.get("mechanic-rules.json");
const fixtures = data.get("review-fixtures.json");
const familyIds = new Set(
  sourceManifest.categories.semantic_fixture_families.records
    .map(({ id }) => id),
);
assert(familyIds.size === 20, "semantic family denominator drift");
assert(rules.length === 26, "mechanic rule count drift");
assert(fixtures.length === 35, "review fixture count drift");
assert(
  [...familyIds].every((family) =>
    rules.some(({ family_id: id }) => id === family)
    && fixtures.some(({ family_id: id }) => id === family)),
  "a semantic family lacks a rule or fixture",
);
assert(
  rules.every((row) =>
    row.runtime_executable === false
    && row.trigger_point.length > 0
    && row.state_owner.length > 0
    && row.ordered_operations.length >= 1
    && row.fixture_ids.every((id) =>
      fixtures.some(({ id: fixtureId }) => fixtureId === id))),
  "mechanic rule structure/linkage drift",
);
assert(
  fixtures.every((row) =>
    row.runtime_executable === false
    && row.trigger_point.length > 0
    && row.state_owner.length > 0
    && row.ordered_operations.length >= 1
    && Object.keys(row.input).length >= 1
    && Object.keys(row.expected_facts).length >= 1),
  "semantic fixture structure drift",
);

assert(
  data.get("shop-upgrades.json").length === 114,
  "combined store level count drift",
);
assert(
  data.get("currencies.json").length === 3,
  "combined currency count drift",
);
const departureGold = data.get("currencies.json").find(({ id }) =>
  id === "galactic-baseballer.departure.currency.raccoon-gold");
assert(
  departureGold !== undefined
    && departureGold.source_item_id === "281019"
    && departureGold.maximum_balance === null
    && departureGold.maximum_balance_disposition
      === "UnspecifiedInFrozenStructuredFamily"
    && departureGold.gold_max_level_vector.join(",") === "100,250,500"
    && Object.values(departureGold.enemy_income).join(",")
      === "5,5,20,200,0"
    && departureGold.chest_income_vector.join(",") === "400,1500,2500"
    && departureGold.chest_basic_gold.join(",") === "50,250,400"
    && departureGold.chest_alternate_gold.join(",") === "50,250,400"
    && departureGold.chest_probability_vector.join(",") === "0.4,0.3,0.3"
    && departureGold.chest_probability_step.join(",") === "-0.1,0,0.1",
  "Departure Raccoon Gold vector drift",
);
assert(
  data.get("unlocks.json").length === 50,
  "combined unlock/tutorial count drift",
);
assert(
  data.get("progression.json").filter(({ progression_kind: kind }) =>
    kind === "TeamBonusDefinition").length === 8
    && data.get("progression.json").filter(({ kind }) =>
      kind === "TeamBonus").length === 7,
  "team-bonus definition count drift",
);

const manifestRowsOut = data.get("manifest.json");
assert(
  manifestRowsOut.length === 1
    && manifestRowsOut[0].source_obligation_count === 2232
    && manifestRowsOut[0].coverage_state_counts.DataReady === 2207
    && manifestRowsOut[0].coverage_state_counts.EvidenceOnly === 25
    && manifestRowsOut[0].mechanic_family_count === 20
    && manifestRowsOut[0].runtime_enabled === false
    && manifestRowsOut[0].delivery_lane === "Candidate",
  "normalized pack manifest drift",
);
const index = data.get("pack-index.json")[0];
assert(
  index.normalized_file_count === 40
    && index.indexed_file_count === 39
    && index.files.length === 39,
  "pack index file count drift",
);
for (const entry of index.files) {
  assert(
    data.has(entry.file)
      && entry.row_count === data.get(entry.file).length
      && entry.canonical_sha256 === digest(data.get(entry.file)),
    `pack index digest/count drift: ${entry.file}`,
  );
}

console.log(
  "Galactic Baseballer reference pack verified: 40 files, 2232/2232 "
  + "exact-once obligations, 2207 DataReady, 25 EvidenceOnly, 12 "
  + "replaceable boundaries, 20 semantic families, 26 rules and 35 fixtures",
);
