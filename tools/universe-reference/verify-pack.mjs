import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const packRoot = path.join(root, "content-reference", "standard-universe-v1");
const raw = async (file) => readFile(path.join(packRoot, file), "utf8");
const read = async (file) => JSON.parse(await raw(file));
const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const assert = (condition, message) => { if (!condition) throw new Error(message); };

const schema = await read("schema.json");
const index = await read("pack-index.json");
const sources = await read("sources.json");
const sourceIds = new Set(sources.map((row) => row.id));
assert(sourceIds.size === sources.length, "duplicate provenance ID");

const records = new Map();
for (const file of schema.files) {
  const value = await read(file);
  records.set(file, value);
  if (!Array.isArray(value)) continue;
  const ids = value.map((row) => row.id);
  assert(ids.length === new Set(ids).size, `${file} contains duplicate stable IDs`);
  if (file === "sources.json") continue;
  for (const row of value) {
    for (const field of schema.common_record.required) assert(Object.hasOwn(row, field), `${file}/${row.id} lacks ${field}`);
    if (["DataReady", "GoldenVerified"].includes(row.coverage_state)) {
      for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"]) assert(row[field]?.trim(), `${file}/${row.id} lacks ${field}`);
      assert(row.provenance_ids.length > 0, `${file}/${row.id} lacks provenance`);
    }
    for (const id of row.provenance_ids) assert(sourceIds.has(id), `${file}/${row.id} references missing provenance ${id}`);
    if (["ProjectPolicy", "ApproximateFromReleasedText"].includes(row.mechanism_quality)) assert(row.note.trim(), `${file}/${row.id} lacks approximation note`);
  }
}

const indexFiles = new Map(index.files.map((entry) => [entry.file, entry]));
assert(indexFiles.size === 24, `expected 24 indexed files, got ${indexFiles.size}`);
for (const [file, entry] of indexFiles) {
  const bytes = await raw(file);
  assert(entry.bytes === Buffer.byteLength(bytes), `${file} byte count drifted`);
  assert(entry.sha256 === sha256(bytes), `${file} digest drifted`);
}
const orderedIndex = [...index.files].sort((left, right) => left.file.localeCompare(right.file));
assert(index.pack_sha256 === sha256(orderedIndex.map((entry) => `${entry.file}\0${entry.sha256}`).join("\n")), "pack digest drifted");

const expect = (file, count) => assert(records.get(file).length === count, `${file} expected ${count} rows`);
expect("worlds.json", 9); expect("world-difficulties.json", 33); expect("domains.json", 9);
expect("maps.json", 579); expect("rooms.json", 163); expect("paths.json", 9); expect("resonances.json", 36);
expect("blessings.json", 162); expect("blessing-levels.json", 324); expect("curios.json", 61); expect("curio-states.json", 67);
expect("occurrences.json", 59); expect("occurrence-variants.json", 67); expect("occurrence-choices.json", 321);
expect("services.json", 94); expect("ability-tree.json", 42); expect("encounter-groups.json", 74); expect("encounter-pools.json", 92);

function idSet(file) { return new Set(records.get(file).map((row) => row.id)); }
function requireRefs(file, field, target) {
  const targets = idSet(target);
  for (const row of records.get(file)) for (const id of row[field] ?? []) assert(targets.has(id), `${file}/${row.id} has missing ${field} ${id}`);
}
requireRefs("worlds.json", "difficulty_ids", "world-difficulties.json");
requireRefs("paths.json", "blessing_ids", "blessings.json");
requireRefs("paths.json", "formation_ids", "resonances.json");
requireRefs("blessings.json", "level_ids", "blessing-levels.json");
requireRefs("curios.json", "state_ids", "curio-states.json");
requireRefs("occurrences.json", "variant_ids", "occurrence-variants.json");
requireRefs("occurrence-variants.json", "choice_ids", "occurrence-choices.json");
requireRefs("ability-tree.json", "prerequisite_ids", "ability-tree.json");
requireRefs("ability-tree.json", "next_ids", "ability-tree.json");

const domainIds = idSet("domains.json");
for (const room of records.get("rooms.json")) assert(domainIds.has(room.domain_id), `room ${room.id} has missing domain`);
const groupIds = idSet("encounter-groups.json");
const roomIds = idSet("rooms.json");
for (const pool of records.get("encounter-pools.json")) {
  assert(roomIds.has(pool.room_id), `${pool.id} has missing room`);
  for (const binding of pool.weighted_group_ids) assert(groupIds.has(binding.group_id), `${pool.id} has missing encounter group`);
}
const enemyVariants = new Set(JSON.parse(await readFile(path.join(root, "content-reference", "v4.4", "enemy-variants.json"), "utf8")).map((row) => row.id));
for (const group of records.get("encounter-groups.json")) for (const member of group.weighted_member_ids) for (const wave of member.waves) for (const enemy of wave.enemy_variant_ids) assert(enemyVariants.has(enemy.enemy_variant_id), `${group.id} has missing Goal 01 enemy ${enemy.enemy_variant_id}`);

const ruleIds = idSet("mechanic-rules.json");
for (const file of ["resonances.json", "blessings.json", "blessing-levels.json", "curios.json", "curio-states.json", "services.json", "ability-tree.json"]) {
  for (const row of records.get(file)) for (const id of row.rule_ids) assert(ruleIds.has(id), `${file}/${row.id} has missing mechanic rule ${id}`);
}
const coverage = records.get("coverage.json");
assert(coverage.coverage_percent === "100" && coverage.required === coverage.data_ready, "coverage is not 100% DataReady");
assert(coverage.blocking_gaps.length === 0, "coverage contains blocking gaps");
const fixtures = records.get("review-fixtures.json");
assert(fixtures.length >= 40, "mechanic fixture coverage is unexpectedly small");

console.log(`Standard universe pack verified: ${coverage.data_ready} DataReady records, ${records.get("mechanic-rules.json").length} rules, ${fixtures.length} mechanic fixtures, ${index.pack_sha256}.`);
