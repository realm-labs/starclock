import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";

const root = "content-reference/pure-fiction-v1";
const manifest = JSON.parse(readFileSync("content-manifests/pure-fiction-v1/content-manifest.json", "utf8"));
const schema = JSON.parse(readFileSync(`${root}/schema.json`, "utf8"));
const documents = new Map();
for (const file of schema.normalized_files) {
  const document = JSON.parse(readFileSync(`${root}/${file}`, "utf8"));
  if (document.file !== file || !Array.isArray(document.records)) throw new Error(`invalid normalized document ${file}`);
  if (new Set(document.records.map((row) => row.id)).size !== document.records.length) throw new Error(`duplicate ID in ${file}`);
  documents.set(file, document);
}
const coverage = documents.get("coverage.json").records;
const sources = documents.get("sources.json").records;
const expected = new Set(manifest.obligations.map((row) => row.id));
const covered = new Set(coverage.map((row) => row.manifest_record_id));
if (coverage.length !== manifest.obligation_count || covered.size !== expected.size || [...expected].some((id) => !covered.has(id))) throw new Error("manifest coverage is not exact-once");
if (sources.length !== manifest.obligation_count || new Set(sources.map((row) => row.id)).size !== expected.size) throw new Error("source coverage mismatch");
if (documents.get("mechanic-rules.json").records.length !== 25) throw new Error("mechanic rule denominator drift");
if (documents.get("semantic-fixtures.json").records.length !== 18) throw new Error("semantic fixture denominator drift");
if (documents.get("research-gaps.json").records.some((row) => row.blocking)) throw new Error("blocking research gap remains");
for (const [file, document] of documents) {
  for (const row of document.records) {
    if (row.runtime_executable === true) throw new Error(`runtime leakage ${file}:${row.id}`);
    if (file !== "sources.json" && [row.name_en, row.name_zh_cn, row.summary_en, row.summary_zh_cn].some((value) => typeof value !== "string" || value.length === 0)) throw new Error(`bilingual envelope gap ${file}:${row.id}`);
    for (const id of row.manifest_record_ids ?? []) if (!expected.has(id)) throw new Error(`unknown manifest reference ${file}:${row.id}:${id}`);
  }
}
const files = readdirSync(root).filter((file) => file.endsWith(".json")).sort();
const digest = createHash("sha256");
for (const file of files) digest.update(file).update("\0").update(readFileSync(`${root}/${file}`));
console.log(`Pure Fiction pack verified: ${manifest.obligation_count} DataReady obligations, 25 rules, 18 fixtures, ${documents.get("pack-index.json").records.length} index rows, ${digest.digest("hex")}`);
