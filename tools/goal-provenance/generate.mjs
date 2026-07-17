import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const PACK_SHA = "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a";
const MANIFEST_SHA = "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19";
const SCHEMA = "starclock-goal-provenance-v1";
const GENERATED_ON = "2026-07-17";
const root = path.resolve(process.cwd());
const refRoot = path.join(root, "content-reference", "v4.4");
const manifestRoot = path.join(root, "content-manifests", "core-combat-v1");
const outputRoot = path.join(root, "evidence", "core-combat-v1", "reference-binding");
const cacheRoot = path.join(root, ".cache", "content-reference");
const regeneratedRoot = path.join(cacheRoot, "regenerated-v4.4");
const checkOnly = process.argv.includes("--check");

const packIndex = readJson(path.join(refRoot, "pack-index.json"));
const manifestIndex = readJson(path.join(manifestRoot, "manifest-index.json"));
assert(packIndex.pack_sha256 === PACK_SHA, `reference pack digest mismatch: ${packIndex.pack_sha256}`);
assert(manifestIndex.manifest_sha256 === MANIFEST_SHA, `goal manifest digest mismatch: ${manifestIndex.manifest_sha256}`);
verifyIndexedFiles(refRoot, packIndex.files);
verifyIndexedFiles(manifestRoot, manifestIndex.files);

const refs = {
  characters: readJson(path.join(refRoot, "characters.json")),
  abilities: readJson(path.join(refRoot, "character-abilities.json")),
  traces: readJson(path.join(refRoot, "character-traces.json")),
  eidolons: readJson(path.join(refRoot, "character-eidolons.json")),
  lightCones: readJson(path.join(refRoot, "light-cones.json")),
  templates: readJson(path.join(refRoot, "enemy-templates.json")),
  variants: readJson(path.join(refRoot, "enemy-variants.json")),
  enemyAbilities: readJson(path.join(refRoot, "enemy-abilities.json")),
  encounters: readJson(path.join(refRoot, "encounters.json")),
  sources: readJson(path.join(refRoot, "sources.json")),
};
const maps = Object.fromEntries(Object.entries(refs).map(([name, rows]) => [name, byId(rows)]));
const characterManifest = readJson(path.join(manifestRoot, "released-character-forms.json"));
const coneManifest = readJson(path.join(manifestRoot, "released-light-cones.json"));
const standardManifest = readJson(path.join(manifestRoot, "standard-v1.json"));
const referenceManifest = readJson(path.join(refRoot, "manifest.json"));

const repositories = {
  "dimbreath-turnbasedgamedata": path.join(cacheRoot, "turnbasedgamedata"),
  "mar-7th-star-rail-res": path.join(cacheRoot, "StarRailRes"),
};
const cacheReport = verifyCaches(referenceManifest.repositories, refs.sources, repositories);
const regeneration = verifyRegeneration();
const saberArcherAudit = auditFallbackCharacters(repositories);

const sourcePaths = {
  avatar: [
    "ExcelOutput/AvatarConfig.json",
    "ExcelOutput/AvatarPromotionConfig.json",
    "ExcelOutput/AvatarRankConfig.json",
    "ExcelOutput/AvatarSkillConfig.json",
    "ExcelOutput/AvatarSkillTreeConfig.json",
  ],
  cone: [
    "ExcelOutput/EquipmentConfig.json",
    "ExcelOutput/EquipmentPromotionConfig.json",
    "ExcelOutput/EquipmentSkillConfig.json",
  ],
  enemy: [
    "ExcelOutput/MonsterConfig.json",
    "ExcelOutput/MonsterSkillConfig.json",
    "ExcelOutput/MonsterStatusConfig.json",
    "ExcelOutput/MonsterTemplateConfig.json",
  ],
  encounter: ["ExcelOutput/StageConfig.json"],
  fallback: [
    "index_new/en/characters.json",
    "index_new/cn/characters.json",
    "index_new/en/character_promotions.json",
    "index_new/en/character_skills.json",
    "index_new/cn/character_skills.json",
    "index_new/en/character_skill_trees.json",
    "index_new/cn/character_skill_trees.json",
    "index_new/en/character_ranks.json",
    "index_new/cn/character_ranks.json",
  ],
};
const sourceId = (repository, sourcePath) => {
  const found = refs.sources.find((entry) => entry.repository === repository && entry.path === sourcePath);
  assert(found, `source inventory is missing ${repository}:${sourcePath}`);
  return found.id;
};
const sourceIds = (repository, paths) => paths.map((sourcePath) => sourceId(repository, sourcePath));

const mappings = [];
const closure = new Map();
for (const entry of characterManifest.entries) {
  const character = requireMap(maps.characters, entry.reference_id, "character");
  const abilities = character.ability_ids.map((id) => requireMap(maps.abilities, id, "character ability"));
  const traces = character.trace_ids.map((id) => requireMap(maps.traces, id, "character trace"));
  const eidolons = character.eidolon_ids.map((id) => requireMap(maps.eidolons, id, "character eidolon"));
  addClosure(closure, "characters.json", character);
  abilities.forEach((row) => addClosure(closure, "character-abilities.json", row));
  traces.forEach((row) => addClosure(closure, "character-traces.json", row));
  eidolons.forEach((row) => addClosure(closure, "character-eidolons.json", row));
  const provenance = character.quality === "ExactPreviousRelease"
    ? sourceIds("mar-7th-star-rail-res", sourcePaths.fallback)
    : unique([
      ...sourceIds("dimbreath-turnbasedgamedata", sourcePaths.avatar),
      ...abilities.flatMap((row) => row.source_ability_files ?? []),
    ]);
  mappings.push({
    kind: "CharacterCombatForm",
    id: entry.id,
    reference_id: character.id,
    reference_quality: character.quality,
    record_ids: {
      character: character.id,
      abilities: character.ability_ids,
      traces: character.trace_ids,
      eidolons: character.eidolon_ids,
    },
    source_file_ids: provenance,
    binding: character.quality === "ExactPreviousRelease" ? "PinnedPreviousReleaseFallback" : "PreparedStructuredReference",
  });
}
for (const entry of coneManifest.entries) {
  const cone = requireMap(maps.lightCones, entry.reference_id, "Light Cone");
  addClosure(closure, "light-cones.json", cone);
  mappings.push({
    kind: "LightCone",
    id: entry.id,
    reference_id: cone.id,
    reference_quality: cone.quality,
    record_ids: { light_cone: cone.id },
    source_file_ids: sourceIds("dimbreath-turnbasedgamedata", sourcePaths.cone),
    binding: "PreparedStructuredReference",
  });
}
for (const entry of standardManifest.enemies) {
  const variant = requireMap(maps.variants, entry.variant_reference_id, "enemy variant");
  const template = requireMap(maps.templates, entry.template_reference_id, "enemy template");
  assert(variant.enemy_id === template.id, `${variant.id} does not belong to ${template.id}`);
  const abilities = template.ability_ids.map((id) => requireMap(maps.enemyAbilities, id, "enemy ability"));
  addClosure(closure, "enemy-variants.json", variant);
  addClosure(closure, "enemy-templates.json", template);
  abilities.forEach((row) => addClosure(closure, "enemy-abilities.json", row));
  mappings.push({
    kind: "StandardEnemyVariant",
    id: entry.id,
    reference_id: variant.id,
    reference_quality: variant.quality,
    record_ids: { variant: variant.id, template: template.id, abilities: template.ability_ids },
    source_file_ids: unique([
      ...sourceIds("dimbreath-turnbasedgamedata", sourcePaths.enemy),
      template.source_character_config?.source_file_id,
      template.source_ai?.source_file_id,
      ...abilities.flatMap((row) => row.source_ability_files ?? []),
    ].filter(Boolean)),
    binding: "PreparedStructuredReference",
  });
}
for (const entry of standardManifest.encounters) {
  const encounter = requireMap(maps.encounters, entry.reference_id, "encounter");
  assert(encounter.source_stage_id === entry.source_stage_id, `stage mismatch for ${entry.id}`);
  addClosure(closure, "encounters.json", encounter);
  mappings.push({
    kind: "StandardEncounter",
    id: entry.id,
    reference_id: encounter.id,
    reference_quality: encounter.quality,
    record_ids: { encounter: encounter.id },
    source_file_ids: sourceIds("dimbreath-turnbasedgamedata", sourcePaths.encounter),
    binding: "PreparedStructuredReference",
  });
}
for (const scenario of standardManifest.scenarios) {
  requireMap(maps.encounters, scenario.encounter_id, "scenario encounter");
  for (const build of scenario.builds) {
    requireMap(maps.characters, build.form_id, "scenario character");
    requireMap(maps.lightCones, build.light_cone_id, "scenario Light Cone");
  }
  mappings.push({
    kind: "StandardScenario",
    id: scenario.id,
    reference_id: scenario.encounter_id,
    reference_quality: "ProjectPolicyComposition",
    record_ids: {
      encounter: scenario.encounter_id,
      characters: unique(scenario.builds.map((build) => build.form_id)),
      light_cones: unique(scenario.builds.map((build) => build.light_cone_id)),
    },
    source_file_ids: [],
    binding: "FrozenManifestComposition",
  });
}
mappings.push({
  kind: "StandardProfile",
  id: standardManifest.profile.id,
  reference_id: null,
  reference_quality: "ProjectPolicyComposition",
  record_ids: {},
  source_file_ids: [],
  binding: "NormativeProjectPolicy",
});
mappings.sort((a, b) => a.kind.localeCompare(b.kind) || a.id.localeCompare(b.id));
for (const mapping of mappings) {
  for (const id of mapping.source_file_ids) requireMap(maps.sources, id, "source file");
}

const closureRows = [...closure.values()].sort((a, b) => a.file.localeCompare(b.file) || a.record.id.localeCompare(b.record.id));
const closureSummary = summarizeClosure(closureRows);
const provenanceMap = {
  schema_revision: SCHEMA,
  goal_id: "core-combat-v1",
  generated_on: GENERATED_ON,
  reference_pack_sha256: PACK_SHA,
  goal_manifest_sha256: MANIFEST_SHA,
  mapping_count: mappings.length,
  mappings,
  required_reference_closure: closureSummary,
};
const sourceCacheReport = {
  schema_revision: SCHEMA,
  generated_on: GENERATED_ON,
  reference_pack_sha256: PACK_SHA,
  repositories: cacheReport.repositories,
  source_inventory: cacheReport.source_inventory,
  regeneration,
  required_reference_closure: closureSummary,
};

const outputs = {
  "provenance-map.json": provenanceMap,
  "saber-archer-audit.json": saberArcherAudit,
  "source-cache-report.json": sourceCacheReport,
};
const indexFiles = Object.entries(outputs).sort(([a], [b]) => a.localeCompare(b)).map(([name, value]) => ({
  name,
  sha256: sha256Text(formatJson(value)),
}));
const evidenceIndex = {
  schema_revision: SCHEMA,
  generated_on: GENERATED_ON,
  files: indexFiles,
  evidence_sha256: sha256Text(indexFiles.map((entry) => `${entry.name}\0${entry.sha256}\n`).join("")),
};
outputs["evidence-index.json"] = evidenceIndex;

if (checkOnly) {
  for (const [name, value] of Object.entries(outputs)) {
    const file = path.join(outputRoot, name);
    assert(fs.existsSync(file), `generated evidence is missing ${relative(file)}`);
    assert(fs.readFileSync(file, "utf8") === formatJson(value), `${relative(file)} has generated drift`);
  }
  console.log(`Goal provenance evidence is current (${evidenceIndex.evidence_sha256}).`);
} else {
  fs.mkdirSync(outputRoot, { recursive: true });
  for (const [name, value] of Object.entries(outputs)) fs.writeFileSync(path.join(outputRoot, name), formatJson(value));
  console.log(`Wrote ${Object.keys(outputs).length} evidence files (${evidenceIndex.evidence_sha256}).`);
}

function verifyCaches(expectedRepositories, sources, roots) {
  const repositoriesReport = expectedRepositories.map((expected) => {
    const cache = roots[expected.id];
    assert(cache && fs.existsSync(cache), `missing cache for ${expected.id}`);
    const actualRevision = git(cache, ["rev-parse", "HEAD"]);
    assert(actualRevision === expected.revision, `${expected.id} revision mismatch: ${actualRevision}`);
    const remote = git(cache, ["remote", "get-url", "origin"]);
    assert(remote === expected.remote, `${expected.id} remote mismatch: ${remote}`);
    return { id: expected.id, expected_revision: expected.revision, actual_revision: actualRevision, remote, verified: true };
  }).sort((a, b) => a.id.localeCompare(b.id));
  const byRepository = {};
  for (const source of sources) {
    const cache = roots[source.repository];
    assert(cache, `unknown source repository ${source.repository}`);
    const file = path.join(cache, ...source.path.split("/"));
    assert(fs.existsSync(file), `source cache is missing ${source.repository}:${source.path}`);
    assert(sha256File(file) === source.sha256, `source hash mismatch for ${source.repository}:${source.path}`);
    byRepository[source.repository] = (byRepository[source.repository] ?? 0) + 1;
  }
  return {
    repositories: repositoriesReport,
    source_inventory: { expected_files: sources.length, verified_files: sources.length, by_repository: sortObject(byRepository), all_hashes_match: true },
  };
}

function verifyRegeneration() {
  assert(fs.existsSync(regeneratedRoot), `missing regenerated pack ${relative(regeneratedRoot)}`);
  const committedFiles = fs.readdirSync(refRoot).filter((name) => name.endsWith(".json")).sort();
  const regeneratedFiles = fs.readdirSync(regeneratedRoot).filter((name) => name.endsWith(".json")).sort();
  assert(JSON.stringify(committedFiles) === JSON.stringify(regeneratedFiles), "regenerated pack file list differs");
  for (const name of committedFiles) {
    assert(sha256File(path.join(refRoot, name)) === sha256File(path.join(regeneratedRoot, name)), `regenerated ${name} differs`);
  }
  const regeneratedIndex = readJson(path.join(regeneratedRoot, "pack-index.json"));
  assert(regeneratedIndex.pack_sha256 === PACK_SHA, `regenerated pack digest mismatch: ${regeneratedIndex.pack_sha256}`);
  return { generator: "tools/content-reference/bootstrap.mjs", compared_files: committedFiles.length, all_files_match: true, pack_sha256: PACK_SHA };
}

function auditFallbackCharacters(roots) {
  const turnAvatars = readJson(path.join(roots["dimbreath-turnbasedgamedata"], "ExcelOutput", "AvatarConfig.json"));
  const resRoot = roots["mar-7th-star-rail-res"];
  const en = {
    characters: readJson(path.join(resRoot, "index_new", "en", "characters.json")),
    skills: readJson(path.join(resRoot, "index_new", "en", "character_skills.json")),
    traces: readJson(path.join(resRoot, "index_new", "en", "character_skill_trees.json")),
    ranks: readJson(path.join(resRoot, "index_new", "en", "character_ranks.json")),
  };
  const expected = {
    "character.saber": { source_id: "1014", path: "Destruction", raw_path: "Warrior", element: "Wind", max_energy: "360" },
    "character.archer": { source_id: "1015", path: "The Hunt", raw_path: "Rogue", element: "Quantum", max_energy: "220" },
  };
  const audits = [];
  for (const [id, expectation] of Object.entries(expected)) {
    const prepared = requireMap(maps.characters, id, "fallback character");
    const raw = en.characters[expectation.source_id];
    assert(raw, `fallback source is missing ${expectation.source_id}`);
    const absent = !turnAvatars.some((row) => String(row.AvatarID) === expectation.source_id);
    assert(absent, `${id} unexpectedly exists in pinned 4.4 AvatarConfig`);
    assert(prepared.quality === "ExactPreviousRelease", `${id} quality was relabeled`);
    assert(raw.path === expectation.raw_path && prepared.path === expectation.path, `${id} path mismatch`);
    assert(raw.element === expectation.element && prepared.element === expectation.element, `${id} element mismatch`);
    assert(String(raw.max_sp) === expectation.max_energy && prepared.max_energy === expectation.max_energy, `${id} energy mismatch`);
    assert(raw.rarity === prepared.rarity && prepared.rarity === 5, `${id} rarity mismatch`);
    const abilities = prepared.ability_ids.map((rowId) => requireMap(maps.abilities, rowId, "fallback ability"));
    const traces = prepared.trace_ids.map((rowId) => requireMap(maps.traces, rowId, "fallback trace"));
    const eidolons = prepared.eidolon_ids.map((rowId) => requireMap(maps.eidolons, rowId, "fallback eidolon"));
    assertSetEqual(abilities.flatMap((row) => row.source_skill_ids), raw.skills, `${id} skill IDs`);
    assertSetEqual(traces.flatMap((row) => row.source_point_ids), raw.skill_trees, `${id} trace IDs`);
    assertSetEqual(eidolons.flatMap((row) => row.source_rank_ids), raw.ranks, `${id} rank IDs`);
    for (const row of abilities) {
      assert(row.quality === "ExactPreviousRelease" && row.mechanism_quality === "ExactPreviousReleaseText", `${row.id} quality mismatch`);
      verifyFallbackText(row, en.skills[row.source_skill_ids[0]]?.desc);
    }
    for (const row of traces) {
      assert(row.quality === "ExactPreviousRelease", `${row.id} quality mismatch`);
      verifyFallbackText(row, en.traces[row.source_point_ids[0]]?.desc);
    }
    for (const row of eidolons) {
      assert(row.quality === "ExactPreviousRelease", `${row.id} quality mismatch`);
      verifyFallbackText(row, en.ranks[row.source_rank_ids[0]]?.desc);
    }
    audits.push({
      id,
      source_avatar_id: expectation.source_id,
      absent_from_pinned_4_4_avatar_config: true,
      fallback_repository: "mar-7th-star-rail-res",
      fallback_revision: referenceManifest.repositories.find((repo) => repo.id === "mar-7th-star-rail-res").revision,
      identity: { name_en: prepared.name_en, path: prepared.path, element: prepared.element, rarity: prepared.rarity, max_energy: prepared.max_energy },
      child_counts: { abilities: abilities.length, traces: traces.length, eidolons: eidolons.length },
      source_id_sets_match: true,
      released_text_hashes_match: true,
      retained_quality: "ExactPreviousRelease",
    });
  }
  return {
    schema_revision: SCHEMA,
    generated_on: GENERATED_ON,
    case_id: "G01-R-SABER-ARCHER-SOURCE",
    conclusion: "VerifiedPreviousReleaseFallback",
    characters: audits,
    decision: "Retain the pinned previous-release records because source IDs 1014 and 1015 are absent from the pinned 4.4 structured AvatarConfig; preserve ExactPreviousRelease labels and bind their released-text evidence hashes to the pinned fallback revision.",
  };
}

function verifyFallbackText(record, rawText) {
  const expectedHash = rawText ? sha256Text(rawText) : "";
  assert(record.source_text.sha256 === expectedHash, `${record.id} released-text hash mismatch`);
  assert(record.source_text.source_hash === "" && record.source_text.emitted === false, `${record.id} fallback evidence metadata mismatch`);
}

function summarizeClosure(rows) {
  const byFile = {};
  const byQuality = {};
  const byMechanismQuality = {};
  let evidencePresent = 0;
  let evidenceAbsent = 0;
  let approximations = 0;
  for (const { file, record } of rows) {
    byFile[file] = (byFile[file] ?? 0) + 1;
    byQuality[record.quality ?? "NotApplicable"] = (byQuality[record.quality ?? "NotApplicable"] ?? 0) + 1;
    if (record.mechanism_quality) byMechanismQuality[record.mechanism_quality] = (byMechanismQuality[record.mechanism_quality] ?? 0) + 1;
    if (record.source_text) {
      if (record.source_text.sha256) evidencePresent += 1;
      else evidenceAbsent += 1;
      if (String(record.mechanism_quality ?? "").startsWith("Approximate")) {
        approximations += 1;
        assert(/^[0-9a-f]{64}$/.test(record.source_text.sha256), `${record.id} has an unbound approximation`);
      }
    }
    if (record.passive?.source_text) {
      if (record.passive.source_text.sha256) evidencePresent += 1;
      else evidenceAbsent += 1;
    }
  }
  return {
    records: rows.length,
    by_file: sortObject(byFile),
    by_quality: sortObject(byQuality),
    by_mechanism_quality: sortObject(byMechanismQuality),
    source_text_evidence: { present: evidencePresent, absent_for_structured_non_approximation: evidenceAbsent, approximations, unbound_approximations: 0 },
    verification: "Every record and evidence field matched a full regeneration from the pinned source cache.",
  };
}

function addClosure(closure, file, record) {
  closure.set(`${file}\0${record.id}`, { file, record });
}
function verifyIndexedFiles(directory, files) {
  for (const entry of files) assert(sha256File(path.join(directory, entry.name)) === entry.sha256, `${relative(path.join(directory, entry.name))} hash mismatch`);
}
function byId(rows) { return new Map(rows.map((row) => [row.id, row])); }
function requireMap(map, id, label) { const row = map.get(id); assert(row, `missing ${label} ${id}`); return row; }
function unique(values) { return [...new Set(values)].sort(); }
function sortObject(value) { return Object.fromEntries(Object.entries(value).sort(([a], [b]) => a.localeCompare(b))); }
function assertSetEqual(actual, expected, label) { assert(JSON.stringify(unique(actual.map(String))) === JSON.stringify(unique(expected.map(String))), `${label} mismatch`); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function formatJson(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256File(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function sha256Text(value) { return crypto.createHash("sha256").update(value, "utf8").digest("hex"); }
function git(directory, args) { return execFileSync("git", ["-C", directory, ...args], { encoding: "utf8" }).trim(); }
function relative(file) { return path.relative(root, file).replaceAll("\\", "/"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
