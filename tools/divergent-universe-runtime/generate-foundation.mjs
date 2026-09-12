#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const outputPath = "content-manifests/divergent-universe-runtime-v1/foundation.json";

const expectedFamilyRows = {
  "adventure-outcomes": 32,
  areas: 28,
  "arithmetic-mapping-builds": 95,
  "arithmetic-mapping-eligibility": 84,
  "arithmetic-mapping-rules": 7,
  "astronomical-divisions": 9,
  "blessing-equation-contributions": 414,
  "blessing-groups": 118,
  "blessing-levels": 828,
  "blessing-paths": 8,
  "blessing-rewrite-rules": 416,
  blessings: 414,
  "boss-pools": 618,
  cognoculi: 9,
  "common-constants": 34,
  coverage: 6762,
  "curio-groups": 286,
  "curio-lifecycle-rules": 179,
  "curio-pool-membership": 235,
  "curio-states": 235,
  curios: 179,
  currencies: 2,
  "curse-chests": 29,
  "cyclical-challenges": 13,
  difficulties: 22,
  "encounter-groups": 43,
  "encounter-source-obligations": 877,
  "encounter-waves": 176,
  "enemy-slots": 385,
  entries: 2,
  "equation-categories": 4,
  "equation-effects": 25,
  "equation-expansion-states": 160,
  "equation-offers": 136,
  "equation-progress": 80,
  "equation-recipes": 80,
  "equation-replacement-rules": 4,
  equations: 80,
  "finish-conditions": 13,
  "gamble-groups": 126,
  "gamble-units": 89,
  "grand-miracle-eligibility": 74,
  "grand-miracle-states": 34,
  "grand-miracles": 17,
  "layer-rooms": 0,
  layers: 11,
  manifest: 1,
  "mechanic-rules": 669,
  "mechanic-source-files": 669,
  "mode-service-npcs": 23,
  modules: 1,
  "occurrence-choices": 0,
  "occurrence-variants": 97,
  occurrences: 118,
  "pack-index": 1,
  "permanent-talents": 38,
  "persona-source-obligations": 547,
  "pool-membership": 3044,
  profiles: 1,
  "progression-effects": 38,
  protocols: 8,
  "reconciliation-receipts": 102,
  "research-gaps": 25,
  "review-fixtures": 25,
  "room-marks": 24,
  rooms: 848,
  "semantic-fixture-families": 25,
  "service-offer-rules": 161,
  "service-rules": 6,
  sources: 8171,
  "stage-flow": 111,
  "star-pioneer-practice": 2,
  "titan-boons": 84,
  "titan-choices": 36,
  "titan-contributions": 120,
  "titan-talents": 36,
  "titan-types": 12,
  unlocks: 97,
  "weekly-modifiers": 103,
  "workbench-functions": 6,
  workbenches: 11,
};

const expectedSourceFamilyCounts = sortObject({
  divergent_adventure_graph_candidate: 13,
  divergent_adventure_modifier_evidence: 1,
  divergent_maze_graph_candidate: 9,
  divergent_mechanic_evidence: 6,
  divergent_npc_graph_candidate: 159,
  divergent_occurrence_graph_candidate: 478,
  divergent_presentation_account_locator: 18,
  divergent_persona_structured_candidate: 11,
  divergent_service_graph_candidate: 3,
  divergent_structured_candidate: 44,
  divergent_test_exclusion_evidence: 3,
  encounter_stage_evidence: 1,
  gold_and_gears_mechanic_exclusion_evidence: 2,
  gold_and_gears_structured_exclusion_evidence: 21,
  localized_text_evidence: 2,
  other_mode_exclusion_evidence: 5,
  presentation_account_exclusion_evidence: 17,
  public_index_cross_check: 9,
  shared_level_graph_candidate: 304,
  shared_mechanic_evidence_candidate: 108,
  shared_occurrence_graph_candidate: 1332,
  shared_structured_candidate: 52,
  swarm_disaster_mechanic_exclusion_evidence: 6,
  swarm_disaster_structured_exclusion_evidence: 32,
  unknowable_domain_mechanic_exclusion_evidence: 16,
  unknowable_domain_structured_exclusion_evidence: 32,
});

const expectedManifestCategoryCounts = sortObject({
  profiles: 1,
  entry_points: 2,
  enabled_modules: 1,
  finish_conditions: 13,
  area_groups: 1,
  areas: 28,
  difficulties: 22,
  layers: 11,
  layer_rooms: 0,
  room_reuse_candidates: 848,
  room_types: 11,
  astronomical_divisions: 9,
  astronomical_division_effects: 8,
  arithmetic_mapping_avatars: 79,
  arithmetic_mapping_build_refs: 84,
  arithmetic_mapping_roles: 95,
  equations: 80,
  equation_displays: 80,
  equation_randomizers: 136,
  equation_keywords: 25,
  equation_keyword_params: 9,
  blessing_paths: 8,
  blessings: 414,
  blessing_levels: 828,
  blessing_groups: 118,
  curio_states: 235,
  curios: 179,
  curio_groups: 286,
  grand_miracles: 17,
  grand_miracle_eligibility: 57,
  titan_types: 12,
  titan_bless_levels: 84,
  titan_talent_levels: 36,
  workbenches: 11,
  workbench_functions: 6,
  gamble_groups: 126,
  gamble_units: 89,
  curse_chests: 29,
  permanent_talents: 38,
  unlocks: 97,
  common_constants: 34,
  weekly_modifiers: 103,
  room_marks: 24,
  occurrences: 118,
  occurrence_variants: 97,
  mode_service_npcs: 23,
  adventure_outcomes: 32,
  encounter_source_obligations: 877,
  mechanic_source_files: 669,
  semantic_fixture_families: 25,
  persona_source_obligations: 547,
});

const expectedDenominators = {
  source_files: 2684,
  source_files_by_repository: {
    starrailres: 9,
    turnbasedgamedata: 2675,
  },
  source_file_families: expectedSourceFamilyCounts,
  source_content_obligations: 6762,
  manifest_categories: 51,
  manifest_category_counts: expectedManifestCategoryCounts,
  source_ownership: {
    DivergentUniverse: 4585,
    Shared: 1,
    SharedCandidate: 2176,
  },
  normalized_families: 81,
  sora_tables: 81,
  authored_exported_rows: 28732,
  verified_empty_families: 2,
  mechanic_programs: 669,
  mechanic_program_scopes: {
    Activity: 662,
    Battle: 3,
    CrossBattle: 1,
    EvidenceLayout: 3,
  },
  runtime_lowered_mechanic_programs: 0,
  semantic_fixture_families: 25,
  runtime_executable_semantic_families: 0,
  research_gaps: 25,
  blocking_research_gaps: 0,
  project_policy_sources: 54,
  project_policy_sources_with_replacement_conditions: 54,
  coverage_rows: 6762,
  data_ready_coverage_rows: 6215,
  profiles_modules_entries_finish_conditions: 17,
  areas: 28,
  difficulties: 22,
  layers: 11,
  arithmetic_mapping_obligations: 258,
  equations: 80,
  divergent_blessings: 414,
  current_curio_mode_copies: 235,
  grand_miracles: 17,
  titan_types: 12,
  titan_boons: 84,
  titan_talent_levels: 36,
};

const inputPaths = [
  "content-manifests/divergent-universe-v1/source-inventory.json",
  "content-manifests/divergent-universe-v1/content-manifest.json",
  "content-manifests/divergent-universe-v1/normalized-schema.json",
  "content-manifests/divergent-universe-v1/authoring-contract.json",
  "content-manifests/divergent-universe-v1/fixture-contract.json",
  "content-reference/divergent-universe-v1/pack-index.json",
  "content-reference/divergent-universe-v1/mechanic-rules.json",
  "content-reference/divergent-universe-v1/semantic-fixture-families.json",
  "content-reference/divergent-universe-v1/review-fixtures.json",
  "content-reference/divergent-universe-v1/research-gaps.json",
  "content-reference/divergent-universe-v1/coverage.json",
  "evidence/divergent-universe-reference-v1/semantic-fixture-results.json",
  "evidence/divergent-universe-reference-v1/sora-current-state.json",
  "config/divergent-universe-generated/schema.lock",
  "config/divergent-universe-generated/config.sora",
  "policy/sora-toolchain.json",
];

export function buildFoundation() {
  const sourceInventory = json(inputPaths[0]);
  const contentManifest = json(inputPaths[1]);
  const normalizedSchema = json(inputPaths[2]);
  const authoringContract = json(inputPaths[3]);
  const mechanicRules = json(inputPaths[6]);
  const fixtureFamilies = json(inputPaths[7]);
  const reviewFixtures = json(inputPaths[8]);
  const researchGaps = json(inputPaths[9]);
  const coverage = json(inputPaths[10]);
  const fixtureResults = json(inputPaths[11]);
  const soraState = json(inputPaths[12]);
  const schemaLock = json(inputPaths[13]);
  const soraPolicy = json(inputPaths[15]);

  const normalizedFiles = authoringContract.workbooks
    .flatMap(({ normalized_files: files }) => files);
  assert(new Set(normalizedFiles).size === normalizedFiles.length,
    "authoring contract contains duplicate normalized files");
  assert(normalizedFiles.length === normalizedSchema.files.length,
    "authoring and normalized family counts differ");
  assert(equal([...normalizedFiles].sort(),
    normalizedSchema.files.map(({ file }) => file).sort()),
  "authoring and normalized family assignments differ");

  const familyRows = Object.fromEntries(normalizedFiles.map((file) => {
    const rows = json(`content-reference/divergent-universe-v1/${file}`);
    assert(Array.isArray(rows), `${file} is not a normalized row array`);
    return [file.slice(0, -".json".length), rows.length];
  }).sort(([left], [right]) => left.localeCompare(right)));
  assert(equal(familyRows, expectedFamilyRows),
    `Goal 22 family denominator drift:\nexpected ${pretty(expectedFamilyRows)}`
      + `actual ${pretty(familyRows)}`);

  const projectPolicySources = collectProjectPolicySources(normalizedFiles);
  const mechanicScopes = countBy(mechanicRules, ({ scope }) => scope);
  const denominators = {
    source_files: sourceInventory.counts.total,
    source_files_by_repository: sortObject(sourceInventory.counts.by_repository),
    source_file_families: sortObject(sourceInventory.counts.by_family),
    source_content_obligations: contentManifest.counts.records,
    manifest_categories: contentManifest.counts.categories,
    manifest_category_counts: sortObject(Object.fromEntries(
      Object.entries(contentManifest.categories).map(([category, { count }]) =>
        [category, count]))),
    source_ownership: contentManifest.counts.ownership,
    normalized_families: normalizedFiles.length,
    sora_tables: schemaLock.schema.tables.length,
    authored_exported_rows: sum(Object.values(familyRows)),
    verified_empty_families: Object.values(familyRows).filter((rows) => rows === 0).length,
    mechanic_programs: mechanicRules.length,
    mechanic_program_scopes: mechanicScopes,
    runtime_lowered_mechanic_programs:
      mechanicRules.filter(({ runtime_lowered: lowered }) => lowered).length,
    semantic_fixture_families: fixtureFamilies.length,
    runtime_executable_semantic_families:
      fixtureFamilies.filter(({ runtime_executable: executable }) => executable).length,
    research_gaps: researchGaps.length,
    blocking_research_gaps: researchGaps.filter(({ blocking }) => blocking).length,
    project_policy_sources: projectPolicySources.size,
    project_policy_sources_with_replacement_conditions:
      [...projectPolicySources.values()].filter(Boolean).length,
    coverage_rows: coverage.length,
    data_ready_coverage_rows:
      coverage.filter(({ state }) => state === "DataReady").length,
    profiles_modules_entries_finish_conditions:
      rowTotal(familyRows, ["profiles", "modules", "entries", "finish-conditions"]),
    areas: familyRows.areas,
    difficulties: familyRows.difficulties,
    layers: familyRows.layers,
    arithmetic_mapping_obligations: rowTotal(contentManifest.categories, [
      "arithmetic_mapping_avatars",
      "arithmetic_mapping_build_refs",
      "arithmetic_mapping_roles",
    ], ({ count }) => count),
    equations: familyRows.equations,
    divergent_blessings: familyRows.blessings,
    current_curio_mode_copies: familyRows["curio-states"],
    grand_miracles: familyRows["grand-miracles"],
    titan_types: familyRows["titan-types"],
    titan_boons: familyRows["titan-boons"],
    titan_talent_levels: familyRows["titan-talents"],
  };
  assert(equal(denominators, expectedDenominators),
    `Goal 22 denominator drift:\nexpected ${pretty(expectedDenominators)}`
      + `actual ${pretty(denominators)}`);
  assert(reviewFixtures.length === fixtureFamilies.length,
    "semantic and review fixture family counts differ");
  assert(researchGaps.every(({ state, blocking, replacement_condition: replacement }) =>
    state === "PolicyBound" && blocking === false && replacement.length > 0),
  "research gaps must remain nonblocking PolicyBound rows with replacement conditions");
  assert(fixtureResults.summary.runtime_executions === 0,
    "reference fixtures unexpectedly claim runtime executions");
  assert(soraPolicy.version === "0.6.1" && soraState.toolchain.version === "0.6.1",
    "Goal 22 requires Sora 0.6.1");
  assert(soraState.generated.tables === denominators.sora_tables
    && soraState.generated.rows === denominators.authored_exported_rows,
  "current Sora state differs from frozen denominators");
  assert(soraState.generated.schema_lock_sha256 === sha256(inputPaths[13])
    && soraState.generated.bundle.sha256 === sha256(inputPaths[14])
    && soraState.generated.bundle.bytes === fs.statSync(path.join(root, inputPaths[14])).size,
  "current Sora identity differs from actual schema or bundle bytes");

  return {
    generated_by: "tools/divergent-universe-runtime/generate-foundation.mjs",
    game_version: "4.4",
    source_snapshot: {
      access_date: sourceInventory.snapshot.access_date,
      repositories: sourceInventory.snapshot.repositories,
    },
    toolchain: {
      package: soraPolicy.package,
      version: soraPolicy.version,
      crate_sha256: soraPolicy.crate_sha256,
      install_root: soraPolicy.install_root,
    },
    promoted_reference: {
      status: "CandidateReferenceOnly",
      runtime_execution_credit: false,
      full_playability_proven: false,
      project: soraState.project,
      authoring_semantic_sha256: soraState.authoring.semantic_sha256,
      schema_lock_sha256: soraState.generated.schema_lock_sha256,
      bundle: soraState.generated.bundle,
      rust_reader_files: soraState.generated.rust_reader_files,
      rust_reader_sha256: soraState.generated.rust_reader_sha256,
    },
    input_digests: Object.fromEntries(inputPaths.map((input) => [input, sha256(input)])),
    denominators,
    family_rows: familyRows,
    reference_fixture_summary: fixtureResults.summary,
    boundary: "Current reference-input accounting only; executable behavior requires independent production acceptance.",
  };
}

function collectProjectPolicySources(normalizedFiles) {
  const sources = new Map();
  for (const file of normalizedFiles) {
    for (const row of json(`content-reference/divergent-universe-v1/${file}`)) {
      for (const source of row.source_refs ?? []) {
        if (source.evidence_quality !== "ProjectPolicy") continue;
        const replacement = source.replacement_condition ?? "";
        const previous = sources.get(source.source_id);
        assert(previous === undefined || previous === replacement,
          `ProjectPolicy replacement drift for ${source.source_id}`);
        sources.set(source.source_id, replacement);
      }
    }
  }
  return sources;
}

function rowTotal(counts, names, valueOf = (value) => value) {
  return names.reduce((total, name) => total + valueOf(counts[name]), 0);
}

function sum(values) {
  return values.reduce((total, value) => total + value, 0);
}

function countBy(values, keyOf) {
  const counts = {};
  for (const value of values) {
    const key = keyOf(value);
    assert(typeof key === "string" && key.length > 0,
      "cannot count a row without a stable category");
    counts[key] = (counts[key] ?? 0) + 1;
  }
  return sortObject(counts);
}

function sortObject(value) {
  return Object.fromEntries(Object.entries(value).sort(([left], [right]) =>
    left.localeCompare(right)));
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function sha256(relativePath) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relativePath)))
    .digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function equal(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const expected = pretty(buildFoundation());
  const output = path.join(root, outputPath);
  if (process.argv.includes("--check")) {
    assert(fs.readFileSync(output, "utf8") === expected,
      `${outputPath} is stale; run node tools/divergent-universe-runtime/generate-foundation.mjs`);
    console.log("Divergent Universe runtime foundation is current.");
  } else {
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, expected);
    console.log(`Generated ${outputPath}.`);
  }
}
