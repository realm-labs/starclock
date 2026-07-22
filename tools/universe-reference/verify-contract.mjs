import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(process.argv[2] ?? ".");
const read = async (relative) => JSON.parse(await readFile(path.join(root, ...relative.split("/")), "utf8"));
const schema = await read("content-reference/standard-universe-v1/schema.json");
const manifest = await read("content-manifests/standard-universe-v1/content-manifest.json");
const policy = await read("policy/standard-universe-reference.json");

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

assert(schema.schema === "starclock.standard-universe-normalized-schema.v1", "normalized schema revision drifted");
assert(manifest.schema === "starclock.standard-universe-content-manifest.v1", "content manifest revision drifted");
assert(policy.goal_id === "standard-universe-reference-v1", "policy goal ID drifted");
assert(schema.enums.quality.length === new Set(schema.enums.quality).size, "duplicate quality label");
assert(schema.enums.coverage_state.includes("DataReady"), "DataReady coverage state missing");
assert(schema.canonical_encoding.decimal_pattern === "^-?(0|[1-9][0-9]*)(\\.[0-9]{1,6})?$", "decimal grammar drifted");
assert(schema.files.length === new Set(schema.files).size, "duplicate normalized filename");

for (const [categoryId, category] of Object.entries(manifest.categories)) {
  assert(category.count === category.records.length, `${categoryId} count differs from records`);
  assert(category.membership_basis.length > 20, `${categoryId} lacks a reviewable membership basis`);
  const ids = category.records.map((record) => record.id);
  assert(ids.length === new Set(ids).size, `${categoryId} contains duplicate IDs`);
  for (const record of category.records) {
    assert(/^[0-9a-f]{64}$/u.test(record.evidence_sha256), `${categoryId}/${record.id} has invalid evidence digest`);
    assert(/^ExcelOutput\/.+\.json#[0-9]+$/u.test(record.source), `${categoryId}/${record.id} has invalid source locator`);
  }
}

assert(manifest.categories.worlds.count === policy.snapshot.world_count, "World denominator differs from policy");
assert(manifest.categories.paths.count === policy.snapshot.selectable_path_count, "Path denominator differs from policy");
assert(manifest.categories.blessings.count === 162, "Blessing denominator drifted");
assert(manifest.categories.blessing_levels.count === 324, "Blessing-level denominator drifted");
assert(manifest.categories.resonances_and_formations.count === 36, "Resonance denominator drifted");

console.log(`Standard universe normalized-data contract verified: ${Object.keys(manifest.categories).length} categories, ${schema.files.length} pack files.`);
