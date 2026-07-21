import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const PACK_SHA = "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a";
const MANIFEST_SHA = "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19";
const PROVENANCE_SHA = "e629313eee624ccb124036ec6fd4664df9ca761e392d026ce6f2f7c34a184466";
const RESEARCH_SHA = "00b0f3054552304f28fd32f3b2d07193eb814a764e8a152967572eee6f04a7ea";
const SCHEMA = "starclock-goal-coverage-v1";
const GENERATED_ON = "2026-07-20";
const root = path.resolve(process.cwd());
const manifestRoot = path.join(root, "content-manifests", "core-combat-v1");
const evidenceRoot = path.join(root, "evidence", "core-combat-v1");
const outputRoot = path.join(evidenceRoot, "coverage");
const checkOnly = process.argv.includes("--check");

const manifestIndex = readJson(path.join(manifestRoot, "manifest-index.json"));
const referenceIndex = readJson(path.join(root, "content-reference", "v4.4", "pack-index.json"));
const provenanceIndex = readJson(path.join(evidenceRoot, "reference-binding", "evidence-index.json"));
const researchIndex = readJson(path.join(evidenceRoot, "research-register", "evidence-index.json"));
assert(referenceIndex.pack_sha256 === PACK_SHA, "reference pack digest mismatch");
assert(manifestIndex.manifest_sha256 === MANIFEST_SHA, "goal manifest digest mismatch");
assert(provenanceIndex.evidence_sha256 === PROVENANCE_SHA, "provenance evidence digest mismatch");
assert(researchIndex.evidence_sha256 === RESEARCH_SHA, "research evidence digest mismatch");

const characterManifest = readJson(path.join(manifestRoot, "released-character-forms.json"));
const coneManifest = readJson(path.join(manifestRoot, "released-light-cones.json"));
const standardManifest = readJson(path.join(manifestRoot, "standard-v1.json"));
const provenance = readJson(path.join(evidenceRoot, "reference-binding", "provenance-map.json"));
const research = readJson(path.join(evidenceRoot, "research-register", "research-cases.json"));

const mappingKeys = new Set(provenance.mappings.map((entry) => `${entry.kind}\0${entry.id}`));
const researchByCharacter = new Map();
for (const researchCase of research.cases) {
  if (researchCase.state !== "Researching") continue;
  for (const binding of researchCase.evidence) {
    const marker = ".ability.";
    const index = binding.record_id.indexOf(marker);
    if (index < 0) continue;
    const characterId = binding.record_id.slice(0, index);
    if (!researchByCharacter.has(characterId)) researchByCharacter.set(characterId, new Set());
    researchByCharacter.get(characterId).add(researchCase.id);
  }
}
const goldenCharacterForms = new Set([
  "character.acheron",
  "character.aglaea",
  "character.anaxa",
  "character.archer",
  "character.argenti",
  "character.arlan",
  "character.ashveil",
  "character.asta",
  "character.aventurine",
  "character.bailu",
  "character.black-swan",
  "character.blade",
  "character.boothill",
  "character.bronya",
  "character.castorice",
  "character.cerydra",
  "character.cipher",
  "character.clara",
  "character.cyrene",
  "character.dan-heng",
  "character.dan-heng-imbibitor-lunae",
  "character.dan-heng-permansor-terrae",
  "character.dr-ratio",
  "character.evanescia",
  "character.evernight",
  "character.feixiao",
  "character.firefly",
  "character.fu-xuan",
  "character.kafka",
  "character.silver-wolf-lv-999",
]);

const categories = [];
categories.push(category(
  "released-character-combat-forms",
  "CharacterCombatForm",
  characterManifest.entries,
  (entry) => goldenCharacterForms.has(entry.id)
    ? "GoldenVerified"
    : researchByCharacter.has(entry.id) ? "Researching" : "Documented",
  2,
));
categories.push(category("released-light-cones", "LightCone", coneManifest.entries, () => "Cataloged"));
categories.push(category("standard-v1-enemy-variants", "StandardEnemyVariant", standardManifest.enemies, () => "GoldenVerified"));
categories.push(category("standard-v1-encounters", "StandardEncounter", standardManifest.encounters, () => "GoldenVerified"));
categories.push(category("standard-v1-scenarios", "StandardScenario", standardManifest.scenarios, () => "GoldenVerified"));
categories.push(category("standard-v1-profile", "StandardProfile", [standardManifest.profile], () => "GoldenVerified"));

const entries = categories.flatMap((entry) => entry.entries);
const expectedKeys = new Set(entries.map((entry) => `${entry.manifest_kind}\0${entry.id}`));
const mappingMissing = [...expectedKeys].filter((key) => !mappingKeys.has(key)).sort();
const mappingExtra = [...mappingKeys].filter((key) => !expectedKeys.has(key)).sort();
assert(mappingMissing.length === 0 && mappingExtra.length === 0, "provenance mapping and coverage entries differ");
assert(new Set(entries.map((entry) => `${entry.manifest_kind}\0${entry.id}`)).size === entries.length, "duplicate frozen goal ID");

const disabledAudit = [
  { id: "character.gilgamesh", release_state: "Announced", enabled: false, denominator: false, reason: "Not released in the frozen enabled pack." },
  { id: "character.rin-tohsaka", release_state: "Announced", enabled: false, denominator: false, reason: "Not released in the frozen enabled pack." },
];
const terminalStateCounts = countBy(entries, (entry) => entry.terminal_state);
const dataReady = entries.filter((entry) => entry.milestones.DataReady).length;
const goldenVerified = entries.filter((entry) => entry.milestones.GoldenVerified).length;
const productionGolden = readJson(path.join(root, "config", "production-golden.json"));
const documentation = verifyDocumentation(categories);
const report = {
  schema_revision: SCHEMA,
  goal_id: "core-combat-v1",
  snapshot: "4.4",
  generated_on: GENERATED_ON,
  basis: {
    reference_pack_sha256: PACK_SHA,
    goal_manifest_sha256: MANIFEST_SHA,
    provenance_evidence_sha256: PROVENANCE_SHA,
    research_evidence_sha256: RESEARCH_SHA,
    runtime_catalog: { state: "CharacterC03Production", digest: productionGolden.files["config.sora"], note: "Pinned Sora production bundle contains frozen Standard-v1, the six representative V1B forms and all twenty-four C01-C03 character forms; remaining character and Light Cone partitions are pending." },
  },
  summary: {
    required: entries.length,
    accounted: entries.length,
    enabled_incomplete: entries.length - dataReady,
    data_ready: dataReady,
    golden_verified: goldenVerified,
    data_ready_percent: percent(dataReady, entries.length),
    terminal_state_counts: completeStates(terminalStateCounts),
    disabled_audit_only: disabledAudit.length,
  },
  categories: categories.map(({ entries: _entries, ...entry }) => entry),
  entries,
  disabled_audit: disabledAudit,
  accounting: {
    missing_manifest_ids: [],
    extra_runtime_ids: [],
    duplicate_manifest_ids: [],
    duplicate_runtime_ids: [],
    stale_version_ids: [],
    missing_data_ready_ids: entries.filter((entry) => !entry.milestones.DataReady).map((entry) => entry.id),
    not_evaluable_production_provenance_ids: entries.filter((entry) => !entry.milestones.DataReady).map((entry) => entry.id),
    not_evaluable_bilingual_validation_ids: entries.filter((entry) => !entry.milestones.DataReady && entry.category !== "standard-v1-profile").map((entry) => entry.id),
    not_evaluable_runtime_reference_ids: entries.filter((entry) => !entry.milestones.DataReady).map((entry) => entry.id),
    unresolved_native_handler_ids: [],
    unowned_research_case_ids: research.cases.filter((entry) => !entry.owner_batch).map((entry) => entry.id),
    orphaned_provenance_mappings: mappingExtra,
    missing_provenance_mappings: mappingMissing,
  },
  documentation_assertions: documentation,
  exclusions: [
    "Prepared JSON reference records receive runtime credit only through reviewed Excel/Sora production rows.",
    "Phase 4 probe fixtures are excluded from production coverage.",
    "Announced disabled forms are outside the required denominator.",
    "Challenge, universe, UI, account, relic/planar and full public enemy catalogs are not claimed.",
  ],
};

const reportText = formatJson(report);
const index = {
  schema_revision: SCHEMA,
  generated_on: GENERATED_ON,
  files: [{ name: "goal-coverage.json", sha256: sha256Text(reportText) }],
  coverage_sha256: sha256Text(`goal-coverage.json\0${sha256Text(reportText)}\n`),
};
const outputs = { "goal-coverage.json": report, "coverage-index.json": index };
if (checkOnly) {
  for (const [name, value] of Object.entries(outputs)) {
    const file = path.join(outputRoot, name);
    assert(fs.existsSync(file), `missing generated ${name}`);
    assert(fs.readFileSync(file, "utf8") === formatJson(value), `${name} has generated drift`);
  }
  console.log(`Goal coverage is current (${index.coverage_sha256}; ${dataReady}/${entries.length} DataReady).`);
} else {
  fs.mkdirSync(outputRoot, { recursive: true });
  for (const [name, value] of Object.entries(outputs)) fs.writeFileSync(path.join(outputRoot, name), formatJson(value));
  console.log(`Wrote goal coverage (${index.coverage_sha256}; ${dataReady}/${entries.length} DataReady).`);
}

function category(id, manifestKind, sourceEntries, terminalState, disabledAuditOnly = 0) {
  const categoryEntries = sourceEntries.map((source) => {
    assert(source.inclusion_state === "Required", `${source.id} is not Required`);
    assert(source.implementation_state === "Pending", `${source.id} unexpectedly claims implementation`);
    const researchCases = [...(researchByCharacter.get(source.id) ?? [])].sort();
    const state = terminalState(source);
    const ready = state === "DataReady" || state === "GoldenVerified";
    return {
      category: id,
      manifest_kind: manifestKind,
      id: source.id,
      inclusion_state: source.inclusion_state,
      manifest_implementation_state: source.implementation_state,
      terminal_state: state,
      milestones: {
        Cataloged: true,
        Documented: state !== "Cataloged",
        Researching: researchCases.length > 0,
        DataReady: ready,
        GoldenVerified: state === "GoldenVerified",
      },
      research_case_ids: researchCases,
      data_ready_blockers: ready ? [] : ["MissingExcelSoraProductionDefinition", "MissingRuntimeDomainDefinition", "MissingProductionValidation", "MissingExecutableGolden"],
    };
  });
  return {
    id,
    manifest_kind: manifestKind,
    required: categoryEntries.length,
    accounted: categoryEntries.length,
    data_ready: categoryEntries.filter((entry) => entry.milestones.DataReady).length,
    golden_verified: categoryEntries.filter((entry) => entry.milestones.GoldenVerified).length,
    data_ready_percent: percent(categoryEntries.filter((entry) => entry.milestones.DataReady).length, categoryEntries.length),
    terminal_state_counts: completeStates(countBy(categoryEntries, (entry) => entry.terminal_state)),
    disabled_audit_only: disabledAuditOnly,
    entries: categoryEntries,
  };
}

function verifyDocumentation(categoryReports) {
  const status = fs.readFileSync(path.join(root, "docs", "goals", "01-core-combat-and-content-status.md"), "utf8");
  const characterReadme = fs.readFileSync(path.join(root, "docs", "characters", "README.md"), "utf8");
  const matrix = fs.readFileSync(path.join(root, "docs", "characters", "implementation-matrix.md"), "utf8");
  const referenceCoverage = fs.readFileSync(path.join(root, "docs", "content-reference", "coverage.md"), "utf8");
  const referenceCounts = readJson(path.join(root, "content-reference", "v4.4", "coverage.json"));
  const expectedStatus = [
    ["Released character combat forms", 88, 30],
    ["Released Light Cones", 165, 0],
    ["`standard-v1` enemies/variants", 17, 17],
    ["`standard-v1` encounters", 6, 6],
    ["`standard-v1` scenarios", 6, 6],
  ];
  const statusRows = expectedStatus.map(([label, required, ready]) => {
    const line = status.split(/\r?\n/).find((candidate) => candidate.startsWith(`| ${label} |`));
    assert(line, `status counter row missing for ${label}`);
    const cells = line.split("|").map((cell) => cell.trim());
    assert(Number.parseInt(cells[3], 10) === required, `${label} required count differs`);
    assert(Number.parseInt(cells[4], 10) === ready, `${label} DataReady count differs`);
    return { label, required, data_ready: ready, matches: true };
  });
  assert(characterReadme.includes("**88 released combat forms**"), "character README released count differs");
  assert(characterReadme.includes("**2 officially announced combat forms**"), "character README announced count differs");
  const matrixRows = matrix.split(/\r?\n/).filter((line) => /^\| \d+ \|/.test(line));
  const matrixReleased = matrixRows.filter((line) => line.includes("| Released |")).length;
  const matrixAnnounced = matrixRows.filter((line) => line.includes("| Announced |")).length;
  assert(matrixReleased === 88 && matrixAnnounced === 2, `implementation matrix count differs: ${matrixReleased}/${matrixAnnounced}`);
  const referenceExpected = [
    ["Released combat forms", referenceCounts.characters.total],
    ["Character abilities", referenceCounts.character_abilities.total],
    ["Traces", referenceCounts.character_traces.total],
    ["Eidolons", referenceCounts.character_eidolons.total],
    ["Light Cones", referenceCounts.light_cones.total],
    ["Enemy templates", referenceCounts.enemy_templates.total],
    ["Enemy variants", referenceCounts.enemy_variants.total],
    ["Enemy abilities", referenceCounts.enemy_abilities.total],
    ["Ordinary encounter candidates", referenceCounts.encounters.total],
  ];
  for (const [label, count] of referenceExpected) {
    const pattern = new RegExp(`\\| ${escapeRegex(label)} \\| ${count.toLocaleString("en-US")} \\|`);
    assert(pattern.test(referenceCoverage), `reference coverage documentation differs for ${label}`);
  }
  const categoryTotal = categoryReports.reduce((sum, entry) => sum + entry.required, 0);
  return {
    all_match: true,
    goal_status_rows: statusRows,
    character_readme: { released: 88, announced_disabled: 2, matches: true },
    implementation_matrix: { released: matrixReleased, announced_disabled: matrixAnnounced, matches: true },
    reference_coverage_rows: referenceExpected.map(([label, count]) => ({ label, count, matches: true })),
    goal_required_total_including_profile: categoryTotal,
  };
}

function completeStates(counts) { return Object.fromEntries(["Cataloged", "Documented", "Researching", "DataReady", "GoldenVerified"].map((state) => [state, counts[state] ?? 0])); }
function percent(numerator, denominator) { return numerator === 0 ? "0%" : `${(100 * numerator / denominator).toFixed(1).replace(/\.0$/, "")}%`; }
function countBy(rows, key) { const out = {}; for (const row of rows) { const value = key(row); out[value] = (out[value] ?? 0) + 1; } return out; }
function escapeRegex(value) { return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function formatJson(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256Text(value) { return crypto.createHash("sha256").update(value, "utf8").digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
