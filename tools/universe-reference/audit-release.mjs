import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const packRoot = path.join(root, "content-reference", "standard-universe-v1");
const readPack = async (file) => JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
const readRoot = async (file) => JSON.parse(await readFile(path.join(root, ...file.split("/")), "utf8"));
const assert = (condition, message) => { if (!condition) throw new Error(message); };

const schema = await readPack("schema.json");
const coverage = await readPack("coverage.json");
const sources = await readPack("sources.json");
const manifest = await readRoot("content-manifests/standard-universe-v1/content-manifest.json");
const rules = await readPack("mechanic-rules.json");
const fixtures = await readPack("review-fixtures.json");
const enemyVariants = await readRoot("content-reference/v4.4/enemy-variants.json");
const quality = new Set(schema.enums.quality);
const sourceById = new Map(sources.map((row) => [row.id, row]));
const enemyIds = new Set(enemyVariants.map((row) => row.id));

assert(sourceById.size === sources.length, "duplicate source ID");
for (const source of sources) {
  assert(source.game_version === "4.4", `${source.id}: wrong game version`);
  assert(source.repository_or_url.trim(), `${source.id}: repository/URL missing`);
  assert(source.revision_or_access_date.trim(), `${source.id}: revision/access date missing`);
  assert(source.relative_path_or_page.trim() && source.row_locator.trim(), `${source.id}: source locator missing`);
  assert(/^[0-9a-f]{64}$/u.test(source.evidence_sha256), `${source.id}: evidence digest invalid`);
  assert(quality.has(source.quality), `${source.id}: quality invalid`);
  assert(source.license_note.trim(), `${source.id}: license note missing`);
}

const rowsByFile = new Map();
const contentById = new Map();
const referencedSources = new Set();
const commonFields = schema.common_record.required;
for (const category of coverage.categories) {
  const rows = await readPack(category.file);
  rowsByFile.set(category.file, rows);
  assert(rows.length === category.required, `${category.file}: required count drifted`);
  assert(rows.length === category.accounted, `${category.file}: accounted count drifted`);
  assert(rows.length === category.data_ready, `${category.file}: DataReady count drifted`);
  assert(category.coverage_percent === "100", `${category.file}: coverage is not 100%`);
  for (const row of rows) {
    assert(!contentById.has(row.id), `${category.file}/${row.id}: duplicate global stable ID`);
    contentById.set(row.id, { file: category.file, row });
    for (const field of commonFields) assert(Object.hasOwn(row, field), `${category.file}/${row.id}: missing ${field}`);
    assert(row.enabled === true, `${category.file}/${row.id}: released Standard row disabled`);
    assert(["Standard", "Shared"].includes(row.mode_owner), `${category.file}/${row.id}: non-Standard owner leaked`);
    assert(row.coverage_state === "DataReady", `${category.file}/${row.id}: not DataReady`);
    assert(quality.has(row.quality) && quality.has(row.mechanism_quality), `${category.file}/${row.id}: invalid quality`);
    for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
      assert(row[field]?.trim(), `${category.file}/${row.id}: bilingual field ${field} missing`);
    assert(row.provenance_ids.length > 0, `${category.file}/${row.id}: provenance missing`);
    for (const sourceId of row.provenance_ids) {
      assert(sourceById.has(sourceId), `${category.file}/${row.id}: source ${sourceId} missing`);
      referencedSources.add(sourceId);
    }
    if (["ProjectPolicy", "ApproximateFromReleasedText"].includes(row.mechanism_quality))
      assert(row.note.trim(), `${category.file}/${row.id}: approximation replacement note missing`);
  }
}

assert(contentById.size === coverage.required, "global content denominator drifted");
assert(coverage.required === coverage.data_ready && coverage.coverage_percent === "100", "aggregate coverage drifted");
assert(coverage.blocking_gaps.length === 0, "blocking coverage gaps remain");
for (const source of sources) assert(referencedSources.has(source.id), `${source.id}: orphan provenance record`);

const rows = (file) => rowsByFile.get(file);
const ids = (file) => new Set(rows(file).map((row) => row.id));
const requireId = (owner, value, target, field) => assert(target.has(value), `${owner}: missing ${field} ${value}`);
const requireMany = (owner, values, target, field) => {
  for (const value of values ?? []) requireId(owner, value, target, field);
};

const worldIds = ids("worlds.json");
const difficultyIds = ids("world-difficulties.json");
const domainIds = ids("domains.json");
const mapIds = ids("maps.json");
const roomIds = ids("rooms.json");
const pathIds = ids("paths.json");
const resonanceIds = ids("resonances.json");
const blessingIds = ids("blessings.json");
const blessingLevelIds = ids("blessing-levels.json");
const curioIds = ids("curios.json");
const curioStateIds = ids("curio-states.json");
const occurrenceIds = ids("occurrences.json");
const variantIds = ids("occurrence-variants.json");
const choiceIds = ids("occurrence-choices.json");
const serviceIds = ids("services.json");
const abilityIds = ids("ability-tree.json");
const encounterGroupIds = ids("encounter-groups.json");
const ruleIds = new Set(rules.map((row) => row.id));

for (const row of rows("worlds.json")) requireMany(row.id, row.difficulty_ids, difficultyIds, "difficulty");
for (const row of rows("world-difficulties.json")) {
  requireId(row.id, row.world_id, worldIds, "world");
  requireMany(row.id, row.boss_variant_ids.map((binding) => binding.enemy_variant_id), enemyIds, "boss variant");
  requireMany(row.id, row.elite_variant_ids.map((binding) => binding.enemy_variant_id), enemyIds, "elite variant");
}
for (const row of rows("maps.json")) {
  if (row.domain_id) requireId(row.id, row.domain_id, domainIds, "domain");
  requireMany(row.id, row.next_node_ids, mapIds, "next node");
}
for (const row of rows("rooms.json")) {
  requireId(row.id, row.domain_id, domainIds, "domain");
  assert(Number(row.map_entrance) < 8_110_000, `${row.id}: DLC room entrance leaked`);
}
for (const row of rows("paths.json")) {
  requireId(row.id, row.resonance_id, resonanceIds, "resonance");
  requireMany(row.id, row.formation_ids, resonanceIds, "formation");
  requireMany(row.id, row.blessing_ids, blessingIds, "blessing");
}
for (const row of rows("resonances.json")) {
  requireId(row.id, row.path_id, pathIds, "path");
  requireMany(row.id, row.rule_ids, ruleIds, "rule");
}
for (const row of rows("blessings.json")) {
  requireId(row.id, row.path_id, pathIds, "path");
  requireMany(row.id, row.level_ids, blessingLevelIds, "level");
  requireMany(row.id, row.rule_ids, ruleIds, "rule");
}
for (const row of rows("blessing-levels.json")) {
  requireId(row.id, row.blessing_id, blessingIds, "blessing");
  requireMany(row.id, row.rule_ids, ruleIds, "rule");
}
for (const row of rows("curios.json")) {
  requireId(row.id, row.initial_state_id, curioStateIds, "initial state");
  requireMany(row.id, row.state_ids, curioStateIds, "state");
  requireMany(row.id, row.rule_ids, ruleIds, "rule");
}
for (const row of rows("curio-states.json")) {
  requireId(row.id, row.curio_id, curioIds, "curio");
  if (row.next_state_id) requireId(row.id, row.next_state_id, curioStateIds, "next state");
  if (row.repair_state_id) requireId(row.id, row.repair_state_id, curioStateIds, "repair state");
  if (row.replacement_curio_id) requireId(row.id, row.replacement_curio_id, curioIds, "replacement Curio");
  requireMany(row.id, row.rule_ids, ruleIds, "rule");
  assert(Number(row.source_effect_id) < 1_000, `${row.id}: DLC Curio effect copy leaked`);
}
for (const row of rows("occurrences.json")) requireMany(row.id, row.variant_ids, variantIds, "variant");
for (const row of rows("occurrence-variants.json")) {
  requireId(row.id, row.occurrence_id, occurrenceIds, "occurrence");
  requireMany(row.id, row.choice_ids, choiceIds, "choice");
  assert(Number(row.source_ids[0]) < 100_000, `${row.id}: DLC occurrence variant leaked`);
}
for (const row of rows("occurrence-choices.json")) requireId(row.id, row.variant_id, variantIds, "variant");
for (const row of rows("services.json")) requireMany(row.id, row.rule_ids, ruleIds, "rule");
for (const row of rows("ability-tree.json")) {
  requireMany(row.id, row.prerequisite_ids, abilityIds, "prerequisite");
  requireMany(row.id, row.next_ids, abilityIds, "successor");
  requireMany(row.id, row.rule_ids, ruleIds, "rule");
}
for (const row of rows("encounter-pools.json")) {
  requireId(row.id, row.room_id, roomIds, "room");
  requireMany(row.id, row.world_ids, worldIds, "world");
  requireMany(row.id, row.difficulty_ids, difficultyIds, "difficulty");
  for (const binding of row.weighted_group_ids) requireId(row.id, binding.group_id, encounterGroupIds, "group");
}
for (const row of rows("encounter-groups.json"))
  for (const member of row.weighted_member_ids)
    for (const wave of member.waves)
      for (const enemy of wave.enemy_variant_ids) requireId(row.id, enemy.enemy_variant_id, enemyIds, "enemy variant");

const allReleaseIds = new Set([...contentById.keys(), ...ruleIds]);
for (const row of rules) {
  assert(row.enabled && ["Standard", "Shared"].includes(row.mode_owner), `${row.id}: invalid rule ownership`);
  requireId(row.id, row.source_record_id, new Set(contentById.keys()), "source record");
  if (["ProjectPolicy", "ApproximateFromReleasedText"].includes(row.mechanism_quality))
    assert(row.approximation_replacement_condition.trim(), `${row.id}: approximation replacement condition missing`);
}
for (const fixture of fixtures) requireMany(fixture.id, fixture.input_ids, allReleaseIds, "fixture input");

const sourceLocatorKeys = new Set(sources.map((row) => `${row.relative_path_or_page}#${row.row_locator}#${row.evidence_sha256}`));
for (const [category, manifestCategory] of Object.entries(manifest.categories)) {
  for (const row of manifestCategory.records) {
    assert(/^[0-9a-f]{64}$/u.test(row.evidence_sha256), `${category}/${row.id}: manifest digest invalid`);
    if (category !== "worlds")
      assert(sourceLocatorKeys.has(`${row.source}#${row.evidence_sha256}`), `${category}/${row.id}: manifest evidence is not promoted`);
  }
}

console.log(
  `Standard universe release audit passed: ${contentById.size} content rows, ${rules.length} rules, ` +
  `${fixtures.length} fixtures, ${sources.length} provenance rows and ${manifest.categories.rooms.count} manifest rooms.`,
);
